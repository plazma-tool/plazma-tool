use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, DirEntry, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rand::Rng;

use actix_web::actix::*;
use actix_web::ws;

use crate::app::{AppInfo, AppStartParams};
use crate::project_data::{NewProjectTemplate, ProjectData};
use crate::utils::clean_windows_str_path;

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
    pub app_params: AppStartParams,
    pub project_data: ProjectData,
    pub clients: HashMap<usize, Addr<ServerActor>>,
    pub webview_child: Option<Child>,
    pub nwjs_child: Option<Child>,
    pub preview_child: Option<Child>,
    pub dialogs_child: Option<Child>,
}

pub type ServerStateWrap = Arc<Mutex<ServerState>>;

impl ServerState {
    pub fn new(
        app_info: AppInfo,
        app_params: AppStartParams,
        demo_yml_path: Option<PathBuf>,
    ) -> Result<ServerState, Box<dyn Error>> {
        let state = ServerState {
            app_info,
            app_params,
            project_data: ProjectData::new(demo_yml_path, false)?,
            clients: HashMap::new(),
            webview_child: None,
            nwjs_child: None,
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

impl Default for ServerActor {
    fn default() -> ServerActor {
        ServerActor {
            client_id: rand::thread_rng().gen::<usize>(),
            hb: Instant::now(),
        }
    }
}

impl ServerActor {
    pub fn new() -> ServerActor {
        ServerActor::default()
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

    /// Only the browser UI should request Dmo with this message. Actix clients
    /// should use FetchDmoFile.
    ///
    /// Client is asking for Dmo data. Serialize ServerState.dmo
    /// and send it back.
    fn fetch_dmo_inline(&self, ctx: &mut <ServerActor as Actor>::Context) {
        info!("handle() Received FetchDmoInline");
        let resp;
        {
            let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
            resp = Sending {
                data_type: MsgDataType::SetDmoInline,
                data: serde_json::to_string(&SetDmoMsg {
                    project_root: state.project_data.project_root.clone(),
                    demo_yml_path: state.project_data.demo_yml_path.clone(),
                    dmo_data_json_str: serde_json::to_string(&state.project_data.dmo_data).unwrap(),
                    embedded: state.project_data.embedded,
                })
                .unwrap(),
            };
        }
        let body = serde_json::to_string(&resp).unwrap();
        info!("handle() respond with message length {}", body.len());
        ctx.text(body);
    }

    /// Actix clients should use this to request Dmo data.
    ///
    /// Client is asking for Dmo data. Serialize ServerState.dmo
    /// and send it back.
    fn fetch_dmo_file(&self, ctx: &mut <ServerActor as Actor>::Context, pid: u32) {
        info!("handle() Received FetchDmoFile");
        let resp;
        {
            // NOTE: Don't send SetDmoMsg in data. Write a temp file and send its
            // path. The client is responsible for deleting it after reading.
            //
            // A bug affets large messages, such as when sending SetDmoMsg. When the
            // message body is too large (~100k), the server process dies for some
            // reason. Sending the message succeeds, but only the browser can
            // receive it successfully as client. When the client is an actix
            // process, it dies without even entering the handle() function.

            let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

            let data = serde_json::to_string(&SetDmoMsg {
                project_root: state.project_data.project_root.clone(),
                demo_yml_path: state.project_data.demo_yml_path.clone(),
                dmo_data_json_str: serde_json::to_string(&state.project_data.dmo_data).unwrap(),
                embedded: state.project_data.embedded,
            })
            .unwrap();

            let path = write_data_to_temp(data.as_bytes(), pid).expect("Can't write temp file");

            resp = Sending {
                data_type: MsgDataType::SetDmoFile,
                data: serde_json::to_string(&path).unwrap(),
            };
        }
        let body = serde_json::to_string(&resp).unwrap();
        info!("handle() respond with message length {}", body.len());
        ctx.text(body);
    }

    /// Client is sending Dmo data. Deserialize and replace the
    /// ServerState.dmo. Repeat message to other clients.
    fn set_dmo_inline(&self, ctx: &<ServerActor as Actor>::Context, message: &Receiving) {
        info!("SetDmoInline: received, data length {}", message.data.len());

        match serde_json::from_str::<SetDmoMsg>(&message.data) {
            Ok(dmo_msg) => {
                info! {"Deserialized SetDmoMsg"};
                let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                state.project_data.project_root = dmo_msg.project_root.clone();
                state.project_data.demo_yml_path = dmo_msg.demo_yml_path.clone();
                state.project_data.dmo_data =
                    serde_json::from_str(&dmo_msg.dmo_data_json_str).unwrap();

                self.repeat_message_to_others(&ctx, &message);
            }

            Err(e) => {
                error! {"🔥 Error deserializing Dmo: {:?}", e};
                // Could not deserialize data, tell client to show an error.
                /* TODO: send a message, but not with ctx
                let resp = Sending {
                    data_type: MsgDataType::ShowErrorMessage,
                    data: format! {"{:?}", e},
                };
                let body = serde_json::to_string(&resp).unwrap();
                ctx.text(body);
                */
            }
        }
    }

    /// Client is sending Dmo data. Message data is a file name. Read it,
    /// deserialize and replace the ServerState.dmo.
    ///
    /// Create copies of the file for each client and send them the message.
    /// Each client will delete its message file after reading it.
    fn set_dmo_file(&self, ctx: &<ServerActor as Actor>::Context, message: &Receiving) {
        info!("SetDmoFile: received, data length {}", message.data.len());

        match serde_json::from_str::<PathBuf>(&message.data) {
            Ok(path) => {
                info! {"Deserialized path"};

                let mut file = File::open(&path).unwrap();
                let mut data = String::new();
                file.read_to_string(&mut data).unwrap();
                let dmo_msg: SetDmoMsg = serde_json::from_str(&data).unwrap();

                let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                state.project_data.project_root = dmo_msg.project_root.clone();
                state.project_data.demo_yml_path = dmo_msg.demo_yml_path.clone();
                state.project_data.dmo_data =
                    serde_json::from_str(&dmo_msg.dmo_data_json_str).unwrap();

                self.copy_file_and_send_message_to_others(&ctx, message.data_type, &path);
                match fs::remove_file(&path) {
                    Ok(_) => {}
                    Err(e) => error! {"Can't remove file: {:?}", e},
                };
            }

            Err(e) => {
                error! {"🔥 Error deserializing Dmo: {:?}", e};
                // Could not deserialize data, tell client to show an error.
                /* TODO: send a message, but not with ctx
                let resp = Sending {
                    data_type: MsgDataType::ShowErrorMessage,
                    data: format! {"{:?}", e},
                };
                let body = serde_json::to_string(&resp).unwrap();
                ctx.text(body);
                */
            }
        }
    }

    /// Client is sending a shader to be updated. Repeat the message to the other
    /// clients such as the OpenGL preview, and update DmoData in the server state.
    fn set_shader(&self, ctx: &<ServerActor as Actor>::Context, message: &Receiving) {
        self.repeat_message_to_others(&ctx, &message);

        match serde_json::from_str::<SetShaderMsg>(&message.data) {
            Ok(shader_msg) => {
                let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                state.project_data.dmo_data.context.shader_sources[shader_msg.idx] =
                    shader_msg.content;
            }
            Err(e) => error!("🔥 Error deserializing SetShaderMsg: {:?}", e),
        }
    }

    fn set_settings(&self, ctx: &<ServerActor as Actor>::Context, message: &Receiving) {
        match serde_json::from_str(&message.data) {
            Ok(settings) => {
                info! {"Deserialized Settings"};
                let mut state = ctx.state().lock().expect("Can't lock ServerState.");
                state.project_data.dmo_data.settings = settings;

                let resp = Sending {
                    data_type: MsgDataType::SetSettings,
                    data: serde_json::to_string(&state.project_data.dmo_data.settings).unwrap(),
                };
                self.send_message_to_others(&ctx, &resp);
            }

            Err(e) => {
                error! {"Error deserializing Settings: {:?}", e};
                // Could not deserialize data, tell client to show an error.
                /* TODO: send a message, but not with ctx
                let resp = Sending {
                    data_type: MsgDataType::ShowErrorMessage,
                    data: format! {"{:?}", e},
                };
                let body = serde_json::to_string(&resp).unwrap();
                ctx.text(body);
                */
            }
        }
    }

    fn start_preview(&self, ctx: &<ServerActor as Actor>::Context) {
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
                }

                Ok(None) => warn!("⚡ Preview process is still running."),

                Err(e) => error!("🔥 Can't wait for preview child process: {:?}", e),
            }

            return;
        } else {
            info!("🔎 Spawn a new process for preview.");
            let new_child: Option<Child> = run_preview_command(&state.app_info.path_to_binary);

            if new_child.is_some() {
                state.preview_child = new_child;
            }
        }
    }

    fn start_dialogs(&self, ctx: &<ServerActor as Actor>::Context) {
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
                }

                Ok(None) => warn!("⚡ Dialogs process is still running."),

                Err(e) => error!("🔥 Can't wait for dialogs child process: {:?}", e),
            }

