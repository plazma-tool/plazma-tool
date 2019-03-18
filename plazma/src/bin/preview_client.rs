#![allow(non_snake_case)]

extern crate actix;
extern crate actix_web;

extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate futures;

extern crate glutin;

extern crate rocket_client;
extern crate rocket_sync;
extern crate plazma;

use std::sync::Arc;

use plazma::app::start_preview;

fn main() {
    //std::env::set_var("RUST_LOG", "actix_web=info,plazma=info,preview_client=info");
    std::env::set_var("RUST_LOG", "actix_web=info,plazma=info,preview_client=info,rocket_client=info,rocket_sync=info");
    env_logger::init();

    let plazma_server_port = Arc::new(8080);

    let preview_handle = start_preview(&plazma_server_port).unwrap();

    preview_handle.join().unwrap();

    info!("gg thx!");
}


