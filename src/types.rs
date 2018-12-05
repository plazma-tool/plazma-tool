extern crate rand;

extern crate actix_web;
extern crate serde_json;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;

use rand::Rng;

use actix_web::ws;
use actix_web::actix::*;

use crate::utils::file_to_string;

#[derive(Debug, Serialize, Deserialize)]
pub struct Gui {
    pub time: f32,
    pub vertex_shader_src: String,
    pub fragment_shader_src: String,
}

impl Default for Gui {
    fn default() -> Gui {
        Gui {
            time: 0.0,
            vertex_shader_src: file_to_string(&PathBuf::from("./data/screen_quad.vert")).unwrap(),
            fragment_shader_src: file_to_string(&PathBuf::from("./data/shader.frag")).unwrap(),
        }
    }
}

pub struct ServerState {
    pub gui: Gui,
    pub clients: HashMap<usize, Addr<WsActor>>,
}

pub type ServerStateWrap = Arc<Mutex<ServerState>>;

impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            gui: Gui::default(),
            clients: HashMap::new(),
        }
    }
}

/// Actor for websocket connection.
pub struct WsActor {
    pub client_id: usize,
}

impl Actor for WsActor {
    type Context = ws::WebsocketContext<Self, ServerStateWrap>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let id = rand::thread_rng().gen::<usize>();
        let addr = ctx.address();
        self.client_id = id;
        let mut state = ctx.state().lock().expect("Can't lock ServerState.");
        state.clients.insert(id, addr);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        let mut state = ctx.state().lock().expect("Can't lock ServerState.");
        state.clients.remove(&self.client_id);
        Running::Stop
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MsgDataType {
    FetchGui,
    SetGui,
    SetGuiTime,
    SetFragmentShader,
    ShowErrorMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sending {
    pub data_type: MsgDataType,
    pub data: String,
}

impl Message for Sending {
    type Result = ();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Receiving {
    pub data_type: MsgDataType,
    pub data: String,
}

/// Sending a message to a client.
impl Handler<Sending> for WsActor {
    type Result = ();

    fn handle(&mut self, msg: Sending, ctx: &mut Self::Context) {
        let body = serde_json::to_string(&msg).unwrap();
        ctx.text(body);
    }
}

/// Handling incoming messages from a client.
impl StreamHandler<ws::Message, ws::ProtocolError> for WsActor {

    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {

        match msg {
            ws::Message::Ping(m) => ctx.pong(&m),

            ws::Message::Text(text) => {

                println!("\nMSG: {:?}\n", text);

                let message: Receiving = serde_json::from_str(&text).unwrap();

                use crate::MsgDataType::*;
                match message.data_type {

                    FetchGui => {
                        // Client is asking for gui data. Serialize ServerState.Gui
                        // and send it back.
                        let resp;
                        {
                            let state = ctx.state().lock().expect("Can't lock ServerState.");
                            resp = Sending {
                                data_type: SetGui,
                                data: serde_json::to_string(&state.gui).unwrap(),
                            };
                        }
                        let body = serde_json::to_string(&resp).unwrap();
                        ctx.text(body);
                    },

                    SetGui => {
                        // Client is sending gui data. Deserialize and replace the
                        // ServerState.Gui. Serialize and send all other clients the
                        // new Gui data.
                        match serde_json::from_str(&message.data) {

                            Ok(gui) => {
                                let mut state = ctx.state().lock().expect("Can't lock ServerState.");
                                state.gui = gui;

                                for (id, addr) in &state.clients {
                                    if *id == self.client_id {
                                        continue;
                                    }

                                    let resp = Sending {
                                        data_type: SetGui,
                                        data: serde_json::to_string(&state.gui).unwrap(),
                                    };
                                    addr.do_send(resp);
                                }

                            },

                            Err(e) => {
                                // Could not deserialize data, tell client to show an error.
                                let resp = Sending {
                                    data_type: ShowErrorMessage,
                                    data: format!{"{:?}", e},
                                };
                                let body = serde_json::to_string(&resp).unwrap();
                                ctx.text(body);
                            }
                        }
                    }

                    SetGuiTime => {
                        // Client is setting time. Deserialize, update
                        // ServerState and send to other clients.

                        // TODO
                    },

                    SetFragmentShader => {
                        // Client is setting fragment shader. Update ServerState
                        // and send to other clients.

                        let mut state = ctx.state().lock().expect("Can't lock ServerState.");
                        state.gui.fragment_shader_src = message.data;

                        for (id, addr) in &state.clients {
                            if *id == self.client_id {
                                continue;
                            }

                            let resp = Sending {
                                data_type: SetFragmentShader,
                                data: serde_json::to_string(&state.gui.fragment_shader_src).unwrap(),
                            };
                            addr.do_send(resp);
                        }
                    },


                    ShowErrorMessage => {
                        // Client is sending error message to server.
                        // TODO
                    },
                }
            },

            ws::Message::Binary(bin) => ctx.binary(bin),

            ws::Message::Close(_) => ctx.stop(),

            _ => (),
        }

    }

}
