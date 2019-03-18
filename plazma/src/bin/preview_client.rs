#[macro_use]
extern crate log;
extern crate env_logger;

extern crate plazma;

use std::sync::Arc;

use plazma::app::start_preview;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info,plazma=info,preview_client=info,rocket_client=info,rocket_sync=info");
    env_logger::init();

    let plazma_server_port = Arc::new(8080);

    let client_handle = start_preview(&plazma_server_port).unwrap();

    client_handle.join().unwrap();

    info!("gg thx!");
}


