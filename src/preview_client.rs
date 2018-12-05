extern crate actix;

extern crate serde_json;

extern crate byteorder;
extern crate bytes;
extern crate futures;

extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_tcp;

use glium::{self, glutin, Surface};
use glium::glutin::{Event, VirtualKeyCode, WindowEvent};

use std::thread::sleep;
use std::path::PathBuf;
use std::io;
use std::time::Duration;

use actix::*;

use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_io::codec::{Decoder, Encoder};
use tokio_tcp::TcpStream;

use futures::Future;
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};

use crate::types::*;
use crate::utils::file_to_string;
use crate::preview_state::PreviewState;

pub struct PreviewClientCodec;

pub struct PreviewClient {
    pub framed: actix::io::FramedWrite<WriteHalf<TcpStream>, PreviewClientCodec>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PreviewRequest {
    Ping,
    Message(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PreviewResponse {
    Ping,
    Message(String),
}

pub struct Sending {
    pub data: String,
}

impl Message for Sending {
    type Result = ();
}

impl Actor for PreviewClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        println!("PreviewClient Disconnected");

        // Stop application on disconnect
        System::current().stop();

        Running::Stop
    }
}

impl PreviewClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        println!("PreviewClient::hb()");
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.framed.write(PreviewRequest::Ping);
            act.hb(ctx);
        });
    }
}

impl Decoder for PreviewClientCodec {
    type Item = PreviewResponse;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.split_to(2);
            let buf = src.split_to(size);
            Ok(Some(serde_json::from_slice::<PreviewResponse>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for PreviewClientCodec {
    type Item = PreviewRequest;
    type Error = io::Error;

    fn encode(&mut self, msg: PreviewRequest, dst: &mut BytesMut,) -> Result<(), Self::Error> {
        let msg = serde_json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        println!("\nencode(): {}\n", msg);

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}

impl actix::io::WriteHandler<io::Error> for PreviewClient {}

/// Sending a message to the server.
impl Handler<Sending> for PreviewClient {
    type Result = ();

    fn handle(&mut self, msg: Sending, _: &mut Context<Self>) {
        let m = msg.data.trim();
        println!("\nhandle(): {}\n", m);
        self.framed.write(PreviewRequest::Message(m.to_owned()));
    }
}

/// Handling incoming messages from the server.
impl StreamHandler<PreviewResponse, io::Error> for PreviewClient {
    fn handle(&mut self, msg: PreviewResponse, _: &mut Context<Self>) {
        match msg {
            PreviewResponse::Message(ref msg) => {
                println!("message: {}", msg);
            }
            _ => (),
        }
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2],
}

implement_vertex!(Vertex, pos, tex);

pub fn start_opengl_preview(addr: &Addr<PreviewClient>) {

    addr.do_send(Sending{ data: "hey".to_string() });

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

    let mut preview_state: PreviewState = PreviewState::new().unwrap();

    let program = glium::Program::from_source(&display,
                                              &preview_state.vertex_shader_src,
                                              &preview_state.fragment_shader_src,
                                              None).unwrap();

    preview_state.set_is_paused(false);

    while preview_state.get_is_running() {

        // 1. update time

        preview_state.update_time();

        // send Gui with updated time
        let mut gui = Gui::default();
        gui.time = preview_state.time;

        //let resp = WsResponse {
        //    data_type: WsDataType::GuiData,
        //    data: serde_json::to_string(&gui).unwrap(),
        //};
        //let data = serde_json::to_string(&resp).unwrap();

        let data = serde_json::to_string(&gui).unwrap();

        //println!("start_opengl_preview(): .do_send(): {}", data);

        // FIXME don't send data with every frame
        //addr.do_send(Sending{ data: data });

        preview_state.draw_anyway = false;

        let uniforms = uniform! {
            iGlobalTime: preview_state.time,
            iResolution: preview_state.window_resolution,
            bg_color:    [0.9_f32, 0.4_f32, 0.1_f32],
        };

        // 5. deal with events

        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => preview_state.set_is_running(false),

                    WindowEvent::KeyboardInput{ input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            preview_state.set_is_running(false);
                        }
                    },

                    WindowEvent::Resized(size) => {
                        let (wx, wy): (f64, f64) = size.into();
                        preview_state.window_resolution = [wx as f32, wy as f32];
                        preview_state.draw_anyway = true;
                    },
                    _ => (),
                },
                _ => (),
            }
        });

        // 7. draw if we are not paused or should draw anyway (e.g. window resized)

        let mut target = display.draw();

        if !preview_state.get_is_paused() || preview_state.draw_anyway {
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        }

        // ship the frame

        target.finish().unwrap();

        // 8. sleep if there is time left

        preview_state.t_delta = preview_state.t_frame_start.elapsed();

        if preview_state.t_delta < preview_state.t_frame_target {
            if let Some(t_sleep) = preview_state.t_frame_target.checked_sub(preview_state.t_delta)  {
                sleep(t_sleep);
            }
        }
    }
}