            return;
        } else {
            info!("🔎 Spawn a new process for dialogs.");
            let new_child: Option<Child> = run_dialogs_command(&state.app_info.path_to_binary);

            if new_child.is_some() {
                state.dialogs_child = new_child;
            }
        }
    }

    fn start_webview(&self, ctx: &<ServerActor as Actor>::Context) {
        let mut state = ctx.state().lock().expect("Can't lock ServerState.");

        if let Some(ref mut child) = state.webview_child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    info!("🔎 Spawn a new process for webview.");
                    let new_child: Option<Child> =
                        run_subcommand(&state.app_info.path_to_binary, "webview");

                    if new_child.is_some() {
                        state.webview_child = new_child;
                    }
                }

                Ok(None) => warn!("⚡ Webview process is still running."),

                Err(e) => error!("🔥 Can't wait for webview child process: {:?}", e),
            }

            return;
        } else {
            info!("🔎 Spawn a new process for webview.");
            let new_child: Option<Child> = run_subcommand(&state.app_info.path_to_binary, "webview");

            if new_child.is_some() {
                state.dialogs_child = new_child;
            }
        }
    }

    fn start_nwjs(&self, ctx: &<ServerActor as Actor>::Context) {
        let mut state = ctx.state().lock().expect("Can't lock ServerState.");

        let cmd = format!{"--nwjs_path {} nwjs", state.app_params.nwjs_path.to_str().unwrap()};
        if let Some(ref mut child) = state.nwjs_child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    info!("🔎 Spawn a new process for NWJS.");
                    let new_child: Option<Child> =
                        run_subcommand(&state.app_info.path_to_binary, &cmd);

                    if new_child.is_some() {
                        state.nwjs_child = new_child;
                    }
                }

                Ok(None) => warn!("⚡ NWJS process is still running."),

                Err(e) => error!("🔥 Can't wait for NWJS child process: {:?}", e),
            }

            return;
        } else {
            info!("🔎 Spawn a new process for NWJS.");
            let new_child: Option<Child> =
                run_subcommand(&state.app_info.path_to_binary, &cmd);

            if new_child.is_some() {
                state.dialogs_child = new_child;
            }
        }
    }

    fn open_project_file_path(
        &self,
        ctx: &<ServerActor as Actor>::Context,
        message: &Receiving,
        pid: u32,
    ) {
        info!("OpenProjectFilePath: received");

        // Deserialize and sanity check the path. It must point to a YAML file.
        let yml_path = match serde_json::from_str::<String>(&message.data) {
            Ok(p) => {
                let p = PathBuf::from(p);
                if p.exists() {
                    if let Some(ext) = p.extension() {
                        if ext.to_str() != Some("yml") || ext.to_str() != Some("yaml") {
                            p
                        } else {
                            error! {"🔥 Path must be to .yml or .yaml: {:?}", p};
                            return;
                        }
                    } else {
                        error! {"🔥 Path must be to .yml or .yaml: {:?}", p};
                        return;
                    }
                } else {
                    error! {"🔥 Path does not exist: {:?}", p};
                    return;
                }
            }

            Err(e) => {
                error!("🔥 Deserializing failed: {:?}", e);
                return;
            }
        };

        let project_data = match ProjectData::new(Some(yml_path), false) {
            Ok(x) => x,
            Err(e) => {
                error!("🔥 Failed to build ProjectData: {:?}", e);
                return;
            }
        };

        // Send SetDmoFile
        let data = serde_json::to_string(&SetDmoMsg {
            project_root: project_data.project_root.clone(),
            demo_yml_path: project_data.demo_yml_path.clone(),
            dmo_data_json_str: serde_json::to_string(&project_data.dmo_data).unwrap(),
            embedded: project_data.embedded,
        })
        .unwrap();

        let path = write_data_to_temp(data.as_bytes(), pid).expect("Can't write temp file");

        info!("Send SetDmoFile to everyone");
        // Send to everyone. Usually the dialogs process will send it, which
        // doesn't need the response, but in case the browser sends the message, it
        // will need the updated data.
        self.copy_file_and_send_message_to_everyone(&ctx, MsgDataType::SetDmoFile, &path);
        match fs::remove_file(&path) {
            Ok(_) => {}
            Err(e) => error! {"Can't remove file: {:?}", e},
        };

        let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
        state.project_data = project_data;

        info!("OpenProjectFilePath: done.");
    }

    fn reload_project(&self, ctx: &<ServerActor as Actor>::Context, pid: u32) {
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

        // Send SetDmoFile
        let data = serde_json::to_string(&SetDmoMsg {
            project_root: project_data.project_root.clone(),
            demo_yml_path: project_data.demo_yml_path.clone(),
            dmo_data_json_str: serde_json::to_string(&project_data.dmo_data).unwrap(),
            embedded: project_data.embedded,
        })
        .unwrap();

        let path = write_data_to_temp(data.as_bytes(), pid).expect("Can't write temp file");

        info!("Send SetDmoFile to everyone");
        // Send to everyone, otherwise the browser, which sent the message, will
        // not get the update
        self.copy_file_and_send_message_to_everyone(&ctx, MsgDataType::SetDmoFile, &path);
        match fs::remove_file(&path) {
            Ok(_) => {}
            Err(e) => error! {"Can't remove file: {:?}", e},
        };

        let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
        state.project_data = project_data;
    }

    fn save_project(&self, ctx: &<ServerActor as Actor>::Context) {
        // If demo_yml_path is None, open a dialog to choose the project_root folder, and then:
        // - write the demo.yml
        // - write the shaders
        //
        // If there is already a path:
        // - write the shaders

        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
        match state.project_data.write_shaders() {
            Ok(_) => {}
            // TODO show the error in the UI
            Err(e) => error! {"🔥 Couldn't write shaders: {:?}", e},
        }
    }

    fn new_project(&self, ctx: &<ServerActor as Actor>::Context, message: &Receiving, pid: u32) {
        // Starting a new project selects a template and reads its files from the
        // embedded assets.

        let template: NewProjectTemplate =
            match serde_json::from_str::<NewProjectMsg>(&message.data) {
                Ok(x) => x.template,
                Err(e) => {
                    error!("🔥 Deserializing failed: {:?}", e);
                    return;
                }
            };

        let project_data = match ProjectData::new_from_embedded_template(template) {
            Ok(x) => x,
            Err(e) => {
                error!("🔥 Failed to build ProjectData: {:?}", e);
                return;
            }
        };

        // Send SetDmoFile
        let data = serde_json::to_string(&SetDmoMsg {
            project_root: project_data.project_root.clone(),
            demo_yml_path: project_data.demo_yml_path.clone(),
            dmo_data_json_str: serde_json::to_string(&project_data.dmo_data).unwrap(),
            embedded: project_data.embedded,
        })
        .unwrap();

        let path = write_data_to_temp(data.as_bytes(), pid).expect("Can't write temp file");

        info!("Send SetDmoFile to everyone");
        // Send to everyone, otherwise the browser, which sent the message, will
        // not get the update
        self.copy_file_and_send_message_to_everyone(&ctx, MsgDataType::SetDmoFile, &path);
        match fs::remove_file(&path) {
            Ok(_) => {}
            Err(e) => error! {"Can't remove file: {:?}", e},
        };

        let mut state = ctx.state().lock().expect("👿 Can't lock ServerState.");
        state.project_data = project_data;
    }

    fn delete_message_file(&self, message: &Receiving) {
        match serde_json::from_str::<PathBuf>(&message.data) {
            Ok(path) => {
                match fs::remove_file(&path) {
                    Ok(_) => {}
                    Err(e) => error! {"🔥 Error deleting file: {:?}", e},
                };
            }
            Err(e) => error! {"🔥 Error deserializing path: {:?}", e},
        }
    }

    fn exit_app(&self, ctx: &mut <ServerActor as Actor>::Context, message: &Receiving, pid: u32) {
        info! {"ExitApp"};
        {
            info! {"Cleaning up temp files..."};

            match fs::read_dir(std::env::temp_dir()) {
                Ok(d) => {
                    for entry in d.filter_map(|e| e.ok()).filter(|e| is_plazma_temp(e, pid)) {
                        match std::fs::remove_file(entry.path()) {
                            Ok(_) => info! {"Removed {:?}", entry.path()},
                            Err(e) => error! {"Can't remove: {:?}", e},
                        }
                    }
                }
                Err(e) => error!("Can't read dir: {:?}", e),
            };

            info! {"Repeat ExitApp to other clients"};
            // Repeat the message for other websocket clients (such as dialogs process and
            // preview window) to respond to it.
            self.repeat_message_to_others(&ctx, &message);
        }

        // Stop the Actor, stop the System.
        ctx.stop();
        System::current().stop();
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

    fn copy_file_and_send_message_to_others(
        &self,
        ctx: &<Self as Actor>::Context,
        message_type: MsgDataType,
        path: &PathBuf,
    ) {
        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

        for (id, addr) in &state.clients {
            if *id == self.client_id {
                continue;
            }

            let new_path = get_new_temp_path(state.app_info.pid);
            fs::copy(&path, &new_path).unwrap();

            let resp = Sending {
                data_type: message_type,
                data: serde_json::to_string(&new_path).unwrap(),
            };
            //info!{"💬 Sending {:?} to client {:?}", &message.data_type, id};
            addr.do_send(resp);
        }
    }

    fn copy_file_and_send_message_to_everyone(
        &self,
        ctx: &<Self as Actor>::Context,
        message_type: MsgDataType,
        path: &PathBuf,
    ) {
        let state = ctx.state().lock().expect("👿 Can't lock ServerState.");

        for addr in state.clients.values() {
            let new_path = get_new_temp_path(state.app_info.pid);
            fs::copy(&path, &new_path).unwrap();

            let resp = Sending {
                data_type: message_type,
                data: serde_json::to_string(&new_path).unwrap(),
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
    FetchDmoInline,
    FetchDmoFile,
    SetDmoInline,
    SetDmoFile,
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
    StartWebview,
    StartNwjs,
    OpenProjectFileDialog,
    OpenProjectFilePath,
    ReloadProject,
    SaveProject,
    NewProject,
    DeleteMessageFile,
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
            }

            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }

            ws::Message::Text(text) => {
                /*
                let n = if text.len() < 100 {
                    text.len()
                } else {
                    100
                };
                info!("handle() text message length {}, {}", text.len(), &text[0..n]);
                */

                let message: Receiving = match serde_json::from_str(&text) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("Error on deserializing: {:?}", e);
                        return;
                    }
                };
                //info!{"Received: message.data_type: {:?}", message.data_type};

                let pid: u32;
                {
                    let state = ctx.state().lock().expect("👿 Can't lock ServerState.");
                    pid = state.app_info.pid;
                }

                use self::MsgDataType::*;
                match message.data_type {
                    NoOp => {}

                    FetchDmoInline => self.fetch_dmo_inline(ctx),

                    FetchDmoFile => self.fetch_dmo_file(ctx, pid),

                    SetDmoInline => self.set_dmo_inline(&ctx, &message),

                    SetDmoFile => self.set_dmo_file(&ctx, &message),

                    // Client is setting time. Send it to other clients to record it if they are
                    // tracking time.
                    SetDmoTime => self.repeat_message_to_others(&ctx, &message),

                    // Client is requesting time. Send the message to other clients to respond.
                    GetDmoTime => self.repeat_message_to_others(&ctx, &message),

                    SetShader => self.set_shader(&ctx, &message),

                    ShaderCompilationSuccess => self.repeat_message_to_others(&ctx, &message),

                    ShaderCompilationFailed => self.repeat_message_to_others(&ctx, &message),

                    SetSettings => self.set_settings(&ctx, &message),

                    SetMetadata => {}

                    // Client is sending error message to server.
                    // TODO
                    ShowErrorMessage => {}

                    StartPreview => self.start_preview(&ctx),

                    StopPreview => self.repeat_message_to_others(&ctx, &message),

                    PreviewOpened => self.repeat_message_to_others(&ctx, &message),

                    PreviewClosed => self.repeat_message_to_others(&ctx, &message),

                    StartDialogs => self.start_dialogs(&ctx),

                    StartWebview => self.start_webview(&ctx),

                    StartNwjs => self.start_nwjs(&ctx),

                    OpenProjectFileDialog => self.repeat_message_to_others(&ctx, &message),

                    OpenProjectFilePath => self.open_project_file_path(&ctx, &message, pid),

                    ReloadProject => self.reload_project(&ctx, pid),

                    SaveProject => self.save_project(&ctx),

                    NewProject => self.new_project(&ctx, &message, pid),

                    DeleteMessageFile => self.delete_message_file(&message),

                    ExitApp => self.exit_app(ctx, &message, pid),
                }
            }

            // Echoes back the binary data.
            ws::Message::Binary(bin) => ctx.binary(bin),

            ws::Message::Close(_) => ctx.stop(),
        }
    }
}

