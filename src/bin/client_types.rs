use std::thread::{self, sleep};
use std::sync::mpsc;
use std::time::Duration;

use actix::*;
use actix_web::ws::{ClientWriter, Message, ProtocolError};

use glium::{self, glutin, Surface};
use glium::glutin::{Event, VirtualKeyCode, WindowEvent};

use plasma::server_types::*;
use crate::preview_state::PreviewState;

pub struct PreviewClient {
    pub writer: ClientWriter,
    pub channel_sender: mpsc::Sender<String>,
}

#[derive(Message)]
pub struct ClientMessage{
    pub data: String,
}

impl Actor for PreviewClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        info!("PreviewClient Disconnected");

        // Stop application on disconnect
        System::current().stop();

        Running::Stop
    }
}

/// Sending a message to the server.
impl Handler<ClientMessage> for PreviewClient {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let m = format!("{}", msg.data.trim());
        self.writer.text(m)
    }
}

impl PreviewClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.writer.ping("");
            act.hb(ctx);

            // TODO client should also check for a timeout here, similar to the
            // server code
        });
    }
}

/// Handling incoming messages from the server.
impl StreamHandler<Message, ProtocolError> for PreviewClient {
    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {

        match msg {
            Message::Text(text) => {
                match self.channel_sender.send(text) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("Can't send on channel: {:?}", e);
                    }
                }
            },
            _ => (),
        }
    }

    fn started(&mut self, _: &mut Context<Self>) {
        println!("Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2],
}

implement_vertex!(Vertex, pos, tex);

impl PreviewState {

    pub fn start_opengl_preview(&mut self, channel_receiver: mpsc::Receiver<String>) {

        //addr.do_send(ClientMessage("hey".to_string()));

        // Setup glium

        let mut events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let quad = vec![
            Vertex { pos: [-1.0, -1.0], tex: [0.0, 0.0] },
            Vertex { pos: [-1.0,  1.0], tex: [0.0, 1.0] },
            Vertex { pos: [ 1.0, -1.0], tex: [1.0, 0.0] },
            Vertex { pos: [ 1.0,  1.0], tex: [1.0, 1.0] },
        ];

        let vertex_buffer = glium::VertexBuffer::new(&display, &quad).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let mut program = glium::Program::from_source(&display,
                                                      &self.vertex_shader_src,
                                                      &self.fragment_shader_src,
                                                      None).unwrap();

        self.set_is_paused(false);

        while self.get_is_running() {

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

                    use plasma::server_types::MsgDataType::*;
                    match message.data_type {

                        NoOp => {},

                        FetchGui => {},

                        SetGui => {},

                        SetGuiTime => {},

                        SetFragmentShader => {
                            let frag = message.data.trim_matches('"').replace("\\n", "\n");
                            self.set_fragment_shader_src(frag);
                        },

                        ShowErrorMessage =>
                            error!("Server sending error: {:?}", message.data),
                    }

                },
                Err(_) => {},
            }

            // 0. recompile if needed
            if self.should_recompile {
                match glium::Program::from_source(&display,
                                                  &self.vertex_shader_src,
                                                  &self.fragment_shader_src,
                                                  None) {
                    Ok(p) => {
                        program = p;
                    },

                    Err(e) => {
                        error!("Failed to compile shader: {:?}", e);
                    }
                }
                self.should_recompile = false;
            }

            // 1. update time

            self.update_time();

            self.draw_anyway = false;

            let uniforms = uniform! {
                iGlobalTime: self.time,
                iResolution: self.window_resolution,
                bg_color:    [0.9_f32, 0.4_f32, 0.1_f32],
            };

            // 5. deal with events

            events_loop.poll_events(|event| {
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => self.set_is_running(false),

                        WindowEvent::KeyboardInput{ input, .. } => {
                            if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                                self.set_is_running(false);
                            }
                        },

                        WindowEvent::Resized(size) => {
                            let (wx, wy): (f64, f64) = size.into();
                            self.window_resolution = [wx as f32, wy as f32];
                            self.draw_anyway = true;
                        },
                        _ => (),
                    },
                    _ => (),
                }
            });

            // 7. draw if we are not paused or should draw anyway (e.g. window resized)

            let mut target = display.draw();

            if !self.get_is_paused() || self.draw_anyway {
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
            }

            // ship the frame

            target.finish().unwrap();

            // 8. sleep if there is time left

            self.t_delta = self.t_frame_start.elapsed();

            if self.t_delta < self.t_frame_target {
                if let Some(t_sleep) = self.t_frame_target.checked_sub(self.t_delta)  {
                    sleep(t_sleep);
                }
            }
        }
    }

}
