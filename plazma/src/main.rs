#[macro_use]
extern crate clap;
extern crate web_view;
extern crate actix;
extern crate actix_web;

extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate kankyo;
extern crate futures;
extern crate glutin;

extern crate rocket_client;
extern crate rocket_sync;
extern crate plazma;

//use std::env;
use std::thread;
use clap::App;

use plazma::app::{AppStartParams, process_cli_args, start_server, start_webview, start_preview};

fn main() {
    kankyo::load().unwrap();
    //std::env::set_var("RUST_LOG", "actix_web=info,plazma=info");
    env_logger::init();

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
        start_preview: param_start_preview
    } = process_cli_args(matches).unwrap();

    // --- HTTP and WebSocket server ---

    let server_handle = if param_start_server {
        start_server(&plazma_server_port, yml_path).unwrap()
    } else {
        thread::spawn(|| {})
    };

    // --- OpenGL preview window ---

    let preview_handle = if param_start_preview {
        start_preview(&plazma_server_port).unwrap()
    } else {
        thread::spawn(|| {})
    };

    // --- WebView ---

    // Blocking until webview window is closed.

    if param_start_webview {
        start_webview(&plazma_server_port).unwrap();
    }

    preview_handle.join().unwrap();

    server_handle.join().unwrap();

    info!("gg thx!");
}