fn run_preview_command(path_to_binary: &PathBuf) -> Option<Child> {
    let s = path_to_binary.to_str().unwrap();
    let bin_cmd = if cfg!(target_os = "windows") {
        format!("{} preview", clean_windows_str_path(s))
    } else {
        format!("{} preview", s)
    };

    if cfg!(target_os = "windows") {
        match Command::new("cmd").arg("/C").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned preview");
                Some(child)
            }
            Err(e) => {
                error!("🔥 failed to spawn: {:?}", e);
                None
            }
        }
    } else {
        // Not testing for `cfg!(target_os = "linux") || cfg!(target_os =
        // "macos")`, try to run some command in any case.

        match Command::new("sh").arg("-c").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned preview");
                Some(child)
            }
            Err(e) => {
                error!("🔥 failed to spawn preview: {:?}", e);
                None
            }
        }
    }
}

fn run_dialogs_command(path_to_binary: &PathBuf) -> Option<Child> {
    let s = path_to_binary.to_str().unwrap();
    let bin_cmd = if cfg!(target_os = "windows") {
        format!("{} dialogs", clean_windows_str_path(s))
    } else {
        format!("{} dialogs", s)
    };

    if cfg!(target_os = "windows") {
        match Command::new("cmd").arg("/C").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned dialogs");
                Some(child)
            }
            Err(e) => {
                error!("🔥 failed to spawn dialogs: {:?}", e);
                None
            }
        }
    } else {
        match Command::new("sh").arg("-c").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned dialogs");
                Some(child)
            }
            Err(e) => {
                error!("🔥 failed to spawn dialogs: {:?}", e);
                None
            }
        }
    }
}

