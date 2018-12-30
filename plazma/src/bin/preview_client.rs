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

use actix::*;
use actix_web::ws;

use futures::Future;

use glutin::{Window, GlWindow, GlContext, EventsLoop, Event, WindowEvent, VirtualKeyCode, ElementState};

use plazma::dmo_data::DmoData;
use plazma::server_actor::Receiving;
use plazma::preview_client::client_actor::ClientActor;
use plazma::preview_client::preview_state::PreviewState;

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
    //unsafe { gl::Enable(gl::DEPTH_TEST); }

    // set the clear color at least once
    unsafe { gl::ClearColor(0.1, 0.2, 0.3, 1.0); }

    let logical_size = window.window().get_inner_size().unwrap();
    // NOTE should we use LogicalSize instead, keep in mind the u32 truncates later
    let (wx, wy) = (logical_size.width, logical_size.height);

    // Window size = screen size because we start fullscreen.
    let mut state =
        PreviewState::new(wx as f64, wy as f64,
                          wx as f64, wy as f64).unwrap();

    // Start with a default DmoData until we receive update from the server.
    let dmo_data = DmoData::default();
    state.build(&dmo_data).unwrap();

    state.set_is_paused(false);

    render_loop(&window, &mut events_loop, &mut state, rx);

    client_handle.join().unwrap();

    info!("gg thx!");
}

fn render_loop(window: &GlWindow,
               events_loop: &mut EventsLoop,
               state: &mut PreviewState,
               channel_receiver: mpsc::Receiver<String>) {

    while state.get_is_running() {

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
                                if let Some(ref scene) = d.context.quad_scenes.get(0) {
                                    state.dmo_gfx.update_shader_src(0, &scene.vert_src);
                                    state.dmo_gfx.update_shader_src(1, &scene.frag_src);
                                    state.should_recompile = true;
                                } else {
                                    println!("Error, no index 0");
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
        state.draw_anyway = false;

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

        // 2. deal with events

        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => state.set_is_running(false),

                    WindowEvent::Resized(logical_size) => {
                        // NOTE should we use LogicalSize instead, keep in mind the u32 truncates later
                        let (wx, wy) = (logical_size.width, logical_size.height);
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

        // 3. move, update camera in explore mode

        // TODO

        // 4. rebuild when assets change

        // TODO

        // 5. draw if we are not paused or should draw anyway (e.g. assets changed)

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

