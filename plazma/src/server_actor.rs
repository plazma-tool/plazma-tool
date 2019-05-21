use std::path::PathBuf;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::process::{Command, Child};

use rand::Rng;

use actix_web::ws;
use actix_web::actix::*;

use crate::project_data::ProjectData;
use crate::app::AppInfo;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// `preview_client` is running the render loop. On start, it builds a Dmo using a
/// json blob from the Server. `Dmo.timeline.draw_ops_at_time(x)` returns a
/// `Vec<DrawOp>` which is used to draw the current frame.
///
/// `preview_client` has paused or playing state. When playing, it updates the
/// time and sends it to Server.
///
/// `preview_client` can receive a time value from server, and it will jump
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
    pub app_info: AppInfo,
    pub webview_sender_arc: Arc<Mutex<mpsc::Sender<String>>>,
    pub project_data: ProjectData,
    pub clients: HashMap<usize, Addr<ServerActor>>,
    pub preview_child: Option<Child>,
}

pub type ServerStateWrap = Arc<Mutex<ServerState>>;

impl ServerState {
    pub fn new(app_info: AppInfo,
               webview_sender_arc: Arc<Mutex<mpsc::Sender<String>>>,
               demo_yml_path: Option<PathBuf>)
        -> Result<ServerState, Box<Error>>
    {
        let state = ServerState {
            app_info: app_info,
            webview_sender_arc: webview_sender_arc,
            project_data: ProjectData::new(demo_yml_path)?,
            clients: HashMap::new(),
            preview_child: None,
        };

        Ok(state)
    }
}

/// Actor for websocket connection.
pub struct ServerActor {
    pub client_id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for ServerActor {
    type Context = ws::WebsocketContext<Self, ServerStateWrap>;

    /// Method is called on actor start. Store the client in ServerState and
    /// start the heartbeat process.
    fn started(&mut self, ctx: &mut Self::Context) {
        {
            let addr = ctx.address();
            let mut state = ctx.state().lock().expect("Can't lock ServerState.");
            info!("Adding client: {}", self.client_id);
            state.clients.insert(self.client_id, addr);
        }

        self.hb(ctx);
    }

    /// Remove client from list.
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        let mut state = ctx.state().lock().expect("Can't lock ServerState.");
        info!("Removing client: {}", self.client_id);
        state.clients.remove(&self.client_id);
        Running::Stop
    }
}

impl ServerActor {
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
                error!("👹 Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgDataType {
    NoOp,
    FetchDmo,
    SetDmo,
    SetDmoTime,
    GetDmoTime,
    SetShader,
    ShaderCompilationSuccess,
    ShaderCompilationFailed,
    ShowErrorMessage,
    SetSettings,
    StartPreview,
    StopPreview,
    PreviewOpened,
    PreviewClosed,
    ExitApp,
}

/// Message to send the project root and DmoData. The preview starts with a minimal demo which
/// doesn't have a project root, but when the server sends the user's demo, it will have to be read
/// from disk.
#[derive(Serialize, Deserialize, Debug)]
pub struct SetDmoMsg {
    pub project_root: Option<PathBuf>,
    pub dmo_data_json_str: String,
}

/// Message to update the content of a specific shader.
#[derive(Serialize, Deserialize, Debug)]
pub struct SetShaderMsg {
    pub idx: usize,
    pub content: String,
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
impl Handler<Sending> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: Sending, ctx: &mut Self::Context) {
        let body = serde_json::to_string(&msg).unwrap();
        ctx.text(body);
    }
}

