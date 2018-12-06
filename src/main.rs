extern crate clap;
extern crate web_view;
extern crate hyper;
extern crate actix;
extern crate actix_web;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate plasma;

use std::sync::{Arc, Mutex};
use std::thread;

use hyper::rt;
use hyper::Client as HyperClient;
use hyper::rt::Future as HyperFuture;

use clap::App as ClApp;

use web_view::Content;

use actix_web::{fs, middleware, server, ws, App, HttpRequest, HttpResponse};
use actix_web::Error as AxError;
use actix_web::actix::*;

use plasma::types::*;

fn static_index(_req: &HttpRequest<ServerStateWrap>) -> Result<fs::NamedFile, AxError> {
    Ok(fs::NamedFile::open("./gui-static/index.html")?)
}

fn stop_server(_req: &HttpRequest<ServerStateWrap>) -> Result<HttpResponse, AxError> {
    System::current().stop();
    Ok(HttpResponse::Ok()
       .content_type("text/plain")
       .body("g2g"))
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info,plasma=info");
    env_logger::init();

    // --- CLI options ---

    let _matches = ClApp::new("Plasma")
        .version("0.1.0")
        .get_matches();

    // --- HTTP and WebSocket server ---

    let server_handle = thread::spawn(|| {

        let sys = actix::System::new("plasma server");

        let server_state = Arc::new(Mutex::new(ServerState::new()));

        server::new(move || {

            App::with_state(server_state.clone())
            // logger
                .middleware(middleware::Logger::default())
            // WebSocket routes (there is no CORS)
                .resource("/ws/", |r| r.f(|req| ws::start(req, WsActor::new())))
            // tell the server to stop
                .resource("/stop_server",
                          |r| r.get().f(stop_server))
            // static files
                .handler("/static/", fs::StaticFiles::new("./gui/src/static/").unwrap()
                         .default_handler(static_index))
        })
            .bind("127.0.0.1:8080")
            .unwrap()
            .start();

        sys.run();
    });

    // --- WebView ---

    {
        web_view::builder()
            .title("Plasma")
            .content(Content::Url("http://localhost:8080/static/"))
            .size(1366, 768)
            .resizable(true)
            .debug(true)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .run()
            .unwrap();

        // Blocked until gui exits. Then it hits the /stop_server url.

        let url = "http://localhost:8080/stop_server".parse::<hyper::Uri>().unwrap();
        rt::run(fetch_url(url));
    }

    server_handle.join().unwrap();

    info!("gg thx!");
}

// FIXME use actix as HTTP client intead of hyper
fn fetch_url(url: hyper::Uri) -> impl HyperFuture<Item=(), Error=()> {
    let client = HyperClient::new();
    client.get(url).map(|_| {}).map_err(|_| {})
}

