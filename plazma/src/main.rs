#[macro_use]
extern crate clap;
extern crate actix;
extern crate actix_web;
extern crate nfd;
extern crate web_view;

extern crate serde;
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
extern crate glutin;
extern crate kankyo;

extern crate plazma;
extern crate rocket_client;
extern crate rocket_sync;

use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};

use clap::App;

use plazma::app::{self, AppStartParams};
use plazma::server_actor::{MsgDataType, Sending};

fn main() {
    match kankyo::load() {
        Ok(_) => {}
        Err(e) => info!("Couldn't find a .env file: {:?}", e),
    };
    //std::env::set_var("RUST_LOG", "actix_web=info,plazma=info");
    env_logger::init();
    info!("🚀 Launched");

    let app_info = app::app_info().unwrap();

    info!("🔎 CWD: {:?}", &app_info.cwd);
    info!("🔎 Path to binary: {:?}", &app_info.path_to_binary);

    // --- CLI options ---

    let cli_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_yaml).get_matches();

    // Process the cli args and destructure the members to separate owned variables.

    let AppStartParams {
        yml_path,
        dmo_path: _,
        plazma_server_port,
        start_server: param_start_server,
        start_webview: param_start_webview,
        start_preview: param_start_preview,
        is_dialogs: param_is_dialogs,
    } = app::process_cli_args(matches).unwrap();

    // --- OpenGL preview window ---

    // Starts on the main thread. The render loop is blocking until it exits.

    let port_a = plazma_server_port.clone();
    if param_start_preview {
        let p = if let Some(ref path) = yml_path {
            Some(PathBuf::from(&path))
        } else {
            None
        };
        app::start_preview(port_a, p).unwrap();
    };

    // --- HTTP and WebSocket server ---

    // Starts on a spawned thread. It allows to start the server before the webview window starts
    // blocking.

    // Channel to pass messages from the server to the webview window.
    let (webview_sender, webview_receiver) = mpsc::channel();
    let webview_sender_arc = Arc::new(Mutex::new(webview_sender));

    // Channel to pass messages from the webview to the server.
    let (server_sender, server_receiver) = mpsc::channel();

    let port_b = plazma_server_port.clone();
    let (server_handle, server_receiver_handle, client_receiver_handle) = if param_start_server {
        let (a, b, c) = app::start_server(
            port_b,
            app_info,
            yml_path,
            webview_sender_arc,
            server_receiver,
        )
        .unwrap();
        (Some(a), Some(b), Some(c))
    } else {
        (None, None, None)
    };

    // --- Dialogs ---

    // Start a separate process where dialogs can block the main thread.

    let port_b = plazma_server_port.clone();
    if param_is_dialogs {
        app::start_dialogs(port_b).unwrap();
    };

    // --- WebView ---

    // Starts on the main thread. The webview window is blocking until it is closed.

    let server_sender_arc = Arc::new(Mutex::new(server_sender));
    let sender_a = server_sender_arc.clone();
    let sender_b = server_sender_arc.clone();

    let port_c = plazma_server_port.clone();
    if param_start_webview {
        app::start_webview(port_c, webview_receiver, sender_a).unwrap();

        // Send ExitApp to the server, in case it is still running. This can happen when the window
        // manager is used to close the window, not the close button in the web UI.

        let msg = serde_json::to_string(&Sending {
            data_type: MsgDataType::ExitApp,
            data: "".to_owned(),
        })
        .unwrap();

        let sender = sender_b.lock().expect("Can't lock server sender.");
        match sender.send(msg) {
            Ok(_) => {}
            Err(e) => error!("🔥 Can't send on user_data.server_sender: {:?}", e),
        }
    }

    if let Some(h) = client_receiver_handle {
        h.join().unwrap();
    }

    if let Some(h) = server_receiver_handle {
        h.join().unwrap();
    }

    if let Some(h) = server_handle {
        h.join().unwrap();
    }

    info!("🍉 gg thx! 😎");
}