/// Handling incoming messages from a client.
impl StreamHandler<ws::Message, ws::ProtocolError> for ServerActor {

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
                //info!{"Received: message.data_type: {:?}", message.data_type};

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
                                data: serde_json::to_string(&SetDmoMsg {
                                    project_root: state.project_data.project_root.clone(),
                                    dmo_data_json_str: serde_json::to_string(&state.project_data.dmo_data).unwrap(),
                                }).unwrap(),
                            };
                        }
                        let body = serde_json::to_string(&resp).unwrap();
                        ctx.text(body);
                    },

                    SetDmo => {
                        // Client is sending Dmo data. Deserialize and replace the
                        // ServerState.dmo. Serialize and send all other clients the
                        // new Dmo data.
                        match serde_json::from_str::<SetDmoMsg>(&message.data) {

                            Ok(dmo_msg) => {
                                info!{"Deserialized SetDmoMsg"};
                                let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                                state.project_data.project_root = dmo_msg.project_root.clone();
                                state.project_data.dmo_data = serde_json::from_str(&dmo_msg.dmo_data_json_str).unwrap();

                                for (id, addr) in &state.clients {
                                    if *id == self.client_id {
                                        continue;
                                    }

                                    //let resp = Sending {
                                    //    data_type: SetDmo,
                                    //    data: serde_json::to_string(&SetDmoMsg {
                                    //        project_root: state.project_data.project_root.clone(),
                                    //        dmo_data_json_str: serde_json::to_string(&state.project_data.dmo_data).unwrap(),
                                    //    }).unwrap(),
                                    //};
                                    let resp = Sending {
                                        data_type: message.data_type,
                                        data: message.data.clone(),
                                    };
                                    info!{"Sending: {:?}", &message.data_type};
                                    addr.do_send(resp);
                                }
                            },

                            Err(e) => {
                                error!{"Error deserializing Dmo: {:?}", e};
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
                        match serde_json::from_str::<f64>(&message.data) {
                            Ok(time) => {
                                //info!{"Deserialized time"};
                                let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

                                for (id, addr) in &state.clients {
                                    if *id == self.client_id {
                                        continue;
                                    }

                                    let resp = Sending {
                                        data_type: SetDmoTime,
                                        data: serde_json::to_string(&time).unwrap(),
                                    };
                                    //info!{"Sending SetDmoTime"};
                                    addr.do_send(resp);
                                }
                            },

                            Err(e) => {
                                error!{"Error deserializing time: {:?}", e};
                                // Could not deserialize data, tell client to show an error.
                                let resp = Sending {
                                    data_type: ShowErrorMessage,
                                    data: format!{"{:?}", e},
                                };
                                let body = serde_json::to_string(&resp).unwrap();
                                ctx.text(body);
                            }
                        }
                    },

                    GetDmoTime => {
                        // Client is requesting time. Send the message to other clients to respond.
                        let state = ctx.state().lock().expect("Can't lock ServerState.");

                        for (id, addr) in &state.clients {
                            if *id == self.client_id {
                                continue;
                            }

                            let resp = Sending {
                                data_type: GetDmoTime,
                                data: "".to_owned(),
                            };
                            addr.do_send(resp);
                        }
                    },

                    SetShader => {
                        // Client is sending a shader to be updated. We are not storing the shader
                        // sources in the DmoData, so we don't have to update it in the server
                        // state, only pass on the message to the other clients such as the OpenGL
                        // preview.

                        let state = ctx.state().lock().expect("Can't lock ServerState.");
                        for (id, addr) in &state.clients {
                            if *id == self.client_id {
                                continue;
                            }

                            let resp = Sending {
                                data_type: message.data_type,
                                data: message.data.clone(),
                            };
                            info!{"Sending: {:?}", &message.data_type};
                            addr.do_send(resp);
                        }
                    },

                    ShaderCompilationSuccess => {
                        let state = ctx.state().lock().expect("Can't lock ServerState.");
                        for (id, addr) in &state.clients {
                            if *id == self.client_id {
                                continue;
                            }

                            let resp = Sending {
                                data_type: message.data_type,
                                data: message.data.clone(),
                            };
                            info!{"Sending: {:?}", &message.data_type};
                            addr.do_send(resp);
                        }
                    },

                    ShaderCompilationFailed => {
                        let state = ctx.state().lock().expect("Can't lock ServerState.");
                        for (id, addr) in &state.clients {
                            if *id == self.client_id {
                                continue;
                            }

                            let resp = Sending {
                                data_type: message.data_type,
                                data: message.data.clone(),
                            };
                            info!{"Sending: {:?}", &message.data_type};
                            addr.do_send(resp);
                        }
                    },

                    SetSettings => {
                        match serde_json::from_str(&message.data) {
                            Ok(settings) => {
                                info!{"Deserialized Settings"};
                                let mut state = ctx.state().lock().expect("Can't lock ServerState.");
                                state.project_data.dmo_data.settings = settings;

                                for (id, addr) in &state.clients {
                                    if *id == self.client_id {
                                        continue;
                                    }

                                    let resp = Sending {
                                        data_type: SetSettings,
                                        data: serde_json::to_string(&state.project_data.dmo_data.settings).unwrap(),
                                    };
                                    info!{"Sending SetSettings"};
                                    addr.do_send(resp);
                                }
                            },

                            Err(e) => {
                                error!{"Error deserializing Settings: {:?}", e};
                                // Could not deserialize data, tell client to show an error.
                                let resp = Sending {
                                    data_type: ShowErrorMessage,
                                    data: format!{"{:?}", e},
                                };
                                let body = serde_json::to_string(&resp).unwrap();
                                ctx.text(body);
                            }
                        }
                    },

                    ShowErrorMessage => {
                        // Client is sending error message to server.
                        // TODO
                    },

                    StartPreview => {
                        let mut state = ctx.state().lock().expect("Can't lock ServerState.");

                        if let Some(ref mut child) = state.preview_child {

                            match child.try_wait() {
                                Ok(Some(_)) => {
                                    info!("🔎 Spawn a new process.");
                                    let new_child: Option<Child> =
                                        run_preview_command(&state.app_info.path_to_binary);

                                    if new_child.is_some() {
                                        state.preview_child = new_child;
                                    }
                                },

                                Ok(None) => warn!("⚡ Still running."),

                                Err(e) => error!("🔥 Can't wait for child process: {:?}", e),
                            }

                            return;

                        } else {

                            info!("🔎 Spawn a new process.");
                            let new_child: Option<Child> =
                                run_preview_command(&state.app_info.path_to_binary);

                            if new_child.is_some() {
                                state.preview_child = new_child;
                            }

                        }

                    },

                    StopPreview => {
                        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

                        for (id, addr) in &state.clients {
                            if *id == self.client_id {
                                continue;
                            }

                            let resp = Sending {
                                data_type: StopPreview,
                                data: "".to_owned(),
                            };
                            info!{"💬 Sending StopPreview to client {:?}", self.client_id};
                            addr.do_send(resp);
                        }
                    },

                    PreviewOpened => {
                        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

                        for (id, addr) in &state.clients {
                            if *id == self.client_id {
                                continue;
                            }

                            let resp = Sending {
                                data_type: PreviewOpened,
                                data: "".to_owned(),
                            };
                            info!{"💬 Sending PreviewOpened to client {:?}", self.client_id};
                            addr.do_send(resp);
                        }
                    },

                    PreviewClosed => {
                        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

                        for (id, addr) in &state.clients {
                            if *id == self.client_id {
                                continue;
                            }

                            let resp = Sending {
                                data_type: PreviewClosed,
                                data: "".to_owned(),
                            };
                            info!{"💬 Sending PreviewClosed to client {:?}", self.client_id};
                            addr.do_send(resp);
                        }
                    },

                    ExitApp => {
                        {
                            let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

                            // First, send StopPreview.

                            for (id, addr) in &state.clients {
                                if *id == self.client_id {
                                    continue;
                                }

                                let resp = Sending {
                                    data_type: StopPreview,
                                    data: "".to_owned(),
                                };
                                info!{"💬 Sending StopPreview to client {:?}", self.client_id};
                                addr.do_send(resp);
                            }

                            // Send ExitWebview to the webview window.
                            let webview_sender = state.webview_sender_arc.lock()
                                .expect("Can't lock webview sender.");
                            match webview_sender.send("ExitWebview".to_owned()) {
                                Ok(x) => x,
                                Err(e) => error!("🔥 Can't send ExitWebview on state.webview_sender: {:?}", e),
                            };
                        }

                        // Stop the Actor, stop the System.
                        ctx.stop();
                        System::current().stop();
                    },
                }
            },

            // Echoes back the binary data.
            ws::Message::Binary(bin) => ctx.binary(bin),

            ws::Message::Close(_) => ctx.stop(),
        }

    }

}

fn run_preview_command(path_to_binary: &PathBuf) -> Option<Child>
{
    // std::process::Command inherits the current process's working directory.

    let bin_cmd = format!("{:?} preview", path_to_binary);

    if cfg!(target_os = "windows") {

        match Command::new("cmd").arg("/C").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned preview");
                return Some(child);
            },
            Err(e) => {
                error!("🔥 failed to spawn: {:?}", e);
                return None;
            },
        }

    } else {
        // Not testing for `cfg!(target_os = "linux") || cfg!(target_os =
        // "macos")`, try to run some command in any case.

        match Command::new("sh").arg("-c").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned preview");
                return Some(child);
            },
            Err(e) => {
                error!("🔥 failed to spawn: {:?}", e);
                return None;
            },
        }
    }
}
