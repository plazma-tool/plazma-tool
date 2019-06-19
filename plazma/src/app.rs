use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{exit, Command};
use std::sync::mpsc::TryRecvError;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

use nfd::Response as NfdResponse;
use web_view::Content;

use mime_guess::guess_mime_type;

use actix_web::actix::*;
use actix_web::http::Method;
use actix_web::{middleware, server, ws, App, Body, HttpRequest, HttpResponse};

use reqwest;

use futures::Future;

use glutin::dpi::LogicalSize;
use glutin::{ElementState, Event, EventsLoop, GlContext, GlWindow, WindowEvent};

use intro_3d::lib::Vector3;
use rocket_client::SyncClient;

use crate::error::ToolError;
use intro_runtime::error::RuntimeError;

use crate::preview_client::client_actor::{ClientActor, ClientMessage};
use crate::server_actor::{
    MsgDataType, Receiving, Sending, ServerActor, ServerState, ServerStateWrap, SetDmoMsg,
    SetShaderMsg, ShaderCompilationFailedMsg, ShaderCompilationSuccessMsg,
};
use crate::server_init_actor::{self, ServerInitActor};
use crate::webview_actor::{self, WebviewActor};
use crate::nwjs_actor::{self, NwjsActor};

use crate::preview_client::preview_state::PreviewState;
use crate::utils::clean_windows_str_path;

