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

mod types;
mod preview_client;
mod preview_state;
mod utils;

use crate::preview_client::{PreviewClient, PreviewClientCodec, start_opengl_preview};

use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use std::net;
use std::str::FromStr;

use hyper::{rt, Client};
use hyper::rt::Future as HyperFuture;

use clap::App as ClApp;

use web_view::Content;

use actix_web::{http, fs, middleware, server, ws, App, HttpRequest, HttpResponse, Json};
use actix_web::Error as AxError;
use actix_web::actix::*;
use actix_web::middleware::cors::Cors;

use futures::Future; // needed for .and_then()
use tokio_codec::FramedRead;
use tokio_io::AsyncRead; // needed for .split()
use tokio_tcp::TcpStream;

#[macro_use]
extern crate glium;

use crate::types::*;

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
                .resource("/", |r| r.f(|req| ws::start(req, WsActor{ client_id: 0 })))
            // tell the server to stop
                .resource("/stop_server",
                          |r| r.get().f(stop_server))
            // static files
                .handler("/static", fs::StaticFiles::new("./gui-static/").unwrap()
                         .default_handler(static_index))
        })
            .bind("127.0.0.1:8080")
            .unwrap()
            .start();

        sys.run();
    });

    // --- OpenGL Preview ---

    let preview_handle = thread::spawn(|| {

        // Start a WebSocket client

        actix_web::actix::System::run(|| {

            // FIXME find out if server is up
            sleep(Duration::from_millis(1000));

            let (tx, rx) = mpsc::channel();

            let server_addr = net::SocketAddr::from_str("127.0.0.1:8080").unwrap();

            Arbiter::spawn(
                TcpStream::connect(&server_addr)
                    .and_then(move |stream| {
                        let addr = PreviewClient::create(|ctx| {
                            let (r, w) = stream.split();
                            ctx.add_stream(FramedRead::new(r, PreviewClientCodec));
                            PreviewClient {
                                framed: actix::io::FramedWrite::new(
                                    w,
                                    PreviewClientCodec,
                                    ctx,
                                ),
                            }
                        });

                        thread::spawn(move || {
                            start_opengl_preview(&addr);
                            tx.send("stop".to_string()).unwrap();
                        });

                        let _msg = rx.recv().unwrap();
                        System::current().stop();

                        futures::future::ok(())
                    })
                    .map_err(|e| {
                        println!("Can not connect to server: {}", e);
                        // FIXME wait and keep trying to connect in a loop
                        return;
                    }),
            );

        });

        // NOTE blocking here freezes the computer.

    });

    // --- WebView ---

    {
        web_view::builder()
            .title("Plasma")
            .content(Content::Url("http://localhost:8080/static"))
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

    // Close OpenGL Preview window to exit.
    preview_handle.join().unwrap();

    info!("gg thx!");
}

fn fetch_url(url: hyper::Uri) -> impl HyperFuture<Item=(), Error=()> {
    let client = Client::new();
    client.get(url).map(|_| {}).map_err(|_| {})
}

