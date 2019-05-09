use std::env;
use std::process::exit;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, sleep};
use std::time::Duration;
use std::path::PathBuf;
use std::error::Error;

use web_view::Content;

use actix_web::{fs, middleware, server, ws, App, HttpRequest};
use actix_web::Error as AxError;
use actix_web::actix::*;

use futures::Future;

use glutin::{GlWindow, GlContext, EventsLoop, Event, WindowEvent, ElementState};

use intro_3d::Vector3;
use rocket_client::SyncClient;

use crate::server_actor::{ServerActor, ServerState, ServerStateWrap, Sending, Receiving, MsgDataType};
use crate::preview_client::client_actor::{ClientActor, ClientMessage};

use crate::preview_client::preview_state::PreviewState;

pub fn handle_static_index(_req: &HttpRequest<ServerStateWrap>) -> Result<fs::NamedFile, AxError> {
    Ok(fs::NamedFile::open("../gui/build/index.html")?)
}

#[derive(Debug)]
pub struct AppStartParams {
    pub yml_path: Option<PathBuf>,
    pub dmo_path: Option<PathBuf>,
    pub plazma_server_port: Arc<usize>,
    pub start_server: bool,
    pub start_webview: bool,
    pub start_preview: bool,
}

pub struct AppInfo {
    pub cwd: PathBuf,
    pub path_to_binary: PathBuf,
}

impl Default for AppStartParams {
    fn default() -> AppStartParams {
        AppStartParams {
            yml_path: None,
            dmo_path: None,
            plazma_server_port: Arc::new(8080),
            start_server: true,
            start_webview: true,
            start_preview: false,
        }
    }
}

impl AppStartParams {
    fn default_with_port(port: usize) -> AppStartParams {
        let mut params = AppStartParams::default();
        params.plazma_server_port = Arc::new(port);
        params
    }
}

pub fn app_info() -> Result<AppInfo, Box<Error>>
{
    let cwd = PathBuf::from(std::env::current_dir()?).canonicalize()?;
    let mut path_to_binary = PathBuf::from(cwd.clone());

    if let Some(a) = std::env::args().nth(0) {
        path_to_binary = path_to_binary.join(PathBuf::from(a));
    } else {
        if cfg!(target_os = "windows") {
            path_to_binary = path_to_binary.join(PathBuf::from("plazma.exe".to_owned()));
        } else {
            path_to_binary = path_to_binary.join(PathBuf::from("plazma".to_owned()));
        }
    }
    path_to_binary = path_to_binary.canonicalize()?;

    if !path_to_binary.exists() {
        return Err(From::from(format!("🔥 Path does not exist: {:?}", &path_to_binary)));
    }

    Ok(AppInfo{
        cwd: cwd,
        path_to_binary: path_to_binary,
    })
}

pub fn process_cli_args(matches: clap::ArgMatches)
    -> Result<AppStartParams, Box<Error>>
{
    let server_port = match matches.value_of("port").unwrap().parse::<usize>() {
        Ok(x) => x,
        Err(e) => {
            error!{"🔥 {:?}", e};
            exit(2);
        }
    };

    let mut params = AppStartParams::default_with_port(server_port);

    if matches.is_present("yml") {
        params.yml_path = match matches.value_of("yml").unwrap().parse::<String>() {
            Ok(x) => {
                let path = PathBuf::from(&x);
                if path.exists() {
                    Some(path)
                } else {
                    error!("🔥 Path does not exist: {:?}", &path);
                    exit(2);
                }
            },
            Err(e) => {
                error!{"🔥 {:?}", e};
                exit(2);
            }
        };
    }

    if matches.is_present("dmo") {
        params.dmo_path = match matches.value_of("dmo").unwrap().parse::<String>() {
            Ok(x) => {
                let path = PathBuf::from(&x);
                if path.exists() {
                    Some(path)
                } else {
                    error!("🔥 Path does not exist: {:?}", &path);
                    exit(2);
                }
            },
            Err(e) => {
                error!{"🔥 {:?}", e};
                exit(2);
            }
        };
    }

    if let Some(_) = matches.subcommand_matches("server") {

        params.start_server = true;
        params.start_webview = false;
        params.start_preview = false;

    } else if let Some(_) = matches.subcommand_matches("preview") {

        params.start_server = false;
        params.start_webview = false;
        params.start_preview = true;

    };

    Ok(params)
}

