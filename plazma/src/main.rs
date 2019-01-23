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

extern crate plazma;

use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
//use std::time::Duration;
use std::path::PathBuf;
//use std::error::Error;

use clap::App as ClApp;
use clap::{Arg, SubCommand};

use web_view::Content;

use actix_web::{fs, middleware, server, client, ws, App, HttpRequest, HttpResponse};
use actix_web::Error as AxError;
use actix_web::actix::*;

use futures::Future;

use plazma::server_actor::{ServerActor, ServerState, ServerStateWrap};

fn static_index(_req: &HttpRequest<ServerStateWrap>) -> Result<fs::NamedFile, AxError> {
    Ok(fs::NamedFile::open("../gui/build/index.html")?)
}

fn stop_server(_req: &HttpRequest<ServerStateWrap>) -> Result<HttpResponse, AxError> {
    System::current().stop();
    Ok(HttpResponse::Ok()
       .content_type("text/plain")
       .body("g2g"))
}

fn main() {
    kankyo::load().unwrap();
    std::env::set_var("RUST_LOG", "actix_web=info,plazma=info");
    env_logger::init();

    let plazma_server_port = Arc::new(8080);

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

    let matches = ClApp::new("Plazma")
        .version("0.1.0")
        .author("etd <erethedaybreak@gmail.com>")
        .subcommand(SubCommand::with_name("open")
                    .about("open a demo")
                    .arg(Arg::with_name("yml")
                         .long("yml")
                         .value_name("FILE")
                         .required(true)
                         .takes_value(true)
                         .help("YAML demo file")))
        .get_matches();

    // --- Process CLI args ---

    let demo_yml_path: PathBuf;

    if let Some(m) = matches.subcommand_matches("open") {

        demo_yml_path = PathBuf::from(m.value_of("yml").unwrap());

    } else {
        // No CLI subcommands were given. Try default locations for a demo.yml

        // ./demo.yml
        let a = PathBuf::from("demo.yml".to_owned());
        // ./data/demo.yml
        let b = PathBuf::from("data".to_owned()).join(PathBuf::from("demo.yml".to_owned()));
        if a.exists() {
            demo_yml_path = a;
        } else if b.exists() {
            demo_yml_path = b;
        } else {
            // No subcommands were given and default locations don't exist.
            // Start with a minimal default demo.

            // ./data/minimal/demo.yml
            demo_yml_path = PathBuf::from("data".to_owned())
                .join(PathBuf::from("minimal".to_owned()))
                .join(PathBuf::from("demo.yml".to_owned()));
        }
    }

    // --- HTTP and WebSocket server ---

    let plazma_server_port_a = Arc::clone(&plazma_server_port);

    let server_handle = thread::spawn(move || {

        let sys = actix::System::new("plazma server");

        let server_state = Arc::new(Mutex::new(ServerState::new(&demo_yml_path).unwrap()));

        server::new(move || {

            App::with_state(server_state.clone())
            // logger
                .middleware(middleware::Logger::default())
            // WebSocket routes (there is no CORS)
                .resource("/ws/", |r| r.f(|req| ws::start(req, ServerActor::new())))
            // tell the server to stop
                .resource("/stop_server",
                          |r| r.get().f(stop_server))
            // static files
                .handler("/static/", fs::StaticFiles::new("../gui/build/").unwrap()
                         .default_handler(static_index))
        })
            .bind(format!{"127.0.0.1:{}", plazma_server_port_a})
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
        format!{"http://localhost:{}/static/", plazma_server_port}
    };

    {
        web_view::builder()
            .title("Plazma")
            .content(Content::Url(content_url))
            .size(1366, 768)
            .resizable(true)
            .debug(true)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .run()
            .unwrap();

        // Blocked until gui exits. Then it hits the /stop_server url.

        let url = format!{"http://localhost:{}/stop_server", plazma_server_port};

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

