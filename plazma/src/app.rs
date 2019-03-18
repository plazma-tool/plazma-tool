use std::env;
use std::process::exit;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, sleep};
use std::time::Duration;
use std::path::PathBuf;
use std::error::Error;

use web_view::Content;

use actix_web::{fs, middleware, server, client, ws, App, HttpRequest, HttpResponse};
use actix_web::Error as AxError;
use actix_web::actix::*;

use futures::Future;

use glutin::{GlWindow, GlContext, EventsLoop, Event, WindowEvent, ElementState};

use intro_3d::Vector3;
use rocket_client::SyncClient;

use crate::server_actor::{ServerActor, ServerState, ServerStateWrap, Sending, Receiving, MsgDataType};
use crate::preview_client::client_actor::{ClientActor, ClientMessage};

use crate::preview_client::preview_state::PreviewState;
use crate::utils::file_to_string;

pub fn handle_static_index(_req: &HttpRequest<ServerStateWrap>) -> Result<fs::NamedFile, AxError> {
    Ok(fs::NamedFile::open("../gui/build/index.html")?)
}

pub fn handle_stop_server(_req: &HttpRequest<ServerStateWrap>) -> Result<HttpResponse, AxError> {
    System::current().stop();
    Ok(HttpResponse::Ok()
       .content_type("text/plain")
       .body("g2g"))
}

#[derive(Debug)]
pub struct AppStartParams {
    pub yml_path: PathBuf,
    pub dmo_path: Option<PathBuf>,
    pub plazma_server_port: Arc<usize>,
    pub start_server: bool,
    pub start_webview: bool,
    pub start_preview: bool,
}

pub fn process_cli_args(matches: clap::ArgMatches)
    -> Result<AppStartParams, Box<Error>>
{
    let server_port = match matches.value_of("port").unwrap().parse::<usize>() {
        Ok(x) => x,
        Err(e) => {
            error!{"{:?}", e};
            exit(2);
        }
    };

    // Start with a minimal demo until we receive update from the server.
    let minimal_demo_yml_path = PathBuf::from("data".to_owned())
        .join(PathBuf::from("minimal".to_owned()))
        .join(PathBuf::from("demo.yml".to_owned()));

    let mut params = AppStartParams {
        yml_path: minimal_demo_yml_path,
        dmo_path: None,
        plazma_server_port: Arc::new(server_port),
        start_server: true,
        start_webview: true,
        start_preview: false,// FIXME fix blocking and start preview window as well
    };

    if let Some(_) = matches.subcommand_matches("server") {

        params.start_server = true;
        params.start_webview = false;
        params.start_preview = false;

    } else if let Some(m) = matches.subcommand_matches("preview") {

        params.start_server = false;
        params.start_webview = false;
        params.start_preview = true;

        if m.is_present("yml") {

            let path = PathBuf::from(m.value_of("yml").unwrap());
            if path.exists() {
                params.yml_path = path;
            } else {
                error!("Path does not exist: {:?}", &path);
                exit(2);
            }

        } else if m.is_present("dmo") {

            let path = PathBuf::from(m.value_of("dmo").unwrap());
            if path.exists() {
                params.dmo_path = Some(path);
            } else {
                error!("Path does not exist: {:?}", &path);
                exit(2);
            }

        }

    };

    Ok(params)
}

pub fn start_server(port: &Arc<usize>, yml_path: PathBuf)
    -> Result<thread::JoinHandle<()>, Box<Error>>
{
    let port_clone = Arc::clone(&port);

    let server_handle = thread::spawn(move || {

        let sys = actix::System::new("plazma server");

        let server_state = Arc::new(Mutex::new(ServerState::new(&yml_path).unwrap()));

        server::new(move || {

            App::with_state(server_state.clone())
            // logger
                .middleware(middleware::Logger::default())
            // WebSocket routes (there is no CORS)
                .resource("/ws/", |r| r.f(|req| ws::start(req, ServerActor::new())))
            // tell the server to stop
                .resource("/stop_server",
                          |r| r.get().f(handle_stop_server))
            // static files
                .handler("/static/", fs::StaticFiles::new("../gui/build/").unwrap()
                         .default_handler(handle_static_index))
        })
            .bind(format!{"127.0.0.1:{}", port_clone})
            .unwrap()
            .start();

        sys.run();
    });

    Ok(server_handle)
}

