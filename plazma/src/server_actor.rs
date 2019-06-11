use std::path::PathBuf;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::process::{Command, Child};

use rand::Rng;

use actix_web::ws;
use actix_web::actix::*;

use crate::project_data::{ProjectData, NewProjectTemplate};
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
    pub dialogs_child: Option<Child>,
}

pub type ServerStateWrap = Arc<Mutex<ServerState>>;

impl ServerState {
    pub fn new(app_info: AppInfo,
               webview_sender_arc: Arc<Mutex<mpsc::Sender<String>>>,
               demo_yml_path: Option<PathBuf>)
        -> Result<ServerState, Box<dyn Error>>
    {
        let state = ServerState {
            app_info: app_info,
            webview_sender_arc: webview_sender_arc,
            project_data: ProjectData::new(demo_yml_path, false)?,
            clients: HashMap::new(),
            preview_child: None,
            dialogs_child: None,
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

    fn repeat_message_to_others(&self, ctx: &<Self as Actor>::Context, message: &Receiving) {
        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
        for (id, addr) in &state.clients {
            if *id == self.client_id {
                continue;
            }

            let resp = Sending {
                data_type: message.data_type,
                data: message.data.clone(),
            };
            //info!{"💬 Sending {:?} to client {:?}", &message.data_type, id};
            addr.do_send(resp);
        }
    }

    fn send_message_to_everyone(&self, ctx: &<Self as Actor>::Context, message: &Sending) {
        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
        for (_id, addr) in &state.clients {
            let resp = Sending {
                data_type: message.data_type,
                data: message.data.clone(),
            };
            //info!{"💬 Sending {:?} to client {:?}", &message.data_type, id};
            addr.do_send(resp);
        }
    }

    fn send_message_to_others(&self, ctx: &<Self as Actor>::Context, message: &Sending) {
        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
        for (id, addr) in &state.clients {
            if *id == self.client_id {
                continue;
            }

            let resp = Sending {
                data_type: message.data_type,
                data: message.data.clone(),
            };
            //info!{"💬 Sending {:?} to client {:?}", &message.data_type, id};
            addr.do_send(resp);
        }
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
    SetMetadata,
    StartPreview,
    StopPreview,
    PreviewOpened,
    PreviewClosed,
    StartDialogs,
    OpenProjectFileDialog,
    OpenProjectFilePath,
    ReloadProject,
    SaveProject,
    NewProject,
    ExitApp,
}

/// Message to send the project root and DmoData. The preview starts with a minimal demo which
/// doesn't have a project root, but when the server sends the user's demo, it will have to be read
/// from disk.
#[derive(Serialize, Deserialize, Debug)]
pub struct SetDmoMsg {
    pub project_root: Option<PathBuf>,
    pub demo_yml_path: Option<PathBuf>,
    pub dmo_data_json_str: String,
    pub embedded: bool,
}

/// Message to update the content of a specific shader.
#[derive(Serialize, Deserialize, Debug)]
pub struct SetShaderMsg {
    pub idx: usize,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShaderCompilationFailedMsg {
    pub idx: usize,
    pub error_message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShaderCompilationSuccessMsg {
    pub idx: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewProjectMsg {
    pub template: NewProjectTemplate,
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
                        info!("server_actor: Received FetchDmo");
                        let resp;
                        {
                            let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                            resp = Sending {
                                data_type: SetDmo,
                                data: serde_json::to_string(&SetDmoMsg {
                                    project_root: state.project_data.project_root.clone(),
                                    demo_yml_path: state.project_data.demo_yml_path.clone(),
                                    dmo_data_json_str: serde_json::to_string(&state.project_data.dmo_data).unwrap(),
                                    embedded: state.project_data.embedded,
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
                                state.project_data.demo_yml_path = dmo_msg.demo_yml_path.clone();
                                state.project_data.dmo_data = serde_json::from_str(&dmo_msg.dmo_data_json_str).unwrap();

                                self.repeat_message_to_others(&ctx, &message);
                            },

                            Err(e) => {
                                error!{"🔥 Error deserializing Dmo: {:?}", e};
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

                    // Client is setting time. Send it to other clients to record it if they are
                    // tracking time.
                    SetDmoTime => self.repeat_message_to_others(&ctx, &message),

                    // Client is requesting time. Send the message to other clients to respond.
                    GetDmoTime => self.repeat_message_to_others(&ctx, &message),

                    // Client is sending a shader to be updated. Repeat the message to the other
                    // clients such as the OpenGL preview, and update DmoData in the server state.
                    SetShader => {
                        self.repeat_message_to_others(&ctx, &message);

                        match serde_json::from_str::<SetShaderMsg>(&message.data) {
                            Ok(shader_msg) => {
                                let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                                state.project_data.dmo_data.context.shader_sources[shader_msg.idx] = shader_msg.content;
                            },
                            Err(e) => error!("🔥 Error deserializing SetShaderMsg: {:?}", e),
                        }
                    },

                    ShaderCompilationSuccess => self.repeat_message_to_others(&ctx, &message),

                    ShaderCompilationFailed => self.repeat_message_to_others(&ctx, &message),

                    SetSettings => {
                        match serde_json::from_str(&message.data) {
                            Ok(settings) => {
                                info!{"Deserialized Settings"};
                                let mut state = ctx.state().lock().expect("Can't lock ServerState.");
                                state.project_data.dmo_data.settings = settings;

                                let resp = Sending {
                                    data_type: SetSettings,
                                    data: serde_json::to_string(&state.project_data.dmo_data.settings).unwrap(),
                                };
                                self.send_message_to_others(&ctx, &resp);
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

                    SetMetadata => {}

                    ShowErrorMessage => {
                        // Client is sending error message to server.
                        // TODO
                    },

                    StartPreview => {
                        let mut state = ctx.state().lock().expect("Can't lock ServerState.");

                        if let Some(ref mut child) = state.preview_child {

                            match child.try_wait() {
                                Ok(Some(_)) => {
                                    info!("🔎 Spawn a new process for preview.");
                                    let new_child: Option<Child> =
                                        run_preview_command(&state.app_info.path_to_binary);

                                    if new_child.is_some() {
                                        state.preview_child = new_child;
                                    }
                                },

                                Ok(None) => warn!("⚡ Preview process is still running."),

                                Err(e) => error!("🔥 Can't wait for preview child process: {:?}", e),
                            }

                            return;

                        } else {

                            info!("🔎 Spawn a new process for preview.");
                            let new_child: Option<Child> =
                                run_preview_command(&state.app_info.path_to_binary);

                            if new_child.is_some() {
                                state.preview_child = new_child;
                            }

                        }

                    },

                    StopPreview => self.repeat_message_to_others(&ctx, &message),

                    PreviewOpened => self.repeat_message_to_others(&ctx, &message),

                    PreviewClosed => self.repeat_message_to_others(&ctx, &message),

                    StartDialogs => {
                        let mut state = ctx.state().lock().expect("Can't lock ServerState.");

                        if let Some(ref mut child) = state.dialogs_child {

                            match child.try_wait() {
                                Ok(Some(_)) => {
                                    info!("🔎 Spawn a new process for dialogs.");
                                    let new_child: Option<Child> =
                                        run_dialogs_command(&state.app_info.path_to_binary);

                                    if new_child.is_some() {
                                        state.dialogs_child = new_child;
                                    }
                                },

                                Ok(None) => warn!("⚡ Dialogs process is still running."),

                                Err(e) => error!("🔥 Can't wait for dialogs child process: {:?}", e),
                            }

                            return;

                        } else {

                            info!("🔎 Spawn a new process for dialogs.");
                            let new_child: Option<Child> =
                                run_dialogs_command(&state.app_info.path_to_binary);

                            if new_child.is_some() {
                                state.dialogs_child = new_child;
                            }

                        }

                    },

                    OpenProjectFileDialog => self.repeat_message_to_others(&ctx, &message),

                    OpenProjectFilePath => {
                        // Deserialize and sanity check the path. It must point to a YAML file.
                        let yml_path = match serde_json::from_str::<String>(&message.data) {
                            Ok(p) => {
                                let p = PathBuf::from(p);
                                if p.exists() {
                                    if let Some(ext) = p.extension() {
                                        if ext.to_str() != Some("yml") || ext.to_str() != Some("yaml") {
                                            p
                                        } else {
                                            error!{"🔥 Path must be to .yml or .yaml: {:?}", p};
                                            return;
                                        }
                                    } else {
                                        error!{"🔥 Path must be to .yml or .yaml: {:?}", p};
                                        return;
                                    }
                                } else {
                                    error!{"🔥 Path does not exist: {:?}", p};
                                    return;
                                }
                            },

                            Err(e) => {
                                error!("🔥 Deserializing failed: {:?}", e);
                                return;
                            },
                        };

                        // Build a new ProjectData and send it to all clients.

                        let project_data = match ProjectData::new(Some(yml_path), false) {
                            Ok(x) => x,
                            Err(e) => {
                                error!("🔥 Failed to build ProjectData: {:?}", e);
                                return;
                            }
                        };

                        // Send SetDmo
                        let resp = Sending {
                            data_type: SetDmo,
                            data: serde_json::to_string(&SetDmoMsg {
                                project_root: project_data.project_root.clone(),
                                demo_yml_path: project_data.demo_yml_path.clone(),
                                dmo_data_json_str: serde_json::to_string(&project_data.dmo_data).unwrap(),
                                embedded: project_data.embedded,
                            }).unwrap(),
                        };

                        self.send_message_to_everyone(&ctx, &resp);

                        let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                        state.project_data = project_data;
                    },

                    ReloadProject => {
                        let demo_yml_path: Option<PathBuf>;
                        {
                            let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                            demo_yml_path = state.project_data.demo_yml_path.clone();
                        }

                        let project_data = match ProjectData::new(demo_yml_path, false) {
                            Ok(x) => x,
                            Err(e) => {
                                error!("🔥 Failed to build ProjectData: {:?}", e);
                                return;
                            }
                        };

                        // Send SetDmo
                        let resp = Sending {
                            data_type: SetDmo,
                            data: serde_json::to_string(&SetDmoMsg {
                                project_root: project_data.project_root.clone(),
                                demo_yml_path: project_data.demo_yml_path.clone(),
                                dmo_data_json_str: serde_json::to_string(&project_data.dmo_data).unwrap(),
                                embedded: project_data.embedded,
                            }).unwrap(),
                        };

                        self.send_message_to_everyone(&ctx, &resp);

                        let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                        state.project_data = project_data;
                    },

                    SaveProject => {
                        // If demo_yml_path is None, open a dialog to choose the project_root folder, and then:
                        // - write the demo.yml
                        // - write the shaders
                        //
                        // If there is already a path:
                        // - write the shaders

                        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                        match state.project_data.write_shaders() {
                            Ok(_) => {},
                            // TODO show the error in the UI
                            Err(e) => error!{"🔥 Couldn't write shaders: {:?}", e},
                        }
                    },

                    NewProject => {
                        // Starting a new project selects a template and reads its files from the
                        // embedded assets.

                        let template: NewProjectTemplate = match serde_json::from_str::<NewProjectMsg>(&message.data) {
                            Ok(x) => x.template,
                            Err(e) => {
                                error!("🔥 Deserializing failed: {:?}", e);
                                return;
                            },
                        };

                        // Build a new ProjectData and send it to all clients.

                        let project_data = match ProjectData::new_from_embedded_template(template) {
                            Ok(x) => x,
                            Err(e) => {
                                error!("🔥 Failed to build ProjectData: {:?}", e);
                                return;
                            }
                        };

                        // Send SetDmo
                        let resp = Sending {
                            data_type: SetDmo,
                            data: serde_json::to_string(&SetDmoMsg {
                                project_root: project_data.project_root.clone(),
                                demo_yml_path: project_data.demo_yml_path.clone(),
                                dmo_data_json_str: serde_json::to_string(&project_data.dmo_data).unwrap(),
                                embedded: project_data.embedded,
                            }).unwrap(),
                        };

                        self.send_message_to_everyone(&ctx, &resp);

                        let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                        state.project_data = project_data;
                    },

                    ExitApp => {
                        {
                            // Repeat the message for other websocket clients (such as dialogs process and
                            // preview window) to respond to it.
                            self.repeat_message_to_others(&ctx, &message);

                            // The webview is controlled with a channel, not via websocket.
                            let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

                            // Send WebviewExit to the webview window.
                            let webview_sender = state.webview_sender_arc.lock()
                                .expect("Can't lock webview sender.");
                            match webview_sender.send("WebviewExit".to_owned()) {
                                Ok(x) => x,
                                Err(e) => error!("🔥 Can't send WebviewExit on state.webview_sender: {:?}", e),
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

    let bin_cmd = format!("{} preview", path_to_binary.to_str().unwrap());

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
                error!("🔥 failed to spawn preview: {:?}", e);
                return None;
            },
        }
    }
}

fn run_dialogs_command(path_to_binary: &PathBuf) -> Option<Child>
{
    // std::process::Command inherits the current process's working directory.

    let bin_cmd = format!("{} dialogs", path_to_binary.to_str().unwrap());

    if cfg!(target_os = "windows") {

        match Command::new("cmd").arg("/C").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned dialogs");
                return Some(child);
            },
            Err(e) => {
                error!("🔥 failed to spawn dialogs: {:?}", e);
                return None;
            },
        }

    } else {
        // Not testing for `cfg!(target_os = "linux") || cfg!(target_os =
        // "macos")`, try to run some command in any case.

        match Command::new("sh").arg("-c").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned dialogs");
                return Some(child);
            },
            Err(e) => {
                error!("🔥 failed to spawn dialogs: {:?}", e);
                return None;
            },
        }
    }
}