#[derive(RustEmbed)]
#[folder = "../gui/build/"]
pub struct WebAsset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match WebAsset::get(path) {
        Some(content) => {
            let body: Body = match content {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };
            HttpResponse::Ok()
                .content_type(guess_mime_type(path).as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

fn static_index(_req: HttpRequest<ServerStateWrap>) -> HttpResponse {
    handle_embedded_file("index.html")
}

fn static_assets(req: HttpRequest<ServerStateWrap>) -> HttpResponse {
    let path = &req.path().trim_start_matches("/static/");
    handle_embedded_file(path)
}

fn fonts_assets(req: HttpRequest<ServerStateWrap>) -> HttpResponse {
    let path = &req.path().trim_start_matches('/');
    handle_embedded_file(path)
}

#[derive(Clone, Debug)]
pub struct AppStartParams {
    pub yml_path: Option<PathBuf>,
    pub plazma_server_port: Arc<usize>,
    pub url: String,
    pub nwjs_path: PathBuf,
    pub start_dialogs: bool,
    pub start_webview: bool,
    pub start_nwjs: bool,
    // FIXME these are exclusive states, should be an enum
    pub is_server: bool,
    pub is_preview: bool,
    pub is_webview: bool,
    pub is_nwjs: bool,
    pub is_dialogs: bool,
}

pub struct AppInfo {
    pub cwd: PathBuf,
    pub path_to_binary: PathBuf,
    pub pid: u32,
}

impl Default for AppStartParams {
    fn default() -> AppStartParams {
        if cfg!(target_os = "windows") {
            AppStartParams {
                yml_path: None,
                plazma_server_port: Arc::new(8080),
                url: "http://localhost:8080/static/".to_owned(),
                nwjs_path: PathBuf::from(".").join(PathBuf::from("nwjs").join(PathBuf::from("nwjs.exe"))),
                start_dialogs: true,
                start_webview: false,
                start_nwjs: true,
                is_server: true,
                is_preview: false,
                is_webview: false,
                is_nwjs: false,
                is_dialogs: false,
            }
        } else {
            AppStartParams {
                yml_path: None,
                plazma_server_port: Arc::new(8080),
                url: "http://localhost:8080/static/".to_owned(),
                nwjs_path: PathBuf::from(".").join(PathBuf::from("nwjs").join(PathBuf::from("nwjs"))),
                start_dialogs: true,
                start_webview: true,
                start_nwjs: false,
                is_server: true,
                is_preview: false,
                is_webview: false,
                is_nwjs: false,
                is_dialogs: false,
            }
        }
    }
}

impl AppStartParams {
    fn default_with_port(port: usize) -> AppStartParams {
        let mut params = AppStartParams::default();
        params.plazma_server_port = Arc::new(port);
        params.url = format!("http://localhost:{}/static/", port);
        params
    }
}

pub fn app_info() -> Result<AppInfo, Box<dyn Error>> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    let mut path_to_binary = cwd.clone();

    if let Some(a) = std::env::args().nth(0) {
        path_to_binary = path_to_binary.join(PathBuf::from(a));
    } else if cfg!(target_os = "windows") {
        path_to_binary = path_to_binary.join(PathBuf::from("plazma.exe".to_owned()));
    } else {
        path_to_binary = path_to_binary.join(PathBuf::from("plazma".to_owned()));
    }
    path_to_binary = path_to_binary.canonicalize()?;

    if !path_to_binary.exists() {
        return Err(From::from(format!(
            "🔥 Path does not exist: {:?}",
            &path_to_binary
        )));
    }

    Ok(AppInfo {
        cwd,
        path_to_binary,
        pid: std::process::id(),
    })
}

pub fn process_cli_args(matches: clap::ArgMatches) -> Result<AppStartParams, Box<dyn Error>> {
    let server_port = match matches.value_of("port").unwrap().parse::<usize>() {
        Ok(x) => x,
        Err(e) => {
            error! {"🔥 {:?}", e};
            exit(2);
        }
    };

    let mut params = AppStartParams::default_with_port(server_port);

    if matches.is_present("yml") {
        if let Ok(x) = matches.value_of("yml").unwrap().parse::<String>() {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.yml_path = Some(path);
            } else {
                error!("🔥 Path does not exist: {:?}", &path);
                exit(2);
            }
        }
    }

    if matches.is_present("url") {
        if let Ok(x) = matches.value_of("url").unwrap().parse::<String>() {
            params.url = x;
        }
    }

    if matches.is_present("nwjs_path") {
        if let Ok(x) = matches.value_of("nwjs_path").unwrap().parse::<String>() {
            let path = PathBuf::from(&x);
            if path.exists() {
                params.nwjs_path = path;
            } else {
                error!("🔥 Path does not exist: {:?}", &path);
            }
        }
    }

    if matches.subcommand_matches("server").is_some() {

        params.start_dialogs = false;
        params.start_webview = false;
        params.start_nwjs = false;

    } else if matches.subcommand_matches("preview").is_some() {

        params.is_preview = true;
        params.is_server = false;
        params.start_dialogs = false;
        params.start_webview = false;
        params.start_nwjs = false;

    } else if matches.subcommand_matches("webview").is_some() {

        params.is_webview = true;
        params.is_nwjs = false;
        params.is_server = false;
        params.start_dialogs = false;
        params.start_webview = false;
        params.start_nwjs = false;

    } else if matches.subcommand_matches("nwjs").is_some() {

        params.is_webview = false;
        params.is_nwjs = true;
        params.is_server = false;
        params.start_dialogs = false;
        params.start_webview = false;
        params.start_nwjs = false;

    } else if matches.subcommand_matches("dialogs").is_some() {

        params.is_dialogs = true;
        params.is_server = false;
        params.start_dialogs = false;
        params.start_webview = false;
        params.start_nwjs = false;

    };

    if matches.is_present("with_nwjs") {
        params.start_webview = false;
        params.start_nwjs = true;
    }

    if matches.is_present("with_webview") {
        params.start_webview = true;
        params.start_nwjs = false;
    }

    Ok(params)
}

#[allow(clippy::type_complexity)]
pub fn start_server(
    port: Arc<usize>,
    app_info: AppInfo,
    app_params: AppStartParams,
) -> Result< (), Box<dyn Error>,
> {
    info!("⚽ start_server() start");

    let port_clone_a = Arc::clone(&port);
    let port_clone_b = Arc::clone(&port);

    let app_params_a = app_params.clone();
    let server_handle = thread::spawn(move || {
        info!("🧵 new thread: server");
        let sys = actix::System::new("plazma server");

        info!("ServerState::new() using yml_path: {:?}", &app_params_a.yml_path);

        let server_state =
            Arc::new(Mutex::new(ServerState::new(
                        app_info,
                        app_params_a.clone(),
                        app_params_a.yml_path.clone()
                        ).unwrap()));

        server::new(move || {
            App::with_state(server_state.clone())
                // logger
                .middleware(middleware::Logger::default())
                // WebSocket routes (there is no CORS)
                .resource("/ws/", |r| r.f(|req| ws::start(req, ServerActor::new())))
                // static files
                .route("/static/", Method::GET, static_index)
                .route("/static/{_:.*}", Method::GET, static_assets)
                .route("/fonts/{_:.*}", Method::GET, fonts_assets)
        })
        .bind(format! {"127.0.0.1:{}", port_clone_a})
            .unwrap()
            .start();

        sys.run();
    });

    let server_init_handle = thread::spawn(move || {
        info!("🧵 new thread: server init client");

        let sys = actix::System::new("server init client");

        // Start a WebSocket client and connect to the server.

        // Check if server is up.
        loop {
            if let Ok(resp) = reqwest::get(&format!{"http://localhost:{}/static/", port_clone_b}) {
                if resp.status().is_success() {
                    break;
                }
            }
            sleep(Duration::from_millis(100));
        }

        Arbiter::spawn(
            ws::Client::new(format! {"http://127.0.0.1:{}/ws/", port_clone_b})
                .connect()
                .map_err(|e| {
                    error!("🔥 ⚔️  Can not connect to server: {}", e);
                })
                .map(move |(reader, writer)| {
                    let addr = ServerInitActor::create(|ctx| {
                        ServerInitActor::add_stream(reader, ctx);
                        ServerInitActor { writer }
                    });

                    if app_params.start_dialogs {
                        let msg = serde_json::to_string(&Sending {
                            data_type: MsgDataType::StartDialogs,
                            data: "".to_owned(),
                        })
                        .unwrap();
                        addr.do_send(server_init_actor::ClientMessage { data: msg });
                    }

                    if app_params.start_webview {
                        let msg = serde_json::to_string(&Sending {
                            data_type: MsgDataType::StartWebview,
                            data: "".to_owned(),
                        })
                        .unwrap();
                        addr.do_send(server_init_actor::ClientMessage { data: msg });
                    }

                    if app_params.start_nwjs {
                        let msg = serde_json::to_string(&Sending {
                            data_type: MsgDataType::StartNwjs,
                            data: "".to_owned(),
                        })
                        .unwrap();
                        addr.do_send(server_init_actor::ClientMessage { data: msg });
                    }
                }),
        );

        sys.run();
    });

    server_init_handle.join().unwrap();

    server_handle.join().unwrap();

    info!("🏁 start_server() finish");

    Ok(())
}

pub fn start_webview(plazma_server_port: Arc<usize>) -> Result<(), Box<dyn Error>> {
    info!("⚽ start_webview() start");

    // In development mode, use the React dev server port.
    let react_server_port: Option<usize> = match env::var("MODE") {
        Ok(x) => {
            if x == "development" {
                Some(3000)
            } else {
                None
            }
        }
        Err(_) => None,
    };

    // If the React dev server is running, load content from there. If not, load
    // our static files route which is serving the React build directory.
    let content_url = if let Some(port) = react_server_port {
        format! {"http://localhost:{}/static/", port}
    } else {
        let a = Arc::clone(&plazma_server_port);
        format! {"http://localhost:{}/static/", a}
    };

    {
        let webview = web_view::builder()
            .title("Plazma")
            .content(Content::Url(content_url.clone()))
            .size(1366, 768)
            .resizable(true)
            .debug(true)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .build()
            .unwrap();

        let webview_handle = webview.handle();

        // When the application window is reloaded (such as when the user is requesting a reload,
        // or when the React dev server recompiles and reloads after a code change), the
        // window.external object is lost and the invoke_handler() above is not accessible.
        //
        // Instead, we can receive messages via WebSocket. We terminate the webview when the server
        // disconnects.
        //
        // Dialog windows can't be opened here such as `webview.dialog().open_file()`, because (a)
        // it blocks the webview rendering thread and both the UI and the dialog freezes, and (b)
        // dialogs have to be opened from the main thread, otherwise it errors and panics. So
        // dialog windows we open in a separate process instead where the dialog is allowed to
        // block on the main thread.

        // use std::sync::mpsc;
        //mpsc::Sender<String>
        let (client_sender, client_receiver) = mpsc::channel();

        let webview_client_handle = thread::spawn(move || {
            info!("🧵 new thread: webview client");

            let sys = actix::System::new("webview client");

            // Start a WebSocket client and connect to the server.
            // It's purpose is to receive messages and terminate the webview when the server
            // disconnects.

            // Check if server is up.
            loop {
                if let Ok(resp) = reqwest::get(&content_url) {
                    if resp.status().is_success() {
                        break;
                    }
                }
                sleep(Duration::from_millis(100));
            }

            Arbiter::spawn(
                ws::Client::new(format! {"http://127.0.0.1:{}/ws/", plazma_server_port})
                .connect()
                .map_err(|e| {
                    error!("🔥 ⚔️  Can not connect to server: {}", e);
                })
                .map(move |(reader, writer)| {
                    let addr = WebviewActor::create(|ctx| {
                        WebviewActor::add_stream(reader, ctx);
                        WebviewActor {
                            writer,
                            webview_handle,
                        }
                    });

                    thread::spawn(move || {
                        info!("🧵 new thread: client receiver from webview");
                        loop {
                            if let Ok(text) = client_receiver.try_recv() {
                                info!("Webview thread: passing on message to server: {}", text);
                                addr.do_send(webview_actor::ClientMessage { data: text });
                            }
                            sleep(Duration::from_millis(100));
                        }
                    });

                }),
                );

            sys.run();
        });


        // This will block until the window is closed.
        info!("Run the webview.");
        webview.run()?;

        // Send ExitApp to the server, in case it is still running. This can happen when the window
        // manager is used to close the window, not the close button in the web UI.
        info!("Webview exited, send ExitApp to the server");

        let msg = serde_json::to_string(&Sending {
            data_type: MsgDataType::ExitApp,
            data: "".to_owned(),
        })
        .unwrap();

        match client_sender.send(msg) {
            Ok(_) => {}
            Err(e) => error!("🔥 Can't send on client_sender: {:?}", e),
        }

        webview_client_handle.join().unwrap();
    }

    info!("🏁 start_webview() finish");
    Ok(())
}

pub fn start_nwjs(plazma_server_port: Arc<usize>, path_to_nwjs: &PathBuf) -> Result<(), Box<dyn Error>> {
    info!("⚽ start_nwjs() start");

    // In development mode, use the React dev server port.
    let react_server_port: Option<usize> = match env::var("MODE") {
        Ok(x) => {
            if x == "development" {
                Some(3000)
            } else {
                None
            }
        }
        Err(_) => None,
    };

    // If the React dev server is running, load content from there. If not, load
    // our static files route which is serving the React build directory.
    let content_url = if let Some(port) = react_server_port {
        format! {"http://localhost:{}/static/", port}
    } else {
        let a = Arc::clone(&plazma_server_port);
        format! {"http://localhost:{}/static/", a}
    };

    let s = path_to_nwjs.to_str().unwrap();
    let bin_cmd = if cfg!(target_os = "windows") {
        format!{"{} --url='{}'", clean_windows_str_path(s), content_url}
    } else {
        format!{"{} --url='{}'", s, content_url}
    };

    let mut nwjs_child = if cfg!(target_os = "windows") {
        match Command::new("cmd").arg("/C").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned NWJS");
                child
            }
            Err(e) => {
                error!("🔥 failed to spawn NWJS: {:?}", e);
                exit(2);
            }
        }
    } else {
        match Command::new("sh").arg("-c").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned NWJS");
                child
            }
            Err(e) => {
                error!("🔥 failed to spawn NWJS: {:?}", e);
                exit(2);
            }
        }
    };

    let (client_sender, client_receiver) = mpsc::channel();

    let nwjs_client_handle = thread::spawn(move || {
        info!("🧵 new thread: NWJS client");

        let sys = actix::System::new("nwjs client");

        // Start a WebSocket client and connect to the server.
        //
        // It's purpose is to receive messages and terminate the NWJS process when the server
        // disconnects.

        // Check if server is up.
        loop {
            if let Ok(resp) = reqwest::get(&content_url) {
                if resp.status().is_success() {
                    break;
                }
            }
            sleep(Duration::from_millis(100));
        }

        Arbiter::spawn(
            ws::Client::new(format! {"http://127.0.0.1:{}/ws/", plazma_server_port})
            .connect()
            .map_err(|e| {
                error!("🔥 ⚔️  Can not connect to server: {}", e);
            })
            .map(move |(reader, writer)| {
                let addr = NwjsActor::create(|ctx| {
                    NwjsActor::add_stream(reader, ctx);
                    NwjsActor { writer }
                });

                thread::spawn(move || {
                    info!("🧵 new thread: client receiver from NWJS");
                    loop {
                        if let Ok(text) = client_receiver.try_recv() {
                            info!("NWJS thread: passing on message to server: {}", text);
                            addr.do_send(nwjs_actor::ClientMessage { data: text });
                        }
                        sleep(Duration::from_millis(100));
                    }
                });

            }),
            );

            sys.run();
    });

    // This will block until the window is closed.
    info!("Wait until NWJS exits.");
    nwjs_child.wait()?;

    // Send ExitApp to the server, in case it is still running. This can happen when the window
    // manager is used to close the window, not the close button in the web UI.
    info!("NWJS exited, send ExitApp to the server");

    let msg = serde_json::to_string(&Sending {
        data_type: MsgDataType::ExitApp,
        data: "".to_owned(),
    })
    .unwrap();

    match client_sender.send(msg) {
        Ok(_) => {}
        Err(e) => error!("🔥 Can't send on client_sender: {:?}", e),
    }

    nwjs_client_handle.join().unwrap();

    info!("🏁 start_nwjs() finish");
    Ok(())
}

