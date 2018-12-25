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

#[macro_use]
extern crate glium;

extern crate plasma;

use std::thread::{self, sleep};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use actix::*;
use actix_web::ws;

use futures::Future;

use glium::{glutin, Surface};
use glium::glutin::{Event, VirtualKeyCode, WindowEvent};

use plasma::dmo_data::DmoData;
use plasma::preview_client::dmo_gfx::{DmoGfx, QuadSceneGfx, Vertex};

use plasma::server_actor::Receiving;
use plasma::preview_client::client_actor::ClientActor;
use plasma::preview_client::preview_state::PreviewState;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info,preview_client=info");
    env_logger::init();

    let plasma_server_port = Arc::new(8080);

    // Channel to pass messages from the Websocket client to the OpenGL window.
    let (tx, rx) = mpsc::channel();

    // Start the Websocket client on a separate thread so that it is not blocked
    // (and is not blocking) the OpenGL window.

    let plasma_server_port_a = Arc::clone(&plasma_server_port);

    let client_handle = thread::spawn(move || {

        let sys = actix::System::new("preview client");

        // Start a WebSocket client and connect to the server.

        // FIXME check if server is up

        Arbiter::spawn(
            ws::Client::new(format!{"http://127.0.0.1:{}/ws/", plasma_server_port_a})
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

    let mut state = PreviewState::new().unwrap();
    render_loop(&mut state, rx);

    client_handle.join().unwrap();

    info!("gg thx!");
}

fn render_loop(state: &mut PreviewState,
               channel_receiver: mpsc::Receiver<String>) {

    //addr.do_send(ClientMessage("hey".to_string()));

    // Setup glium

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // Start with a default DmoData until we receive update from the server.
    let mut dmo_data = DmoData::default();

    // Build the OpenGL objects from the DmoData.
    let mut dmo_gfx = DmoGfx {
        quad_scenes: vec![],
    };

    let quad_vertices = vec![
        Vertex { pos: [-1.0, -1.0], tex: [0.0, 0.0] },
        Vertex { pos: [-1.0,  1.0], tex: [0.0, 1.0] },
        Vertex { pos: [ 1.0, -1.0], tex: [1.0, 0.0] },
        Vertex { pos: [ 1.0,  1.0], tex: [1.0, 1.0] },
    ];

    // Build QuadScenes
    let mut quad_scenes: Vec<QuadSceneGfx> = Vec::new();
    for quad_scene_data in dmo_data.context_data.quad_scenes.iter() {
        let vbo = glium::VertexBuffer::new(&display, &quad_vertices).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let program = glium::Program::from_source(&display,
                                                  &quad_scene_data.vert_src,
                                                  &quad_scene_data.frag_src,
                                                  None).unwrap();

        let quad_scene_gfx = QuadSceneGfx {
            name: quad_scene_data.name.clone(),
            vbo: vbo,
            indices: indices,
            program: program,
        };

        quad_scenes.push(quad_scene_gfx);
    }

    dmo_gfx.quad_scenes = quad_scenes;

    let draw_ops = dmo_data.timeline.draw_ops_at_time(0.0);

    state.set_is_paused(false);

    while state.get_is_running() {

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

                use plasma::server_actor::MsgDataType::*;
                match message.data_type {

                    NoOp => {},

                    FetchDmo => {},

                    SetDmo => {
                        match serde_json::from_str(&message.data) {
                            Ok(d) => {
                                dmo_data = d;
                                state.should_recompile = true;
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

        // 0. recompile if needed
        if state.should_recompile {

            // FIXME don't rebuild all of them

            let mut quad_scenes: Vec<QuadSceneGfx> = Vec::new();

            for quad_scene_data in dmo_data.context_data.quad_scenes.iter() {
                let vbo = glium::VertexBuffer::new(&display, &quad_vertices).unwrap();
                let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

                let program = glium::Program::from_source(&display,
                                                          &quad_scene_data.vert_src,
                                                          &quad_scene_data.frag_src,
                                                          None).unwrap();

                let quad_scene_gfx = QuadSceneGfx {
                    name: quad_scene_data.name.clone(),
                    vbo: vbo,
                    indices: indices,
                    program: program,
                };

                quad_scenes.push(quad_scene_gfx);
            }

            dmo_gfx.quad_scenes = quad_scenes;

            state.should_recompile = false;
        }

        // 1. update time

        state.update_time();

        state.draw_anyway = false;

        let uniforms = uniform! {
            iGlobalTime: state.time,
            iResolution: state.window_resolution,
            bg_color:    [0.9_f32, 0.4_f32, 0.1_f32],
        };

        // 5. deal with events

        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => state.set_is_running(false),

                    WindowEvent::KeyboardInput{ input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            state.set_is_running(false);
                        }
                    },

                    WindowEvent::Resized(size) => {
                        let (wx, wy): (f64, f64) = size.into();
                        state.window_resolution = [wx as f32, wy as f32];
                        state.draw_anyway = true;
                    },
                    _ => (),
                },
                _ => (),
            }
        });

        // 7. draw if we are not paused or should draw anyway (e.g. window resized)

        let mut target = display.draw();

        if !state.get_is_paused() || state.draw_anyway {
            target.clear_color(0.0, 0.0, 0.0, 1.0);

            for quad in dmo_gfx.quad_scenes.iter() {
                target.draw(&quad.vbo,
                            &quad.indices,
                            &quad.program,
                            &uniforms,
                            &Default::default())
                    .unwrap();
            }
        }

        // ship the frame

        target.finish().unwrap();

        // 8. sleep if there is time left

        state.t_delta = state.t_frame_start.elapsed();

        if state.t_delta < state.t_frame_target {
            if let Some(t_sleep) = state.t_frame_target.checked_sub(state.t_delta)  {
                sleep(t_sleep);
            }
        }
    }
}