pub fn start_server(port: Arc<usize>,
                    app_info: AppInfo,
                    yml_path: Option<PathBuf>,
                    webview_sender_arc: Arc<Mutex<mpsc::Sender<String>>>,
                    server_receiver: mpsc::Receiver<String>)
    -> Result<(thread::JoinHandle<()>, thread::JoinHandle<()>, thread::JoinHandle<()>), Box<Error>>
{
    let port_clone_a = Arc::clone(&port);
    let port_clone_b = Arc::clone(&port);

    let a = webview_sender_arc.clone();
    let server_handle = thread::spawn(move || {
        info!("🧵 new thread: server");
        let sys = actix::System::new("plazma server");

        info!("ServerState::new() using yml_path: {:?}", &yml_path);

        let server_state = Arc::new(
            Mutex::new(
                ServerState::new(app_info,
                                 a,
                                 yml_path).unwrap()
                )
            );

        server::new(move || {

            App::with_state(server_state.clone())
            // logger
                .middleware(middleware::Logger::default())
            // WebSocket routes (there is no CORS)
                .resource("/ws/", |r| r.f(|req| ws::start(req, ServerActor::new())))
            // static files
                .handler("/static/", fs::StaticFiles::new("../gui/build/").unwrap()
                         .default_handler(handle_static_index))
        })
            .bind(format!{"127.0.0.1:{}", port_clone_a})
            .unwrap()
            .start();

        sys.run();
    });

    // Start a WebSocket client which can pass messages from the server_receiver channel to the
    // server actor.

    let (client_sender, client_receiver) = mpsc::channel();

    let server_receiver_handle = thread::spawn(move || {
        info!("🧵 new thread: server receiver client");

        let sys = actix::System::new("server receiver client");

        // Start a WebSocket client and connect to the server.

        // FIXME check if server is up. For now, just wait a bit.
        sleep(Duration::from_millis(5000));

        Arbiter::spawn(
            ws::Client::new(format!{"http://127.0.0.1:{}/ws/", port_clone_b})
                .connect()

                .map_err(|e| {
                    error!("🔥 ⚔️  Can not connect to server: {}", e);
                    // FIXME wait and keep trying to connect in a loop
                    //return; // this return is probably not necessary
                    ()
                })

                .map(|(reader, writer)| {
                    let addr = ClientActor::create(|ctx| {
                        ClientActor::add_stream(reader, ctx);
                        ClientActor{
                            writer: writer,
                            channel_sender: client_sender,
                        }
                    });

                    thread::spawn(move || {
                        info!("🧵 new thread: server receiver from webview");
                        loop {
                            match server_receiver.try_recv() {
                                Ok(text) => {
                                    info!("Passing on webview message: {:?}", text);
                                    addr.do_send(ClientMessage{ data: text });
                                },
                                Err(_) => {},
                            }
                            sleep(Duration::from_millis(100));
                        }
                    });

                    ()
                }),
        );

        sys.run();
    });

    let a = webview_sender_arc.clone();
    let client_receiver_handle = thread::spawn(move || {
        info!("🧵 new thread: client receiver");
        loop {
            match client_receiver.try_recv() {
                Ok(text) => {
                    if text == "StopSystem" {
                        info!("client_receiver: {:?}", text);

                        let webview_sender = a.lock().expect("Can't lock webview sender.");
                        match webview_sender.send("ExitWebview".to_owned()) {
                            Ok(_) => {},
                            Err(e) => error!("Can't send on webview_sender: {:?}", e),
                        };

                        break;
                    }
                },
                Err(_) => {},
            }
            sleep(Duration::from_millis(100));
        }
    });

    Ok((server_handle, server_receiver_handle, client_receiver_handle))
}