pub fn start_preview(
    plazma_server_port: Arc<usize>,
    yml_path: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    info!("⚽ start_preview() start");

    // Channel to pass messages from the Websocket client to the OpenGL window.
    let (client_sender, client_receiver) = mpsc::channel();

    // Channel to pass messages from the OpenGL window to the Websocket client which will pass it
    // on to the server.
    let (server_sender, server_receiver) = mpsc::channel();

    // Channel to pass messages from the main app thread to the OpenGL window.
    let (app_sender, app_receiver) = mpsc::channel();

    // Start the Websocket client on a separate thread so that it is not blocked
    // (and is not blocking) the OpenGL window.

    let plazma_server_port_clone = Arc::clone(&plazma_server_port);

    let client_handle = thread::spawn(move || {
        info!("🧵 new thread: preview client");

        let sys = actix::System::new("preview client");

        // Start a WebSocket client and connect to the server.

        // FIXME check if server is up

        Arbiter::spawn(
            ws::Client::new(format! {"http://127.0.0.1:{}/ws/", plazma_server_port_clone})
                .connect()
                .map_err(|e| {
                    error!("🔥 ⚔️  Can not connect to server: {}", e);
                    // FIXME wait and keep trying to connect in a loop
                    //return; // this return is probably not necessary

                    thread::spawn(move || {
                        info!("🧵 new thread: app receiver");
                        loop {
                            if let Ok(text) = app_receiver.try_recv() {
                                if text == "StopSystem" {
                                    info!("🔎 app_receiver StopSystem: exiting");
                                    // FIXME stop the arbiter instead
                                    exit(0);
                                }
                            }
                            sleep(Duration::from_millis(100));
                        }
                    });
                })
                .map(|(reader, writer)| {
                    let addr = ClientActor::create(|ctx| {
                        ClientActor::add_stream(reader, ctx);
                        ClientActor {
                            writer,
                            channel_sender: client_sender,
                        }
                    });

                    thread::spawn(move || {
                        info!("🧵 new thread: server receiver from render_loop()");
                        loop {
                            if let Ok(text) = server_receiver.try_recv() {
                                // A bit noisy because of the frequent SetDmoTime messages.
                                //info!("Preview thread: passing on message: {}", text);
                                addr.do_send(ClientMessage { data: text });
                            }
                            sleep(Duration::from_millis(100));
                        }
                    });
                }),
        );

        sys.run();
    });

    // Tell the UI that preview is open. Allow the OpenGL window to open and the builtin demo to be
    // rendered for user to see the response. Then request the demo currently loaded on the server.

    let msg = serde_json::to_string(&Sending {
        data_type: MsgDataType::PreviewOpened,
        data: "".to_owned(),
    })
    .unwrap();
    match server_sender.send(msg) {
        Ok(_) => {}
        Err(e) => error!(
            "🔥 Can't send PreviewOpened on server_sender channel: {:?}",
            e
        ),
    };

    // Start OpenGL window on the main thread.

    // === Build the Window ===

    let start_fullscreen = false;

    let mut events_loop = glutin::EventsLoop::new();

    let monitor = events_loop
        .get_available_monitors()
        .nth(0)
        .expect("no monitor found");

    let window_builder = if start_fullscreen {
        glutin::WindowBuilder::new()
            .with_title("plazma preview")
            .with_fullscreen(Some(monitor))
    } else {
        glutin::WindowBuilder::new()
            .with_title("plazma preview")
            .with_dimensions(LogicalSize {
                width: 1366.0,
                height: 768.0,
            })
    };

    let context_builder = glutin::ContextBuilder::new();
    let window = glutin::GlWindow::new(window_builder, context_builder, &events_loop).unwrap();

    unsafe { window.make_current() }.unwrap();

    // load the OpenGL context
    gl::load_with(|ptr| window.context().get_proc_address(ptr) as *const _);

    // the polygon scenes need depth desting
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    // set the clear color at least once
    unsafe {
        gl::ClearColor(0.1, 0.2, 0.3, 1.0);
    }

    let logical_size = window.window().get_inner_size().unwrap();
    let dpi_factor = window.window().get_hidpi_factor();
    let physical_size = logical_size.to_physical(dpi_factor);
    // NOTE should we use LogicalSize instead, keep in mind the u32 truncates later
    let (wx, wy) = (physical_size.width, physical_size.height);

    let mut state = PreviewState::new(yml_path, wx, wy).unwrap();

    let mut rocket: Option<SyncClient> = None;
    state.build_rocket_connection(&mut rocket).unwrap();

    if rocket.is_some() {
        state.set_is_paused(true);
    } else {
        state.set_is_paused(false);
    }

    // TODO server_sender will error when server is not connected. Detect the condition and don't
    // send messages.

    // At this point the preview window is open. Request the demo which is currently loaded on the
    // server. The render loop will probably render a frame of the builtin demo before the response
    // is processed.

    let msg = serde_json::to_string(&Sending {
        data_type: MsgDataType::FetchDmoFile,
        data: "".to_owned(),
    })
    .unwrap();
    match server_sender.send(msg) {
        Ok(_) => info!("start_preview() Sent FetchDmoFile to server"),
        Err(e) => error!(
            "🔥 start_preview() Can't send FetchDmoFile on server_sender channel: {:?}",
            e
        ),
    };

    render_loop(
        &window,
        &mut events_loop,
        &mut state,
        &mut rocket,
        client_receiver,
        &server_sender,
    );

    // TODO app_sender will error when server is connected. Detect that condition and don't send
    // messages.

    match app_sender.send("StopSystem") {
        Ok(x) => x,
        Err(e) => {
            error!("🔥 Can't send on app_sender channel: {:?}", e);
        }
    }

    match client_handle.join() {
        Ok(_) => {}
        Err(e) => {
            error! {"{:?}", e};
        }
    };

    info!("🏁 start_preview() finish");

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
fn render_loop(
    window: &GlWindow,
    events_loop: &mut EventsLoop,
    state: &mut PreviewState,
    mut rocket: &mut Option<SyncClient>,
    client_receiver: mpsc::Receiver<String>,
    server_sender: &mpsc::Sender<String>,
) {
    info!("⚽ render_loop() start");

    let mut dpi_factor = window.window().get_hidpi_factor();

    while state.get_is_running() {
        state.draw_anyway = false;

        // 000. handle server messages

        match client_receiver.try_recv() {
            Ok(text) => {
                /*
                let n = if text.len() < 100 {
                    text.len()
                } else {
                    100
                };
                info!("render_loop() text message length {}, {}", text.len(), &text[0..n]);
                */

                if text == "StopSystem" {
                    info!("render_loop() Received StopSystem.");
                    state.set_is_running(false);
                } else {
                    let message: Receiving = match serde_json::from_str(&text) {
                        Ok(x) => x,
                        Err(e) => {
                            error! {"🔥 render_loop() Can't deserialize message: {:?}", e};
                            // Assign a NOOP instead of returning from the function.
                            Receiving {
                                data_type: NoOp,
                                data: "".to_string(),
                            }
                        }
                    };
                    //info!{"Received: message.data_type: {:?}", message.data_type};

                    use crate::server_actor::MsgDataType::*;
                    match message.data_type {
                        NoOp => {}

                        FetchDmoFile => {}
                        FetchDmoInline => {}

                        SetDmoInline => {
                            info!("render_loop() Received SetDmoInline");
                            let (sx, sy) = state.dmo_gfx.context.get_screen_resolution();
                            let (wx, wy) = state.dmo_gfx.context.get_window_resolution();
                            info! {"sx: {}, sy: {}", sx, sy};
                            let camera = state.dmo_gfx.context.camera.get_copy();

                            match serde_json::from_str::<SetDmoMsg>(&message.data) {
                                Ok(msg) => {
                                    // - don't read in shader files again,
                                    //   the updated shaders are sent directly from the UI
                                    // - do read in images,
                                    //   these are passed only by path from the UI
                                    match state.build_dmo_gfx_from_dmo_msg(
                                        &msg,
                                        false,
                                        true,
                                        sx,
                                        sy,
                                        Some(camera),
                                    ) {
                                        Ok(_) => {
                                            match state
                                                .callback_window_resized(wx as f64, wy as f64)
                                            {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    error!("🔥 callback_window_resized() {:?}", e)
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error! {"🔥 Can't perform SetDmoInline: {:?}", e}
                                        }
                                    }
                                }
                                Err(e) => error!("🔥 Can't deserialize SetDmoMsg: {:?}", e),
                            };
                        }

                        SetDmoFile => {
                            info!("render_loop() Received SetDmoFile");
                            let (sx, sy) = state.dmo_gfx.context.get_screen_resolution();
                            let (wx, wy) = state.dmo_gfx.context.get_window_resolution();
                            info! {"sx: {}, sy: {}", sx, sy};
                            let camera = state.dmo_gfx.context.camera.get_copy();

                            let path = serde_json::from_str::<PathBuf>(&message.data).unwrap();
                            let mut file = File::open(&path).unwrap();
                            let mut data = String::new();
                            file.read_to_string(&mut data).unwrap();

                            match fs::remove_file(&path) {
                                Ok(_) => {}
                                Err(e) => error! {"Can't remove file: {:?}", e},
                            };

                            match serde_json::from_str(&data) {
                                Ok(msg) => {
                                    // - don't read in shader files again,
                                    //   the updated shaders are sent directly from the UI
                                    // - do read in images,
                                    //   these are passed only by path from the UI
                                    match state.build_dmo_gfx_from_dmo_msg(
                                        &msg,
                                        false,
                                        true,
                                        sx,
                                        sy,
                                        Some(camera),
                                    ) {
                                        Ok(_) => {
                                            match state
                                                .callback_window_resized(wx as f64, wy as f64)
                                            {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    error!("🔥 callback_window_resized() {:?}", e)
                                                }
                                            }
                                        }
                                        Err(e) => error! {"🔥 Can't perform SetDmoFile: {:?}", e},
                                    }
                                }
                                Err(e) => error!("🔥 Can't deserialize SetDmoMsg: {:?}", e),
                            };
                        }

                        SetDmoTime => {
                            let time: f64 = match serde_json::from_str(&message.data) {
                                Ok(x) => x,
                                Err(e) => {
                                    error! {"🔥 Can't deserialize to time f64: {:?}", e};
                                    return;
                                }
                            };
                            state.set_time(time);
                        }

                        GetDmoTime => {
                            // When Rocket is not connected, send the server the time.
                            let msg = serde_json::to_string(&Sending {
                                data_type: MsgDataType::SetDmoTime,
                                data: format! {"{}", state.get_time()},
                            })
                            .unwrap();
                            if server_sender.send(msg).is_ok() {}
                        }

                        SetShader => {
                            let msg: SetShaderMsg = match serde_json::from_str(&message.data) {
                                Ok(x) => x,
                                Err(e) => {
                                    error! {"🔥 Can't deserialize to SetShaderMsg: {:?}", e};
                                    return;
                                }
                            };

                            match state.set_shader(msg.idx, &msg.content) {
                                Ok(_) => {
                                    info!("ShaderCompilationSuccess");
                                    state.draw_anyway = true;

                                    let data = ShaderCompilationSuccessMsg { idx: msg.idx };

                                    let msg = serde_json::to_string(&Sending {
                                        data_type: MsgDataType::ShaderCompilationSuccess,
                                        data: serde_json::to_string(&data).unwrap(),
                                    })
                                    .unwrap();
                                    match server_sender.send(msg) {
                                        Ok(_) => {},
                                        Err(e) => error!("🔥 Can't send ShaderCompilationSuccess on server_sender: {:?}", e),
                                    };
                                }
                                Err(e) => match e {
                                    ToolError::Runtime(ref e, ref error_msg) => {
                                        info!("{:?}, error message:\n{:#?}", e, error_msg);
                                        match e {
                                            RuntimeError::ShaderCompilationFailed => {
                                                let data = ShaderCompilationFailedMsg {
                                                    idx: msg.idx,
                                                    error_message: error_msg.clone(),
                                                };

                                                let msg = serde_json::to_string(&Sending {
                                                    data_type: MsgDataType::ShaderCompilationFailed,
                                                    data: serde_json::to_string(&data).unwrap(),
                                                })
                                                .unwrap();
                                                match server_sender.send(msg) {
                                                    Ok(_) => {},
                                                    Err(e) => error!("🔥 Can't send ShaderCompilationFailed on server_sender: {:?}", e),
                                                };
                                            }
                                            _ => error! {"🔥 Can't perform SetShader: {:?}", e},
                                        }
                                    }
                                    _ => error! {"🔥 Can't perform SetShader: {:?}", e},
                                },
                            };
                        }

                        SetSettings => {
                            let settings_data: crate::dmo_data::Settings =
                                match serde_json::from_str(&message.data) {
                                    Ok(x) => x,
                                    Err(e) => {
                                        error! {"🔥 Can't deserialize to Settings: {:?}", e};
                                        return;
                                    }
                                };

                            let settings = intro_runtime::dmo_gfx::Settings {
                                start_full_screen: settings_data.start_full_screen,
                                audio_play_on_start: settings_data.audio_play_on_start,
                                mouse_sensitivity: settings_data.mouse_sensitivity,
                                movement_sensitivity: settings_data.movement_sensitivity,
                                total_length: settings_data.total_length,
                            };
                            state.dmo_gfx.settings = settings;
                        }

                        SetMetadata => {}

                        ShowErrorMessage => {
                            error! {"🔥 Server is sending error: {:?}", message.data}
                        }

                        ShaderCompilationSuccess => {}
                        ShaderCompilationFailed => {}
                        StartPreview => {}

                        StopPreview => {
                            info!("render_loop() Received StopPreview.");
                            state.set_is_running(false);
                        }

                        PreviewOpened => {}
                        PreviewClosed => {}
                        StartDialogs => {}
                        StartWebview => {}
                        StartNwjs => {}
                        OpenProjectFileDialog => {}
                        OpenProjectFilePath => {}
                        ReloadProject => {}
                        SaveProject => {}
                        NewProject => {}
                        DeleteMessageFile => {}

                        ExitApp => {
                            info!("render_loop() Received ExitApp.");
                            state.set_is_running(false);
                        }
                    }
                }
            }

            Err(e) => match e {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => {}
                //_ => error!("render_loop() can't receive: {:?}", e),
            },
        }

        // 00. recompile if flag was set
        // FIXME process ShaderCompilationFailed
        match state.recompile_dmo() {
            Ok(_) => {}
            Err(e) => error! {"{:?}", e},
        }

        // 0. update time
        //
        // Note that frame time start is not the same as the time in the sync vars.

        state.update_time_frame_start();

        if state.dmo_gfx.settings.total_length < state.get_time() {
            break;
        }

        // 1. sync vars (time, camera, etc.)

        match state.update_rocket(&mut rocket) {
            Ok(_) => {}
            Err(e) => error!("🔥 state.update_rocket() returned: {:?}", e),
        }

        /* NOTE: this causes the frame update to stutter on Windows
        match state.connect_to_rocket(&mut rocket) {
            Ok(_) => {}
            Err(e) => error!("🔥 state.reconnect_to_rocket() returned: {:?}", e),
        }
        */

        match state.update_vars() {
            Ok(_) => {}
            Err(e) => error!("🔥 state.update_vars() returned: {:?}", e),
        }

        // In explore mode, override camera sync variables (calculated from the
        // sync tracks) with camera position (calculated from keys and mouse).

        if state.explore_mode {
            state.dmo_gfx.context.set_camera_sync();
        }

        // 2. deal with events

        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        state.set_is_running(false);
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        use glutin::VirtualKeyCode::*;

                        if let Some(vcode) = input.virtual_keycode {
                            let pressed = match input.state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            };

                            match vcode {
                                Escape => state.set_is_running(false),

                                // movement
                                W | A | S | D => {
                                    state.set_key_pressed(vcode, pressed);
                                }

                                // explore mode
                                X => {
                                    // act on key release
                                    if !pressed {
                                        state.explore_mode = !state.explore_mode;
                                    }

                                    // When turning on explore mode, copy the
                                    // Dmo Context camera values to State
                                    // camera.
                                    if state.explore_mode {
                                        state.set_camera_from_context();
                                    }
                                }

                                // pause time
                                Space => {
                                    if !pressed {
                                        // Only when Rocket is not on. Otherwise it controls paused state.
                                        if rocket.is_none() {
                                            state.toggle_paused();
                                        }
                                    }
                                }

                                // move time backwards 2s
                                Left => {
                                    if !pressed && rocket.is_none() {
                                        state.move_time_ms(-2000);
                                        match state.update_vars() {
                                            Ok(_) => {}
                                            Err(e) => error!("🔥 update_vars() {:?}", e),
                                        }
                                    }
                                }

                                // move time forward 2s
                                Right => {
                                    if !pressed && rocket.is_none() {
                                        state.move_time_ms(2000);
                                        match state.update_vars() {
                                            Ok(_) => {}
                                            Err(e) => error!("🔥 update_vars() {:?}", e),
                                        }
                                    }
                                }

                                // print camera values
                                C => {
                                    if pressed {
                                        println!("------------------------------");
                                        println!();
                                        println!("--- dmo_gfx.context.camera ---");
                                        let a: &Vector3 =
                                            state.dmo_gfx.context.camera.get_position();
                                        println!(".position: {}, {}, {}", a.x, a.y, a.z);
                                        let a: &Vector3 = state.dmo_gfx.context.camera.get_front();
                                        println!(".front: {}, {}, {}", a.x, a.y, a.z);
                                        println!(".pitch: {}", state.dmo_gfx.context.camera.pitch);
                                        println!(".yaw: {}", state.dmo_gfx.context.camera.yaw);
                                        println!();
                                        println!("--- dmo_gfx.context.polygon_context ---");
                                        let a: &Vector3 = state
                                            .dmo_gfx
                                            .context
                                            .polygon_context
                                            .get_view_position();
                                        println!(".view_position: {}, {}, {}", a.x, a.y, a.z);
                                        let a: &Vector3 =
                                            state.dmo_gfx.context.polygon_context.get_view_front();
                                        println!(".view_front: {}, {}, {}", a.x, a.y, a.z);
                                        println!();
                                    }
                                }

                                _ => (),
                            }
                        }
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        let (mx, my) = position.into();
                        state.callback_mouse_moved(mx, my);
                    }

                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        glutin::MouseScrollDelta::LineDelta(_, dy) => {
                            state.callback_mouse_wheel(dy);
                        }
                        glutin::MouseScrollDelta::PixelDelta(position) => {
                            let (_, dy): (f64, f64) = position.into();
                            state.callback_mouse_wheel(dy as f32);
                        }
                    },

                    WindowEvent::MouseInput {
                        state: pressed_state,
                        button,
                        ..
                    } => {
                        state.callback_mouse_input(pressed_state, button);
                    }

                    WindowEvent::CursorEntered { .. } => {}
                    WindowEvent::CursorLeft { .. } => {}

                    WindowEvent::HiDpiFactorChanged(dpi) => {
                        dpi_factor = dpi;
                    }

                    WindowEvent::Refresh => {
                        state.draw_anyway = true;
                    }

                    WindowEvent::Resized(logical_size) => {
                        info! {"WindowEvent::Resized"};

                        let physical_size = logical_size.to_physical(dpi_factor);
                        let (wx, wy) = (physical_size.width, physical_size.height);

                        match state.callback_window_resized(wx as f64, wy as f64) {
                            Ok(_) => {}
                            Err(e) => error!("🔥 callback_window_resized() {:?}", e),
                        }
                    }
                    _ => (),
                }
            }
        });

        // 3. move, update camera (only works in explore mode)

        state.update_camera_from_keys();

        // 4. rebuild when assets change on disk

        // TODO

        // 5. draw if we are not paused or should draw anyway (e.g. assets changed)

        match state.dmo_gfx.update_polygon_context() {
            Ok(_) => {}
            Err(e) => error!("update_polygon_context() returned: {:?}", e),
        };

        if !state.get_is_paused() || state.draw_anyway {
            state.draw();
        }

        state.update_time_frame_end();

        // ship the frame

        window.swap_buffers().unwrap();

        // 6. sleep if there is time left, or jump if the frame took too long

        state.t_delta = state.t_frame_start.elapsed();

        if state.t_delta < state.t_frame_target {
            if let Some(t_sleep) = state.t_frame_target.checked_sub(state.t_delta) {
                //info!("sleep: {}ms", t_sleep.subsec_nanos() / 1000 / 1000);
                sleep(t_sleep);
            }
        } else if state.t_delta > state.t_frame_target && !state.get_is_paused() {
            let a = state.get_t_frame_target_as_nanos();
            let b = state.get_t_delta_as_nanos() - a;

            // Remainder after last whole frame time in milliseconds.
            let rem_millis: u32 = ((b % a) / 1_000_000) as u32;

            // The delta frame time up to the last whole frame. The next
            // loop will increase the time with the target frame time.
            //
            // Should be 0 when adding the target frame time will be over
            // the current delta frame time, so the next draw will use a
            // time after the current delta, but at whole frame time
            // intervals.
            let c = ((b / 1_000_000) as u32) - rem_millis;

            let sync = state.get_sync_device_mut();

            sync.time += c;
            sync.set_row_from_time();
        }
    }

    let msg = serde_json::to_string(&Sending {
        data_type: MsgDataType::PreviewClosed,
        data: "".to_owned(),
    })
    .unwrap();
    match server_sender.send(msg) {
        Ok(_) => {}
        Err(e) => error!("🔥 Can't send PreviewClosed on server_sender: {:?}", e),
    };

    match server_sender.send("StopSystem".to_owned()) {
        Ok(_) => info!("render_loop() sent StopSystem"),
        Err(e) => error!(
            "🔥 render_loop() Can't send StopSystem on server_sender: {:?}",
            e
        ),
    };

    info!("🏁 render_loop() return");
}

