extern crate actix;
extern crate actix_web;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate futures;

extern crate plasma;

use std::thread::{self, sleep};
use std::sync::mpsc;
use std::time::Duration;

use actix::*;
use actix_web::ws;

use futures::Future;

#[macro_use]
extern crate glium;

use plasma::types::*;

pub mod client_types;
pub mod preview_state;

use self::client_types::{PreviewClient, ClientMessage};
use self::preview_state::PreviewState;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info,preview_client=info");
    env_logger::init();

    // Channel to pass messages from the Websocket client to the OpenGL window.
    let (tx, rx) = mpsc::channel();

    // Start the Websocket client on a separate thread so that it is not blocked
    // (and is not blocking) the OpenGL window.

    let client_handle = thread::spawn(|| {

        let sys = actix::System::new("preview client");

        // Start a WebSocket client and connect to the server.

        // FIXME check if server is up

        Arbiter::spawn(
            ws::Client::new("http://127.0.0.1:8080/ws/")
                .connect()
                .map_err(|e| {
                    error!("Can not connect to server: {}", e);
                    // FIXME wait and keep trying to connect in a loop
                    return;
                })
                .map(|(reader, writer)| {
                    let addr = PreviewClient::create(|ctx| {
                        PreviewClient::add_stream(reader, ctx);
                        PreviewClient{
                            writer: writer,
                            channel_sender: tx,
                        }
                    });

                    // FIXME ? maybe don't need the new thread

                    thread::spawn(move || {
                        //let msg = serde_json::to_string(&Sending{
                        //    data_type: MsgDataType::StartOpenGlPreview,
                        //    data: "".to_string(),
                        //}).unwrap();

                        //addr.do_send(ClientMessage{ data: msg });

                        // FIXME ? should avoid this loop
                        loop {
                            sleep(Duration::from_secs(1));
                        }
                    });

                    // FIXME client is exiting too early, heartbeat fails

                    ()
                }),
        );

        sys.run();
    });

    // Start OpenGL window on the main thread.

    let mut state = PreviewState::new().unwrap();
    state.start_opengl_preview(rx);

    client_handle.join().unwrap();

    info!("gg thx!");
}