pub fn start_webview(plazma_server_port: Arc<usize>,
                     webview_receiver: mpsc::Receiver<String>,
                     server_sender_arc: Arc<Mutex<mpsc::Sender<String>>>)
    -> Result<(), Box<Error>>
{
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

    // If the React dev server is running, load content from there. If not, load
    // our static files route which is serving the React build directory.
    let content_url = if let Some(port) = react_server_port {
        format!{"http://localhost:{}/static/", port}
    } else {
        format!{"http://localhost:{}/static/", plazma_server_port}
    };

    struct UserData {
        webview_receiver: mpsc::Receiver<String>,
    };

    {
        let webview = web_view::builder()
            .title("Plazma")
            .content(Content::Url(content_url))
            .size(1366, 768)
            .resizable(true)
            .debug(true)
            .user_data(UserData {
                webview_receiver: webview_receiver,
            })
            .invoke_handler(|_webview, _arg| Ok(()))
        .build().unwrap();

        let webview_handle = webview.handle();

        // When the application window is reloaded (such as when the user is requesting a reload,
        // or when the React dev server recompiles and reloads after a code change), the
        // window.external object is lost and the invoke_handler() above is not accessible.
        //
        // Instead, we will receive messages such as ExitWebview or FileOpen via a channel. A
        // message is first sent to the server over the WebSocket connection, then the server
        // handles that and puts a message on this channel, which we receive here.

        thread::spawn(move || loop {
            {

                let a = server_sender_arc.clone();
                let res = webview_handle.dispatch(move |webview| {

                    let server_sender = a.lock().expect("Can't lock server sender.");

                    let UserData {
                        webview_receiver,
                    } = webview.user_data();

                    match webview_receiver.try_recv() {
                        Ok(text) => {
                            match text.as_ref() {
                                "FileOpen" => {
                                    match webview.dialog().open_file("Please choose a file...", "")?  {
                                        Some(path) => webview.dialog().info("File chosen", path.to_string_lossy()),
                                        None => webview
                                            .dialog()
                                            .warning("Warning", "You didn't choose a file."),
                                    }?;

                                    // FIXME send server the path to use
                                    match &server_sender.send("data_type, data...".to_owned()) {
                                        Ok(_) => {},
                                        Err(e) => error!("🔥 Can't send on server_sender: {:?}", e),
                                    };
                                },

                                "ExitWebview" =>
                                {
                                    info!("💬 webview dispatch: ExitWebview received from server.");
                                    info!("Terminating the webview.");
                                    webview.terminate();
                                }
                                _ => {
                                    // unimplemented!();
                                },
                            };
                        },
                        Err(_) => {},
                    }

                    Ok(())
                });

                match res {
                    Ok(_) => {},
                    Err(e) => error!("🔥 webview_handle.dispatch() {:?}", e),
                }
            }

            sleep(Duration::from_millis(1000));
        });

        // This will block until the window is closed.
        webview.run()?;

        // Actor and System are stopped in server_actor.rs when handling ExitApp message.

    }

    Ok(())
}