pub fn start_dialogs(plazma_server_port: Arc<usize>) -> Result<(), Box<dyn Error>> {
    info!("⚽ start_dialogs() start");

    // Channel to pass messages from the Websocket client to the OpenGL window.
    let (client_sender, client_receiver) = mpsc::channel();

    // Channel to pass messages from the OpenGL window to the Websocket client which will pass it
    // on to the server.
    let (server_sender, server_receiver) = mpsc::channel();

    // Channel to pass messages from the main app thread to the OpenGL window.
    let (app_sender, app_receiver) = mpsc::channel();

    // Start the Websocket client on a separate thread so that it is not blocked
    // (and is not blocking) the OpenGL window.

    let plazma_server_port_clone = Arc::clone(&plazma_server_port);

    let client_handle = thread::spawn(move || {
        info!("🧵 new thread: dialogs client");

        let sys = actix::System::new("dialogs client");

        // Start a WebSocket client and connect to the server.

        // FIXME check if server is up

        Arbiter::spawn(
            ws::Client::new(format! {"http://127.0.0.1:{}/ws/", plazma_server_port_clone})
                .connect()
                .map_err(|e| {
                    error!("🔥 ⚔️  Can not connect to server: {}", e);
                    // FIXME wait and keep trying to connect in a loop
                    //return; // this return is probably not necessary

                    thread::spawn(move || {
                        info!("🧵 new thread: app receiver");
                        loop {
                            if let Ok(text) = app_receiver.try_recv() {
                                if text == "StopSystem" {
                                    info!("🔎 app_receiver StopSystem: exiting");
                                    // FIXME stop the arbiter instead
                                    exit(0);
                                }
                            }

                            sleep(Duration::from_millis(100));
                        }
                    });
                })
                .map(|(reader, writer)| {
                    let addr = ClientActor::create(|ctx| {
                        ClientActor::add_stream(reader, ctx);
                        ClientActor {
                            writer,
                            channel_sender: client_sender,
                        }
                    });

                    thread::spawn(move || {
                        info!("🧵 new thread: server receiver from dialogs_loop()");
                        loop {
                            if let Ok(text) = server_receiver.try_recv() {
                                info!("Dialogs thread: passing on message: {}", text);
                                addr.do_send(ClientMessage { data: text });
                            }
                            sleep(Duration::from_millis(100));
                        }
                    });
                }),
        );

        sys.run();
    });

    // Open dialogs on the main thread.

    dialogs_loop(client_receiver, &server_sender);

    // TODO app_sender will error when server is connected. Detect that condition and don't send
    // messages.

    match app_sender.send("StopSystem") {
        Ok(x) => x,
        Err(e) => {
            error!("🔥 Can't send on app_sender channel: {:?}", e);
        }
    }

    match client_handle.join() {
        Ok(_) => {}
        Err(e) => {
            error! {"{:?}", e};
        }
    };

    info!("🏁 start_dialogs() finish");

    Ok(())
}

