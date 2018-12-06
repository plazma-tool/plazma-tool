use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use rand::Rng;

use actix_web::ws;
use actix_web::actix::*;

use crate::utils::file_to_string;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

// FIXME should be more like 10 but client heartbeat fails
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

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
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for WsActor {
    type Context = ws::WebsocketContext<Self, ServerStateWrap>;

    /// Method is called on actor start. Store the client in ServerState and
    /// start the heartbeat process.
    fn started(&mut self, ctx: &mut Self::Context) {
        {
            let addr = ctx.address();
            let mut state = ctx.state().lock().expect("Can't lock ServerState.");
            println!("Adding client: {}", self.client_id);
            state.clients.insert(self.client_id, addr);
        }

        self.hb(ctx);
    }

    /// Remove client from list.
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        let mut state = ctx.state().lock().expect("Can't lock ServerState.");
        println!("Removing client: {}", self.client_id);
        state.clients.remove(&self.client_id);
        Running::Stop
    }
}

impl WsActor {
    pub fn new() -> Self {
        Self {
            client_id: rand::thread_rng().gen::<usize>(),
            hb: Instant::now(),
        }
    }

    /// Helper method that sends ping to client every second.
    ///
    /// Also this method checks heartbeats from client.
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MsgDataType {
    StartOpenGlPreview,
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
            ws::Message::Ping(m) => {
                self.hb = Instant::now();
                ctx.pong(&m);
            },

            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }

            ws::Message::Text(text) => {

                let message: Receiving = match serde_json::from_str(&text) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("Error on deserializing: {:?}", e);
                        return;
                    },
                };

                use self::MsgDataType::*;
                match message.data_type {
                    StartOpenGlPreview => {
                        // Repeat to everyone including the sender.
                        let state = ctx.state().lock().expect("Can't lock ServerState.");
                        for (_id, addr) in &state.clients {
                            let msg = Sending {
                                data_type: StartOpenGlPreview,
                                data: message.data.clone(),
                            };
                            addr.do_send(msg);
                        }
                    },

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

            // Echoes back the binary data.
            ws::Message::Binary(bin) => ctx.binary(bin),

            ws::Message::Close(_) => ctx.stop(),
        }

    }

}
