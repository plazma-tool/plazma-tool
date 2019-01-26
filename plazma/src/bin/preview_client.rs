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

extern crate glutin;

extern crate plazma;

use std::thread::{self, sleep};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;
use std::path::PathBuf;

use actix::*;
use actix_web::ws;

use futures::Future;

use glutin::{Window, GlWindow, GlContext, EventsLoop, Event, WindowEvent, VirtualKeyCode, ElementState, MouseButton};

use plazma::dmo_data::DmoData;
use plazma::server_actor::Receiving;
use plazma::preview_client::client_actor::ClientActor;
use plazma::preview_client::preview_state::PreviewState;
use plazma::utils::file_to_string;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info,preview_client=info");
    env_logger::init();

    let plazma_server_port = Arc::new(8080);

    // Channel to pass messages from the Websocket client to the OpenGL window.
    let (tx, rx) = mpsc::channel();

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
                    let _addr = ClientActor::create(|ctx| {
                        ClientActor::add_stream(reader, ctx);
                        ClientActor{
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

    // === Build the Window ===

    let start_fullscreen = false;

    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = if start_fullscreen {
        let monitor = events_loop.get_available_monitors().nth(0).expect("no monitor found");
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

    gl::load_with(|ptr| window.context().get_proc_address(ptr) as *const _);

    // NOTE for cube
    unsafe { gl::Enable(gl::DEPTH_TEST); }

    // set the clear color at least once
    unsafe { gl::ClearColor(0.1, 0.2, 0.3, 1.0); }

    let logical_size = window.window().get_inner_size().unwrap();
    let dpi_factor = window.window().get_hidpi_factor();
    let physical_size = logical_size.to_physical(dpi_factor);
    // NOTE should we use LogicalSize instead, keep in mind the u32 truncates later
    let (wx, wy) = (physical_size.width, physical_size.height);

    // Window size = screen size because we start fullscreen.
    let mut state =
        PreviewState::new(wx as f64, wy as f64,
                          wx as f64, wy as f64).unwrap();

    // Start with a minimal demo until we receive update from the server.
    let demo_yml_path = PathBuf::from("data".to_owned())
        .join(PathBuf::from("minimal".to_owned()))
        .join(PathBuf::from("demo.yml".to_owned()));

    let text: String = file_to_string(&demo_yml_path).unwrap();
    let dmo_data = DmoData::new_from_yml_str(&text).unwrap();

    state.build_dmo_gfx(&dmo_data).unwrap();

    state.set_is_paused(false);

    render_loop(&window, &mut events_loop, &mut state, rx);

    println!("Render loop exited.");

    //add.do_send(ClientMessage("stop the client".to_string()));

    client_handle.join().unwrap();

    info!("gg thx!");
}

fn render_loop(window: &GlWindow,
               events_loop: &mut EventsLoop,
               state: &mut PreviewState,
               channel_receiver: mpsc::Receiver<String>) {

    let mut dpi_factor = window.window().get_hidpi_factor();

    while state.get_is_running() {

        state.draw_anyway = false;

        // 000. handle server messages

        match channel_receiver.try_recv() {
            Ok(text) => {
                // FIXME return a NOOP otherwise it returns from the function.
                let message: Receiving = match serde_json::from_str(&text) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("Can't deserialize message: {:?}", e);
                        return;
                    },
                };

                use plazma::server_actor::MsgDataType::*;
                match message.data_type {

                    NoOp => {},

                    FetchDmo => {},

                    SetDmo => {
                        match serde_json::from_str::<DmoData>(&message.data) {
                            Ok(d) => {
                                match state.build_dmo_gfx(&d) {
                                    Ok(_) => {},
                                    Err(e) => error!("Can't perform SetDmo: {:?}",e ),
                                }
                            },
                            Err(e) => error!("Can't deserialize Dmo: {:?}", e),
                        };
                    },

                    SetDmoTime => {},

                    ShowErrorMessage =>
                        error!("Server sending error: {:?}", message.data),
                }

            },
            Err(_) => {},
        }

        // 00. recompile if flag was set
        state.recompile_dmo();

        // 0. update time

        state.update_time_frame_start();

        if state.dmo_gfx.settings.total_length < state.get_time() {
            break;
        }

        // 1. sync vars (time, camera, etc.)

        /*
        match state.update_rocket(&mut rocket) {
            Ok(_) => {},
            Err(e) => error!("{:?}", e),
        }
         */

        match state.update_vars() {
            Ok(_) => {},
            Err(e) => error!("{:?}", e),
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

                    WindowEvent::KeyboardInput{ device_id, input } => {
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

                                _ => (),
                            }
                        }
                    },

                    WindowEvent::CursorMoved{ device_id, position, modifiers } => {
                        let (mx, my) = position.into();
                        state.callback_mouse_moved(mx, my);
                    },

                    WindowEvent::MouseWheel{ device_id, delta, phase, modifiers } => {
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

                    WindowEvent::MouseInput{ device_id, state: pressed_state, button, modifiers } => {
                        state.callback_mouse_input(pressed_state, button);
                    },

                    WindowEvent::CursorEntered{ device_id } => {},
                    WindowEvent::CursorLeft{ device_id } => {},

                    WindowEvent::HiDpiFactorChanged(dpi) => {
                        dpi_factor = dpi;
                    }

                    WindowEvent::Refresh => {
                        state.draw_anyway = true;
                    },

                    WindowEvent::Resized(logical_size) => {
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

        // 4. rebuild when assets change on disc

        // TODO

        // 5. draw if we are not paused or should draw anyway (e.g. assets changed)

        state.dmo_gfx.update_polygon_context();

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