fn dialogs_loop(client_receiver: mpsc::Receiver<String>, server_sender: &mpsc::Sender<String>) {
    info!("⚽ dialogs_loop() start");
    let mut is_running = true;

    while is_running {
        match client_receiver.try_recv() {
            Ok(text) => {
                if text == "StopSystem" {
                    info!("render_loop() Received StopSystem.");
                    is_running = false;
                } else {
                    let message: Receiving = match serde_json::from_str(&text) {
                        Ok(x) => x,
                        Err(e) => {
                            error! {"🔥 dialogs_loop() Can't deserialize message: {:?}", e};
                            // Assign a NOOP instead of returning from the function.
                            Receiving {
                                data_type: NoOp,
                                data: "".to_string(),
                            }
                        }
                    };

                    use crate::server_actor::MsgDataType::*;
                    match message.data_type {
                        NoOp => {}
                        FetchDmoFile => {}
                        FetchDmoInline => {}
                        SetDmoInline => {}
                        SetDmoFile => {
                            let path = serde_json::from_str::<PathBuf>(&message.data).unwrap();
                            match fs::remove_file(&path) {
                                Ok(_) => {}
                                Err(e) => error! {"Can't remove file: {:?}", e},
                            };
                        }
                        SetDmoTime => {}
                        GetDmoTime => {}
                        SetShader => {}
                        SetSettings => {}
                        SetMetadata => {}
                        ShowErrorMessage => {}
                        ShaderCompilationSuccess => {}
                        ShaderCompilationFailed => {}
                        StartPreview => {}
                        StopPreview => {}
                        PreviewOpened => {}
                        PreviewClosed => {}
                        StartDialogs => {}
                        StartWebview => {}
                        StartNwjs => {}

                        OpenProjectFileDialog => {
                            let res = nfd::open_file_dialog(None, None)
                                .expect("Failed to open a native file dialog.");

                            let mut path = String::new();
                            match res {
                                NfdResponse::Okay(p) => path = p,

                                NfdResponse::OkayMultiple(files) => path = files[0].to_string(),

                                NfdResponse::Cancel => {}
                            }

                            if !path.is_empty() {
                                let msg = serde_json::to_string(&Sending {
                                    data_type: MsgDataType::OpenProjectFilePath,
                                    data: serde_json::to_string(&path).unwrap(),
                                })
                                .unwrap();
                                match server_sender.send(msg) {
                                    Ok(_) => {}
                                    Err(e) => error!(
                                        "🔥 Can't send OpenProjectFilePath on server_sender: {:?}",
                                        e
                                    ),
                                };
                            }
                        }

                        OpenProjectFilePath => {}
                        ReloadProject => {}
                        SaveProject => {}
                        NewProject => {}
                        DeleteMessageFile => {}

                        ExitApp => {
                            info!("dialogs_loop() Received ExitApp.");
                            is_running = false;
                        }
                    }
                }
            }

            Err(e) => match e {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => {}
                //_ => error!("dialogs_loop() can't receive: {:?}", e),
            },
        }

        sleep(Duration::from_millis(100));
    }

    match server_sender.send("StopSystem".to_owned()) {
        Ok(_) => info!("dialogs_loop() sent StopSystem"),
        Err(e) => error!(
            "🔥 dialogs_loop() Can't send StopSystem on server_sender: {:?}",
            e
        ),
    };

    info!("🏁 dialogs_loop() return");
}