pub fn start_preview(plazma_server_port: Arc<usize>,
                     yml_path: Option<PathBuf>)
    -> Result<(), Box<Error>>
{
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
            ws::Client::new(format!{"http://127.0.0.1:{}/ws/", plazma_server_port_clone})
                .connect()

                .map_err(|e| {
                    error!("🔥 ⚔️  Can not connect to server: {}", e);
                    // FIXME wait and keep trying to connect in a loop
                    //return; // this return is probably not necessary

                    thread::spawn(move || {
                        info!("🧵 new thread: app receiver");
                        loop {
                            match app_receiver.try_recv() {
                                Ok(text) => {
                                    if text == "StopSystem" {
                                        info!("🔎 app_receiver StopSystem: exiting");
                                        // FIXME stop the arbiter instead
                                        exit(0);
                                    }
                                },
                                Err(_) => {},
                            }

                            sleep(Duration::from_millis(100));
                        }
                    });

                    ()
                })

                .map(|(reader, writer)| {
                    let addr = ClientActor::create(|ctx| {
                        ClientActor::add_stream(reader, ctx);
                        ClientActor{
                            writer: writer,
                            channel_sender: client_sender,
                        }
                    });

                    thread::spawn(move || {
                        info!("🧵 new thread: server receiver from render_loop()");
                        loop {
                            match server_receiver.try_recv() {
                                Ok(text) => {
                                    addr.do_send(ClientMessage{ data: text });
                                },
                                Err(_) => {},
                            }
                            sleep(Duration::from_millis(100));
                        }
                    });

                    ()
                }),
        );

        sys.run();
    });

    let msg = serde_json::to_string(&Sending{
        data_type: MsgDataType::PreviewOpened,
        data: "".to_owned(),
    }).unwrap();
    match server_sender.send(msg) {
        Ok(_) => {},
        Err(e) => error!("🔥 Can't send PreviewOpened on server_sender channel: {:?}", e),
    };

    // Start OpenGL window on the main thread.

    // === Build the Window ===

    let start_fullscreen = false;

    let mut events_loop = glutin::EventsLoop::new();

    let monitor = events_loop.get_available_monitors().nth(0).expect("no monitor found");

    let window_builder = if start_fullscreen {
        glutin::WindowBuilder::new()
            .with_title("plazma preview")
            .with_fullscreen(Some(monitor))
    } else {
        glutin::WindowBuilder::new()
            .with_title("plazma preview")
    };

    let context_builder = glutin::ContextBuilder::new();
    let window = glutin::GlWindow::new(window_builder, context_builder, &events_loop).unwrap();

    unsafe { window.make_current() }.unwrap();

    // load the OpenGL context
    gl::load_with(|ptr| window.context().get_proc_address(ptr) as *const _);

    // the polygon scenes need depth desting
    unsafe { gl::Enable(gl::DEPTH_TEST); }

    // set the clear color at least once
    unsafe { gl::ClearColor(0.1, 0.2, 0.3, 1.0); }

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

    render_loop(&window, &mut events_loop, &mut state, &mut rocket, client_receiver, &server_sender);

    // TODO app_sender will error when server is connected. Detect that condition and don't send
    // messages.

    match app_sender.send("StopSystem") {
        Ok(x) => x,
        Err(e) => {
            error!("🔥 Can't send on app_sender channel: {:?}", e);
        }
    }

    match client_handle.join() {
        Ok(_) => {},
        Err(e) => {
            error!{"{:?}", e};
        }
    };

    info!("🏁 start_preview() finish");

    Ok(())
}

