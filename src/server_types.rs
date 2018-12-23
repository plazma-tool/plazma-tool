use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use rand::Rng;

use actix_web::ws;
use actix_web::actix::*;

use crate::project_data::ProjectData;
use crate::utils::file_to_string;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// PreviewClient is running the render loop. On start, it builds a Dmo using a
/// json blob from the Server. `Dmo.timeline.draw_ops_at_time(x)` returns a
/// `Vec<DrawOp>` which is used to draw the current frame.
///
/// PreviewClient has paused or playing state. When playing, it updates the
/// time and sends it to Server.
///
/// PreviewClient can receive a time value from server, and it will jump
/// there.
///
/// React Gui renders the time scrub from time value in DmoBlob. When
/// playing, it receives the time value from Server.
///
/// Server sends the React Gui a DmoBlob as a JSON string. React
/// deserializes that and renders the Gui components. Value changes are
/// passed back to the server as messages. Server passes messages on to the
/// PreviewClient, which rebuilds OpenGL objects if necessary.
pub struct ServerState {
    pub project_data: ProjectData,
    pub clients: HashMap<usize, Addr<WsActor>>,
}

pub type ServerStateWrap = Arc<Mutex<ServerState>>;

impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            project_data: ProjectData::default(),
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
    NoOp,
    FetchDmo,
    SetDmo,
    SetDmoTime,
    //SetFragmentShader,
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
                    NoOp => {},

                    FetchDmo => {
                        // Client is asking for Dmo data. Serialize ServerState.dmo
                        // and send it back.
                        let resp;
                        {
                            let state = ctx.state().lock().expect("Can't lock ServerState.");
                            resp = Sending {
                                data_type: SetDmo,
                                data: serde_json::to_string(&state.project_data.dmo).unwrap(),
                            };
                        }
                        let body = serde_json::to_string(&resp).unwrap();
                        ctx.text(body);
                    },

                    SetDmo => {
                        // Client is sending Dmo data. Deserialize and replace the
                        // ServerState.dmo. Serialize and send all other clients the
                        // new Dmo data.
                        match serde_json::from_str(&message.data) {

                            Ok(dmo) => {
                                let mut state = ctx.state().lock().expect("Can't lock ServerState.");
                                state.project_data.dmo = dmo;

                                for (id, addr) in &state.clients {
                                    if *id == self.client_id {
                                        continue;
                                    }

                                    let resp = Sending {
                                        data_type: SetDmo,
                                        data: serde_json::to_string(&state.project_data.dmo).unwrap(),
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

                    SetDmoTime => {
                        // Client is setting time. Deserialize, update
                        // ServerState and send to other clients.

                        // TODO
                    },

                    /*
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
                    */

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
