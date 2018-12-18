extern crate clap;
extern crate web_view;
extern crate actix;
extern crate actix_web;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate kankyo;

extern crate plasma;

use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
//use std::error::Error;

use clap::App as ClApp;

use web_view::Content;

use actix_web::{fs, middleware, server, client, http, ws, App, HttpRequest, HttpResponse};
use actix_web::Error as AxError;
use actix_web::actix::*;

use futures::Future;

use plasma::types::*;

fn static_index(_req: &HttpRequest<ServerStateWrap>) -> Result<fs::NamedFile, AxError> {
    Ok(fs::NamedFile::open("./gui/build/index.html")?)
}

fn stop_server(_req: &HttpRequest<ServerStateWrap>) -> Result<HttpResponse, AxError> {
    System::current().stop();
    Ok(HttpResponse::Ok()
       .content_type("text/plain")
       .body("g2g"))
}

fn main() {
    kankyo::load().unwrap();
    std::env::set_var("RUST_LOG", "actix_web=info,plasma=info");
    env_logger::init();

    let plasma_server_port = Arc::new(8080);

    // In development mode, use the React dev server port.
    let react_server_port: Option<usize> = match env::var("MODE") {
        Ok(x) => {
            if x == "development" {
                Some(3000)
            } else {
                None
            }
        },
        Err(_) => None
    };

    // --- CLI options ---

    let _matches = ClApp::new("Plasma")
        .version("0.1.0")
        .get_matches();

    // --- HTTP and WebSocket server ---

    let plasma_server_port_a = Arc::clone(&plasma_server_port);

    let server_handle = thread::spawn(move || {

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
                .handler("/static/", fs::StaticFiles::new("./gui/build/").unwrap()
                         .default_handler(static_index))
        })
            .bind(format!{"127.0.0.1:{}", plasma_server_port_a})
            .unwrap()
            .start();

        sys.run();
    });

    // --- WebView ---

    // If the React dev server is running, load content from there. If not, load
    // our static files route which is serving the React build directory.
    let content_url = if let Some(port) = react_server_port {
        format!{"http://localhost:{}/static/", port}
    } else {
        format!{"http://localhost:{}/static/", plasma_server_port}
    };

    {
        web_view::builder()
            .title("Plasma")
            .content(Content::Url(content_url))
            .size(1366, 768)
            .resizable(true)
            .debug(true)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .run()
            .unwrap();

        // Blocked until gui exits. Then it hits the /stop_server url.

        let url = format!{"http://localhost:{}/stop_server", plasma_server_port};

        actix::run(|| {
            client::get(url)
                .finish().unwrap()
                .send()
                .map_err(|err| {
                    error!("Error: {:?}", err);
                    ()
                })
                .and_then(|response| {
                    info!("Response: {:?}", response);
                    Ok(())
                })
        });
    }

    server_handle.join().unwrap();

    info!("gg thx!");
}