fn run_subcommand(path_to_binary: &PathBuf, subcommand: &str) -> Option<Child> {
    let s = path_to_binary.to_str().unwrap();
    let bin_cmd = if cfg!(target_os = "windows") {
        format!("{} {}", clean_windows_str_path(s), subcommand)
    } else {
        format!("{} {}", s, subcommand)
    };

    if cfg!(target_os = "windows") {
        match Command::new("cmd").arg("/C").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned {}", subcommand);
                Some(child)
            }
            Err(e) => {
                error!("🔥 failed to spawn {}: {:?}", subcommand, e);
                None
            }
        }
    } else {
        match Command::new("sh").arg("-c").arg(bin_cmd).spawn() {
            Ok(child) => {
                info!("🔎 spawned {}", subcommand);
                Some(child)
            }
            Err(e) => {
                error!("🔥 failed to spawn {}: {:?}", subcommand, e);
                None
            }
        }
    }
}

fn get_new_temp_path(pid: u32) -> PathBuf {
    let dir = std::env::temp_dir();
    let mut n = 0;
    let mut path = dir.join(&PathBuf::from(&format! {"plazma-{}-{}.txt", pid, n}));
    while path.exists() {
        n += 1;
        path = dir.join(&PathBuf::from(&format! {"plazma-{}-{}.txt", pid, n}));
    }
    path
}

fn write_data_to_temp(data: &[u8], pid: u32) -> Result<PathBuf, Box<dyn Error>> {
    let path = get_new_temp_path(pid);
    let mut file = File::create(&path)?;
    file.write_all(data)?;
    Ok(path)
}

fn is_plazma_temp(entry: &DirEntry, pid: u32) -> bool {
    let prefix: String = format! {"plazma-{}-", pid};
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(&prefix))
        .unwrap_or(false)
}
