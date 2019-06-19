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

use clap::App;

use plazma::app;

fn main() {
    match kankyo::load() {
        Ok(_) => {}
        Err(e) => info!("Couldn't find a .env file: {:?}", e),
    };
    //std::env::set_var("RUST_LOG", "actix_web=info,plazma=info");
    env_logger::init();
    info!("🚀 Launched");

    let app_info = app::app_info().unwrap();

    info!("🔎 CWD: {}", &app_info.cwd.to_str().unwrap());
    info!(
        "🔎 Path to binary: {}",
        &app_info.path_to_binary.to_str().unwrap()
    );

    // --- CLI options ---

    let cli_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_yaml).get_matches();
    let app_params = app::process_cli_args(matches).unwrap();

    // --- OpenGL preview window ---

    // Starts on the main thread. The render loop is blocking until it exits.

    let port = app_params.plazma_server_port.clone();
    if app_params.is_preview {
        let p = if let Some(ref path) = app_params.yml_path.clone() {
            Some(PathBuf::from(&path))
        } else {
            None
        };
        app::start_preview(port, p).unwrap();
    };

    // --- HTTP and WebSocket server ---

    // Starts on the main thread and blocking until exits. It will start a dialogs process, a
    // webview or NWJS window according to params.

    let port = app_params.plazma_server_port.clone();
    if app_params.is_server {
        app::start_server(port, app_info, app_params.clone()).unwrap();
    }

    // --- Dialogs ---

    // Start a separate process where dialogs can block the main thread.

    let port = app_params.plazma_server_port.clone();
    if app_params.is_dialogs {
        app::start_dialogs(port).unwrap();
    };

    // --- WebView ---

    // Starts on the main thread. The webview window is blocking until it is closed.

    let port = app_params.plazma_server_port.clone();
    if app_params.is_webview {
        app::start_webview(port).unwrap();
    }

    // --- NWJS ---

    // Starts on the main thread.

    let port = app_params.plazma_server_port.clone();
    if app_params.is_nwjs {
        app::start_nwjs(port, &app_params.nwjs_path).unwrap();
    }

    info!("🍉 gg thx! 😎");
}