pub fn start_webview(plazma_server_port: &Arc<usize>)
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

    Ok(())
}

pub fn start_preview(plazma_server_port: &Arc<usize>)
    -> Result<thread::JoinHandle<()>, Box<Error>>
{
    // Channel to pass messages from the Websocket client to the OpenGL window.
    let (client_sender, client_receiver) = mpsc::channel();

    // Channel to pass messages from the OpenGL window to the Websocket client which will pass it
    // on to the server.
    let (server_sender, server_receiver) = mpsc::channel();

    // Start the Websocket client on a separate thread so that it is not blocked
    // (and is not blocking) the OpenGL window.

    let plazma_server_port_a = Arc::clone(&plazma_server_port);

    let client_handle = thread::spawn(move || {

        let sys = actix::System::new("preview client");

        // Start a WebSocket client and connect to the server.

        // FIXME check if server is up

        Arbiter::spawn(
            ws::Client::new(format!{"http://127.0.0.1:{}/ws/", plazma_server_port_a})
                .connect()
                .map_err(|e| {
                    error!("Can not connect to server: {}", e);
                    // FIXME wait and keep trying to connect in a loop
                    return;
                })
                .map(|(reader, writer)| {
                    let addr = ClientActor::create(|ctx| {
                        ClientActor::add_stream(reader, ctx);
                        ClientActor{
                            writer: writer,
                            channel_sender: client_sender,
                        }
                    });

                    // FIXME ? maybe don't need the new thread

                    thread::spawn(move || {
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

    let mut state = PreviewState::new(wx, wy).unwrap();

    // Start with a minimal demo until we receive update from the server.
    let demo_yml_path = PathBuf::from("data".to_owned())
        .join(PathBuf::from("minimal".to_owned()))
        .join(PathBuf::from("demo.yml".to_owned()));

    let text: String = file_to_string(&demo_yml_path).unwrap();

    // NOTE Must use window size for screen size as well
    state.build_dmo_gfx_from_yml_str(&text, true, true, wx, wy, wx, wy, None).unwrap();

    let mut rocket: Option<SyncClient> = None;
    state.build_rocket_connection(&mut rocket).unwrap();

    if rocket.is_some() {
        state.set_is_paused(true);
    } else {
        state.set_is_paused(false);
    }

    render_loop(&window, &mut events_loop, &mut state, &mut rocket, client_receiver, server_sender);

    info!("Render loop exited.");

    Ok(client_handle)
}

fn render_loop(window: &GlWindow,
               events_loop: &mut EventsLoop,
               state: &mut PreviewState,
               mut rocket: &mut Option<SyncClient>,
               client_receiver: mpsc::Receiver<String>,
               server_sender: mpsc::Sender<String>)
    -> ()
{

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
                        error!{"Can't deserialize message: {:?}", e};
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

                        // NOTE The original aspect when first created has to be preserved, so
                        // passing screen sizes only, which are the size of the window when it was
                        // first created.
                        match state.build_dmo_gfx_from_yml_str(&message.data, false, false,
                                                               sx, sy, sx, sy,
                                                               Some(camera))
                        {
                            Ok(_) => {},
                            Err(e) => error!{"Can't perform SetDmo: {:?}", e},
                        }
                    },

                    SetDmoTime => {
                        let time: f64 = match serde_json::from_str(&message.data) {
                            Ok(x) => x,
                            Err(e) => {
                                error!{"Can't deserialize to time f64: {:?}", e};
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
                                error!{"Can't deserialize to Settings: {:?}", e};
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
                        error!{"Server sending error: {:?}", message.data},
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
            Err(e) => error!("state.update_rocket() returned: {:?}", e),
        }

        match state.update_vars() {
            Ok(_) => {},
            Err(e) => error!("state.update_vars() returned: {:?}", e),
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
                                            Err(e) => error!("{:?}", e),
                                        }
                                    }
                                },

                                // move time forward 2s
                                Right => if !pressed {
                                    if rocket.is_none() {
                                        state.move_time_ms(2000);
                                        match state.update_vars() {
                                            Ok(_) => {},
                                            Err(e) => error!("{:?}", e),
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
                            Err(e) => error!("{:?}", e),
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
}