fn render_loop(window: &GlWindow,
               events_loop: &mut EventsLoop,
               state: &mut PreviewState,
               mut rocket: &mut Option<SyncClient>,
               client_receiver: mpsc::Receiver<String>,
               server_sender: &mpsc::Sender<String>)
    -> ()
{
    info!("⚽ render_loop() start");

    let mut dpi_factor = window.window().get_hidpi_factor();

    while state.get_is_running() {

        state.draw_anyway = false;

        // 000. handle server messages

        match client_receiver.try_recv() {
            Ok(text) => {
                // FIXME return a NOOP otherwise it returns from the function.
                let message: Receiving = match serde_json::from_str(&text) {
                    Ok(x) => x,
                    Err(e) => {
                        error!{"🔥 Can't deserialize message: {:?}", e};
                        return;
                    },
                };
                //info!{"Received: message.data_type: {:?}", message.data_type};

                use crate::server_actor::MsgDataType::*;
                match message.data_type {

                    NoOp => {},

                    FetchDmo => {},

                    SetDmo => {
                        let (sx, sy) = state.dmo_gfx.context.get_screen_resolution();
                        info!{"sx: {}, sy: {}", sx, sy};
                        let camera = state.dmo_gfx.context.camera.get_copy();

                        // - don't read in shader files again, the updated shaders are sent directly from the UI
                        // - do read in images, these are passed only by path from the UI
                        match state.build_dmo_gfx_from_yml_str(&message.data, false, true,
                                                               sx, sy,
                                                               Some(camera))
                        {
                            Ok(_) => {},
                            Err(e) => error!{"🔥 Can't perform SetDmo: {:?}", e},
                        }
                    },

                    SetDmoTime => {
                        let time: f64 = match serde_json::from_str(&message.data) {
                            Ok(x) => x,
                            Err(e) => {
                                error!{"🔥 Can't deserialize to time f64: {:?}", e};
                                return;
                            },
                        };
                        state.set_time(time);
                    },

                    GetDmoTime => {
                        // When Rocket is not connected, send the server the time.
                        let msg = serde_json::to_string(&Sending{
                            data_type: MsgDataType::SetDmoTime,
                            data: format!{"{}", state.get_time()},
                        }).unwrap();
                        match server_sender.send(msg) {
                            Ok(_) => {},
                            Err(_) => {},
                        };
                    },

                    SetSettings => {
                        let settings_data: crate::dmo_data::Settings = match serde_json::from_str(&message.data) {
                            Ok(x) => x,
                            Err(e) => {
                                error!{"🔥 Can't deserialize to Settings: {:?}", e};
                                return;
                            },
                        };

                        let settings = intro_runtime::dmo_gfx::Settings {
                            start_full_screen: settings_data.start_full_screen,
                            audio_play_on_start: settings_data.audio_play_on_start,
                            mouse_sensitivity: settings_data.mouse_sensitivity,
                            movement_sensitivity: settings_data.movement_sensitivity,
                            total_length: settings_data.total_length,
                        };
                        state.dmo_gfx.settings = settings;
                    },

                    ShowErrorMessage =>
                        error!{"🔥 Server is sending error: {:?}", message.data},

                    StartPreview => {},

                    StopPreview => {
                        info!("Received StopPreview.");
                        state.set_is_running(false);
                    },

                    PreviewOpened => {},
                    PreviewClosed => {},

                    ExitApp => {
                        info!("Received ExitApp.");
                        state.set_is_running(false);
                    },
                }

            },

            // Silently drop the error when there is no message to receive.
            Err(_) => {},
        }

        // 00. recompile if flag was set
        match state.recompile_dmo() {
            Ok(_) => {},
            Err(e) => error!{"{:?}", e},
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
            Ok(_) => {},
            Err(e) => error!("🔥 state.update_rocket() returned: {:?}", e),
        }

        match state.update_vars() {
            Ok(_) => {},
            Err(e) => error!("🔥 state.update_vars() returned: {:?}", e),
        }

        // In explore mode, override camera sync variables (calculated from the
        // sync tracks) with camera position (calculated from keys and mouse).

        if state.explore_mode {
            state.dmo_gfx.context.set_camera_sync();
        }

        // 2. deal with events

        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        state.set_is_running(false);
                    },

                    WindowEvent::KeyboardInput{ device_id: _, input } => {
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
                                },

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
                                },

                                // pause time
                                Space => if !pressed {
                                    // Only when Rocket is not on. Otherwise it controls paused state.
                                    if rocket.is_none() { state.toggle_paused(); }
                                },

                                // move time backwards 2s
                                Left => if !pressed {
                                    if rocket.is_none() {
                                        state.move_time_ms(-2000);
                                        match state.update_vars() {
                                            Ok(_) => {},
                                            Err(e) => error!("🔥 update_vars() {:?}", e),
                                        }
                                    }
                                },

                                // move time forward 2s
                                Right => if !pressed {
                                    if rocket.is_none() {
                                        state.move_time_ms(2000);
                                        match state.update_vars() {
                                            Ok(_) => {},
                                            Err(e) => error!("🔥 update_vars() {:?}", e),
                                        }
                                    }
                                },

                                // print camera values
                                C => if pressed {
                                    println!("------------------------------");
                                    println!("");
                                    println!("--- dmo_gfx.context.camera ---");
                                    let a: &Vector3 = state.dmo_gfx.context.camera.get_position();
                                    println!(".position: {}, {}, {}", a.x, a.y, a.z);
                                    let a: &Vector3 = state.dmo_gfx.context.camera.get_front();
                                    println!(".front: {}, {}, {}", a.x, a.y, a.z);
                                    println!(".pitch: {}", state.dmo_gfx.context.camera.pitch);
                                    println!(".yaw: {}", state.dmo_gfx.context.camera.yaw);
                                    println!("");
                                    println!("--- dmo_gfx.context.polygon_context ---");
                                    let a: &Vector3 = state.dmo_gfx.context.polygon_context.get_view_position();
                                    println!(".view_position: {}, {}, {}", a.x, a.y, a.z);
                                    let a: &Vector3 = state.dmo_gfx.context.polygon_context.get_view_front();
                                    println!(".view_front: {}, {}, {}", a.x, a.y, a.z);
                                    println!("");
                                }

                                _ => (),
                            }
                        }
                    },

                    WindowEvent::CursorMoved{ device_id: _, position, modifiers: _ } => {
                        let (mx, my) = position.into();
                        state.callback_mouse_moved(mx, my);
                    },

                    WindowEvent::MouseWheel{ device_id: _, delta, phase: _, modifiers: _ } => {
                        match delta {
                            glutin::MouseScrollDelta::LineDelta(_, dy) => {
                                state.callback_mouse_wheel(dy);
                            },
                            glutin::MouseScrollDelta::PixelDelta(position) => {
                                let (_, dy): (f64, f64) = position.into();
                                state.callback_mouse_wheel(dy as f32);
                            }
                        }
                    },

                    WindowEvent::MouseInput{ device_id: _, state: pressed_state, button, modifiers: _ } => {
                        state.callback_mouse_input(pressed_state, button);
                    },

                    WindowEvent::CursorEntered{ device_id: _ } => {},
                    WindowEvent::CursorLeft{ device_id: _ } => {},

                    WindowEvent::HiDpiFactorChanged(dpi) => {
                        dpi_factor = dpi;
                    }

                    WindowEvent::Refresh => {
                        state.draw_anyway = true;
                    },

                    WindowEvent::Resized(logical_size) => {
                        info!{"WindowEvent::Resized"};

                        let physical_size = logical_size.to_physical(dpi_factor);
                        let (wx, wy) = (physical_size.width, physical_size.height);

                        match state.callback_window_resized(wx as f64, wy as f64) {
                            Ok(_) => {},
                            Err(e) => error!("🔥 callback_window_resized() {:?}", e),
                        }
                    },
                    _ => (),
                },
                _ => (),
            }
        });

        // 3. move, update camera (only works in explore mode)

        state.update_camera_from_keys();

        // 4. rebuild when assets change on disk

        // TODO

        // 5. draw if we are not paused or should draw anyway (e.g. assets changed)

        match state.dmo_gfx.update_polygon_context() {
            Ok(_) => {},
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
            if let Some(t_sleep) = state.t_frame_target.checked_sub(state.t_delta)  {
                //info!("sleep: {}ms", t_sleep.subsec_nanos() / 1000 / 1000);
                sleep(t_sleep);
            }
        } else if state.t_delta > state.t_frame_target {
            if !state.get_is_paused() {
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
    }

    let msg = serde_json::to_string(&Sending{
        data_type: MsgDataType::PreviewClosed,
        data: "".to_owned(),
    }).unwrap();
    match server_sender.send(msg) {
        Ok(_) => {},
        Err(e) => error!("🔥 Can't send PreviewClosed on server_sender: {:?}", e),
    };

    match server_sender.send("StopSystem".to_owned()) {
        Ok(_) => {},
        Err(e) => {
            error!("🔥 Can't send StopSystem on server_sender: {:?}", e);
        },
    };

    info!("🏁 render_loop() return");
}
