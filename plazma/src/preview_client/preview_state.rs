use std::str;
use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_xml::value::{Content, Element};

use glutin::{ElementState, MouseButton, VirtualKeyCode};

use intro_3d::lib::Vector3;
use intro_runtime::camera::Camera;
use intro_runtime::dmo_gfx::{DmoGfx, Settings};
use intro_runtime::frame_buffer::{BufferKind, FrameBuffer};
use intro_runtime::mouse::MouseButton as Btn;
use intro_runtime::polygon_context::PolygonContext;
use intro_runtime::polygon_scene::{PolygonScene, SceneObject};
use intro_runtime::sync_vars::BuiltIn::*;
use intro_runtime::timeline::{SceneBlock, TimeTrack, Timeline};
use intro_runtime::types::{BufferMapping, PixelFormat, UniformMapping, ValueFloat, ValueVec3};
use intro_runtime::ERR_MSG_LEN;

use rocket_client::SyncClient;
use rocket_sync::{code_to_key, SyncDevice, SyncTrack, TrackKey};

use crate::dmo_data::{DmoData, ProjectData};
use crate::error::ToolError;
use crate::project_data::get_template_asset_string;
use crate::server_actor::SetDmoMsg;
use crate::utils::file_to_string;

pub struct PreviewState {
    pub t_rocket_last_connection_attempt: Instant,
    pub t_frame_start: Instant,
    pub t_delta: Duration,
    pub t_frame_target: Duration,

    pub pressed_keys: [bool; 1024],
    pub explore_mode: bool,

    pub draw_anyway: bool,
    pub should_recompile: bool,
    pub movement_speed: f32,

    pub dmo_gfx: DmoGfx,

    /// The sync track names, stored in YAML or sent by the server with DmoData. We keep a copy
    /// here because we have to send it to Rocket when it connects, which can be later than when we
    /// are processing DmoData to DmoGfx.
    pub track_names: Vec<String>,
    /// Mapping the track names to variable indexes in `dmo_gfx.sync_vars`.
    pub track_name_to_idx: BTreeMap<String, usize>,

    pub project_data: ProjectData,
}

impl PreviewState {
    pub fn new(
        demo_yml_path: Option<PathBuf>,
        window_width: f64,
        window_height: f64,
    ) -> Result<PreviewState, Box<dyn Error>> {
        // NOTE screen width and height value has to be the same as window width and height when
        // starting, otherwise the quads that cover the screen will sample framebuffers according
        // to a different aspect ratio and resulting in the image being streched as passed on from
        // one quad pass to another.

        let mut state = PreviewState {
            t_rocket_last_connection_attempt: Instant::now(),
            t_frame_start: Instant::now(),
            t_delta: Duration::new(0, 0),
            t_frame_target: Duration::from_millis(16),

            pressed_keys: [false; 1024],
            explore_mode: false,

            draw_anyway: false,
            should_recompile: false,
            movement_speed: 0.5,

            dmo_gfx: DmoGfx::new_with_dimensions(
                window_width,
                window_height,
                window_width,
                window_height,
                None,
            ),

            track_names: Vec::new(),
            track_name_to_idx: BTreeMap::new(),

            project_data: ProjectData::new(demo_yml_path)?,
        };

        if let Some(ref yml_path) = state.project_data.demo_yml_path {
            info!(
                "PreviewState::new() with yml_path {:?} and build_dmo_gfx_from_yml_str()",
                &yml_path
            );
            let text: String = file_to_string(&yml_path)?;
            state.build_dmo_gfx_from_yml_str(
                &text,
                true,
                true,
                window_width,
                window_height,
                None,
                false,
            )?;
        } else {
            info!("PreviewState::new() with build_dmo_gfx_minimal()");
            // Start with a minimal demo until we receive update from the server. This will be compiled
            // into the binary, so no reading from the disk is needed to open the preview window.
            state.build_dmo_gfx_minimal(window_width, window_height)?;
        }

        Ok(state)
    }

    pub fn build_rocket_connection(
        &mut self,
        rocket: &mut Option<SyncClient>,
    ) -> Result<(), Box<dyn Error>> {
        *rocket = match SyncClient::new("localhost:1338") {
            Ok(x) => Some(x),
            Err(_) => None,
        };

        // If Rocket is on, send the track names.
        //
        // NOTE There is no way to send Rocket the keys. The keys have to be loaded from the XML
        // file using the Rocket editor, and the editor is going to send us the SetKey cmd.

        if let &mut Some(ref mut r) = rocket {
            r.send_track_names(self.get_track_names()).unwrap();
        }

        Ok(())
    }

    pub fn build_dmo_gfx_from_dmo_data(
        &mut self,
        dmo_data: &DmoData,
        window_width: f64,
        window_height: f64,
        camera: Option<Camera>,
        embedded: bool,
    ) -> Result<(), Box<dyn Error>> {
        // NOTE Must use window size for screen size as well
        //
        // NOTE The original aspect when first created has to be preserved, so
        // passing only the size of the window when it was first created.
        let mut dmo_gfx: DmoGfx = DmoGfx::new_with_dimensions(
            window_width,
            window_height,
            window_width,
            window_height,
            camera,
        );

        let (track_names, track_name_to_idx) = build_track_names(
            &mut dmo_gfx,
            &dmo_data,
            &self.project_data.project_root,
            embedded,
        )?;

        build_shader_sources(&mut dmo_gfx, &dmo_data);
        build_image_sources(&mut dmo_gfx, &dmo_data);
        build_settings(&mut dmo_gfx, &dmo_data);
        build_frame_buffers(&mut dmo_gfx, &dmo_data)?;
        // FIXME process ShaderCompilationFailed
        build_quad_scenes(&mut dmo_gfx, &dmo_data, &track_name_to_idx)?;
        // FIXME process ShaderCompilationFailed
        build_polygon_context_and_scenes(
            &mut dmo_gfx,
            &dmo_data,
            &track_name_to_idx,
            &self.project_data.project_root,
            embedded,
        )?;
        build_timeline(&mut dmo_gfx, &dmo_data)?;

        self.track_names = track_names;
        self.track_name_to_idx = track_name_to_idx;
        self.dmo_gfx = dmo_gfx;

        self.should_recompile = true;
        self.draw_anyway = true;

        Ok(())
    }

    pub fn build_dmo_gfx_from_yml_str(
        &mut self,
        yml_str: &str,
        read_shader_paths: bool,
        read_image_paths: bool,
        window_width: f64,
        window_height: f64,
        camera: Option<Camera>,
        embedded: bool,
    ) -> Result<(), Box<dyn Error>> {
        let dmo_data: DmoData = DmoData::new_from_yml_str(
            &yml_str,
            &self.project_data.project_root,
            read_shader_paths,
            read_image_paths,
            embedded,
        )?;
        self.build_dmo_gfx_from_dmo_data(&dmo_data, window_width, window_height, camera, embedded)?;
        Ok(())
    }

    pub fn build_dmo_gfx_from_dmo_msg(
        &mut self,
        msg: &SetDmoMsg,
        read_shader_paths: bool,
        read_image_paths: bool,
        window_width: f64,
        window_height: f64,
        camera: Option<Camera>,
    ) -> Result<(), Box<dyn Error>> {
        self.project_data.project_root = msg.project_root.clone();
        let dmo_data: DmoData = DmoData::new_from_json_str(
            &msg.dmo_data_json_str,
            &self.project_data.project_root,
            read_shader_paths,
            read_image_paths,
            msg.embedded,
        )?;
        self.build_dmo_gfx_from_dmo_data(
            &dmo_data,
            window_width,
            window_height,
            camera,
            msg.embedded,
        )?;
        Ok(())
    }

    pub fn build_dmo_gfx_minimal(
        &mut self,
        window_width: f64,
        window_height: f64,
    ) -> Result<(), Box<dyn Error>> {
        let dmo_data: DmoData = DmoData::new_minimal()?;
        self.build_dmo_gfx_from_dmo_data(&dmo_data, window_width, window_height, None, false)?;
        Ok(())
    }

    pub fn recompile_dmo(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.should_recompile {
            return Ok(());
        }

        // Compile shaders of all QuadScenes

        let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];

        for scene_idx in 0..self.dmo_gfx.context.quad_scenes.len() {
            match self
                .dmo_gfx
                .compile_quad_scene(scene_idx as usize, &mut err_msg_buf)
            {
                Ok(_) => {}
                Err(e) => {
                    let msg = String::from_utf8(err_msg_buf.to_vec())?;
                    return Err(Box::new(ToolError::Runtime(e, msg)));
                }
            }
        }

        // Compile shaders of all Meshes of all Models

        for model_idx in 0..self.dmo_gfx.context.polygon_context.models.len() {
            match self
                .dmo_gfx
                .compile_model_shaders(model_idx, &mut err_msg_buf)
            {
                Ok(_) => {}
                Err(e) => {
                    let msg = String::from_utf8(err_msg_buf.to_vec())?;
                    return Err(Box::new(ToolError::Runtime(e, msg)));
                }
            }
        }

        info!("🎀 Shaders recompiled");
        self.should_recompile = false;
        // TODO draw_anyway needed?
        //self.draw_anyway = true;

        Ok(())
    }

    pub fn set_shader(&mut self, shader_idx: usize, content: &str) -> Result<(), ToolError> {
        // save a copy of the current shader to restore it if the new shader errors
        let prev_content = match self.dmo_gfx.get_shader_src(shader_idx) {
            Ok(x) => x,
            Err(e) => return Err(ToolError::Runtime(e, "".to_owned())),
        };

        match self.dmo_gfx.update_shader_src(shader_idx, content) {
            Ok(_) => {}
            Err(e) => return Err(ToolError::Runtime(e, "".to_owned())),
        };

        // Recompile quad scenes which use this shader.

        // Collect the indexes which use it and recompile.
        let a = self
            .dmo_gfx
            .context
            .quad_scenes
            .iter()
            .enumerate()
            .filter(|i| {
                let (_, scene) = i;
                return scene.vert_src_idx == shader_idx || scene.frag_src_idx == shader_idx;
            })
            .map(|i| {
                let (idx, _) = i;
                return idx;
            })
            .collect::<Vec<usize>>();

        // Iterate over those quad scenes.
        for scene_idx in a.iter() {
            let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];
            match self
                .dmo_gfx
                .compile_quad_scene(*scene_idx, &mut err_msg_buf)
            {
                Ok(_) => {}
                Err(e) => {
                    // restore the previous shader
                    match self
                        .dmo_gfx
                        .update_shader_src_from_vec(shader_idx, &prev_content)
                    {
                        Ok(_) => {}
                        Err(e) => return Err(ToolError::Runtime(e, "".to_owned())),
                    };

                    // send error message
                    let msg = match String::from_utf8(err_msg_buf.to_vec()) {
                        Ok(x) => x,
                        Err(e) => return Err(ToolError::FromUtf8(e)),
                    };
                    return Err(ToolError::Runtime(e, msg));
                }
            }
        }

        // Recompile meshes which use this shader.

        // Collect the indexes of models which has a mesh which uses it.
        let a = self
            .dmo_gfx
            .context
            .polygon_context
            .models
            .iter()
            .enumerate()
            .filter(|i| {
                let (_, model) = i;
                let mut is_using = false;
                for mesh in model.meshes.iter() {
                    if mesh.vert_src_idx == shader_idx || mesh.frag_src_idx == shader_idx {
                        is_using = true;
                    }
                }
                is_using
            })
            .map(|i| {
                let (idx, _) = i;
                return idx;
            })
            .collect::<Vec<usize>>();

        // Iterate over those models.
        for model_idx in a.iter() {
            let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];
            match self
                .dmo_gfx
                .compile_model_shaders(*model_idx, &mut err_msg_buf)
            {
                Ok(_) => {}
                Err(e) => {
                    // restore the previous shader
                    match self
                        .dmo_gfx
                        .update_shader_src_from_vec(shader_idx, &prev_content)
                    {
                        Ok(_) => {}
                        Err(e) => return Err(ToolError::Runtime(e, "".to_owned())),
                    };

                    // send error message
                    let msg = match String::from_utf8(err_msg_buf.to_vec()) {
                        Ok(x) => x,
                        Err(e) => return Err(ToolError::FromUtf8(e)),
                    };
                    return Err(ToolError::Runtime(e, msg));
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self) {
        self.dmo_gfx.draw();
    }

    pub fn update_time_frame_start(&mut self) {
        self.t_frame_start = Instant::now();
        self.dmo_gfx.update_time_frame_start(self.t_frame_start);

        if !self.get_is_paused() {
            let d = self.get_sync_device_mut();
            d.time += 16; // 1s / 60 frames
            d.set_row_from_time();
        }
    }

    pub fn set_time(&mut self, time: f64) {
        self.dmo_gfx.sync.device.time = (time * 1000.0) as u32;
        self.dmo_gfx.sync.device.set_row_from_time();
        // sets the sync var
        self.dmo_gfx.context.set_time(time);
    }

    pub fn update_time_frame_end(&mut self) {
        self.dmo_gfx.update_time_frame_end(Instant::now());
    }

    pub fn move_time_ms(&mut self, ms: i32) {
        self.draw_anyway = true;

        let d = self.get_sync_device_mut();
        if ms > 0 {
            d.time += ms as u32;
        } else if d.time > ((ms * -1) as u32) {
            d.time -= (ms * -1) as u32;
        } else {
            d.time = 0;
        }
        d.set_row_from_time();
    }

    pub fn update_rocket(&mut self, rocket: &mut Option<SyncClient>) -> Result<(), Box<dyn Error>> {
        let mut do_rocket_none = false;
        if let &mut Some(ref mut r) = rocket {
            match r.update(self.get_sync_device_mut()) {
                Ok(a) => self.draw_anyway = a,
                Err(err) => {
                    do_rocket_none = true;
                    // It's a Box<dyn Error>, so we can't restore the original type.
                    // Let's parse the debug string for now.
                    let msg: &str = &format!("{:?}", err);
                    if msg.contains("kind: UnexpectedEof") {
                        warn!("Rocket disconnected");
                    } else {
                        error!("{}", msg);
                    }
                }
            }
        }

        if do_rocket_none {
            *rocket = None;
        }

        // Try to re-connect to Rocket. Good in the case when the Rocket Editor
        // was started after the tool.
        if rocket.is_none()
            && self.t_rocket_last_connection_attempt.elapsed() > Duration::from_secs(1)
        {
            *rocket = match SyncClient::new("localhost:1338") {
                Ok(r) => Some(r),
                Err(_) => None,
            };

            // If Rocket is on, send the track names.
            if let &mut Some(ref mut r) = rocket {
                r.send_track_names(self.get_track_names()).unwrap();
            }

            if rocket.is_some() {
                self.set_is_paused(true);
            }

            self.t_rocket_last_connection_attempt = Instant::now();
        }

        if !self.get_is_paused() {
            if let &mut Some(ref mut r) = rocket {
                match r.send_row(self.get_sync_device_mut()) {
                    Ok(_) => {}
                    Err(e) => warn!("{:?}", e),
                }
            }
        }

        Ok(())
    }

    pub fn update_vars(&mut self) -> Result<(), Box<dyn Error>> {
        match self.dmo_gfx.update_vars() {
            Ok(_) => {}
            Err(e) => return Err(Box::new(ToolError::Runtime(e, "".to_owned()))),
        }
        Ok(())
    }

    pub fn callback_window_resized(&mut self, wx: f64, wy: f64) -> Result<(), Box<dyn Error>> {
        info! {"wx: {}, wy: {}", wx, wy};

        self.dmo_gfx.context.set_window_resolution(wx, wy);
        match self.dmo_gfx.recreate_framebuffers() {
            Ok(_) => {}
            Err(e) => return Err(From::from(format!("{:?}", e))),
        };
        self.draw_anyway = true;

        let window_aspect = wx as f32 / wy as f32;
        self.dmo_gfx.context.camera.aspect = window_aspect;
        self.dmo_gfx.context.camera.update_projection();

        Ok(())
    }

    pub fn callback_mouse_moved(&mut self, mouse_x: i32, mouse_y: i32) {
        // Translating upper-left coords (mouse logic) to lower-left coords (OpenGL logic).
        let (_, wy) = self.get_window_resolution();
        self.dmo_gfx
            .context
            .mouse
            .update_mouse_moved(mouse_x, (wy as i32) - mouse_y);

        if self.explore_mode && self.dmo_gfx.context.mouse.pressed[0] {
            self.dmo_gfx
                .context
                .camera
                .do_pitch_and_yaw_from_mouse_delta(
                    self.dmo_gfx.context.mouse.delta_x,
                    self.dmo_gfx.context.mouse.delta_y,
                );
            self.draw_anyway = true;
        }
    }

    pub fn callback_mouse_input(&mut self, pressed_state: ElementState, button: MouseButton) {
        let pressed = match pressed_state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        };
        let btn = match button {
            MouseButton::Left => Btn::Left,
            MouseButton::Right => Btn::Right,
            MouseButton::Middle => Btn::Middle,
            _ => Btn::NoButton,
        };
        self.dmo_gfx.context.mouse.update_mouse_input(pressed, btn);
    }

    pub fn callback_mouse_wheel(&mut self, dy: f32) {
        if !self.explore_mode {
            return;
        }

        if self.dmo_gfx.context.camera.fovy_angle >= 1.0
            && self.dmo_gfx.context.camera.fovy_angle <= 45.0
        {
            self.dmo_gfx.context.camera.fovy_angle -= dy as f32;
            self.draw_anyway = true;
        }
        if self.dmo_gfx.context.camera.fovy_angle < 1.0 {
            self.dmo_gfx.context.camera.fovy_angle = 1.0;
            self.draw_anyway = true;
        }
        if self.dmo_gfx.context.camera.fovy_angle > 45.0 {
            self.dmo_gfx.context.camera.fovy_angle = 45.0;
            self.draw_anyway = true;
        }

        self.dmo_gfx.context.camera.update_projection();
    }

    // -- get and set --

    pub fn get_is_running(&self) -> bool {
        self.dmo_gfx.context.is_running
    }

    pub fn set_is_running(&mut self, value: bool) {
        self.dmo_gfx.context.is_running = value;
    }

    pub fn get_is_paused(&self) -> bool {
        self.dmo_gfx.sync.device.is_paused
    }

    pub fn get_time(&self) -> f64 {
        self.dmo_gfx.sync.device.time as f64 / 1000.0
    }

    pub fn get_sync_time(&self) -> u32 {
        self.dmo_gfx.sync.device.time
    }

    pub fn set_sync_time(&mut self, time: u32) {
        self.dmo_gfx.sync.device.time = time;
    }

    pub fn set_sync_row_from_time(&mut self) {
        self.dmo_gfx.sync.device.set_row_from_time();
    }

    pub fn set_is_paused(&mut self, value: bool) {
        self.dmo_gfx.sync.device.is_paused = value;
    }

    pub fn get_sync_device(&self) -> &SyncDevice {
        &self.dmo_gfx.sync.device
    }

    pub fn get_sync_device_mut(&mut self) -> &mut SyncDevice {
        &mut self.dmo_gfx.sync.device
    }

    pub fn get_track_names(&self) -> &Vec<String> {
        &self.track_names
    }

    pub fn get_window_resolution(&self) -> (f64, f64) {
        self.dmo_gfx.context.get_window_resolution()
    }

    pub fn set_window_resolution(&mut self, wx: f64, wy: f64) {
        self.dmo_gfx.context.set_window_resolution(wx, wy)
    }

    pub fn get_screen_resolution(&self) -> (f64, f64) {
        self.dmo_gfx.context.get_screen_resolution()
    }

    pub fn set_screen_resolution(&mut self, wx: f64, wy: f64) {
        self.dmo_gfx.context.set_screen_resolution(wx, wy)
    }

    pub fn get_t_delta_as_nanos(&self) -> u64 {
        (self.t_delta.as_secs() * 1_000_000_000) + (self.t_delta.subsec_nanos() as u64)
    }

    pub fn get_t_frame_target_as_nanos(&self) -> u64 {
        (self.t_frame_target.as_secs() * 1_000_000_000)
            + (self.t_frame_target.subsec_nanos() as u64)
    }

    pub fn set_key_pressed(&mut self, vcode: VirtualKeyCode, pressed: bool) {
        let n = vcode as usize;
        if n < self.pressed_keys.len() {
            self.pressed_keys[n] = pressed;
        }
    }

    pub fn toggle_paused(&mut self) {
        self.dmo_gfx.sync.device.is_paused = !self.dmo_gfx.sync.device.is_paused;
    }

    pub fn set_camera_from_context(&mut self) {
        // FIXME set pitch and yaw from the front vector. This is what causes
        // the angle jump when explore mode takes over camera control.

        let a = self.get_context_camera_position();
        self.dmo_gfx.context.camera.set_position(a);
        let a = self.get_context_camera_front();
        self.dmo_gfx.context.camera.set_front(a);
    }

    pub fn get_context_camera_position(&self) -> Vector3 {
        let position: Vector3 = Vector3::new(
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Pos_X) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Pos_Y) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Pos_Z) as f32,
        );
        position
    }

    pub fn get_context_camera_front(&self) -> Vector3 {
        let front: Vector3 = Vector3::new(
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_X) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_Y) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_Z) as f32,
        );
        front
    }

    pub fn update_camera_from_keys(&mut self) {
        if !self.explore_mode {
            return;
        }

        if self.pressed_keys[VirtualKeyCode::W as usize] {
            self.dmo_gfx
                .context
                .camera
                .move_forward(self.movement_speed);
            self.draw_anyway = true;
        }
        if self.pressed_keys[VirtualKeyCode::S as usize] {
            self.dmo_gfx
                .context
                .camera
                .move_backward(self.movement_speed);
            self.draw_anyway = true;
        }
        if self.pressed_keys[VirtualKeyCode::A as usize] {
            self.dmo_gfx.context.camera.move_left(self.movement_speed);
            self.draw_anyway = true;
        }
        if self.pressed_keys[VirtualKeyCode::D as usize] {
            self.dmo_gfx.context.camera.move_right(self.movement_speed);
            self.draw_anyway = true;
        }
        self.dmo_gfx.context.camera.update_view();
    }
}

fn builtin_to_idx(
    track_name_to_idx: &BTreeMap<String, usize>,
    name: &crate::dmo_data::BuiltIn,
) -> Result<usize, Box<dyn Error>> {
    use crate::dmo_data::BuiltIn::*;
    match name {
        Time => Ok(0),

        Window_Width => Ok(1),
        Window_Height => Ok(2),

        Screen_Width => Ok(3),
        Screen_Height => Ok(4),

        Camera_Pos_X => Ok(5),
        Camera_Pos_Y => Ok(6),
        Camera_Pos_Z => Ok(7),

        Camera_Front_X => Ok(8),
        Camera_Front_Y => Ok(9),
        Camera_Front_Z => Ok(10),

        Camera_Up_X => Ok(11),
        Camera_Up_Y => Ok(12),
        Camera_Up_Z => Ok(13),

        Camera_LookAt_X => Ok(14),
        Camera_LookAt_Y => Ok(15),
        Camera_LookAt_Z => Ok(16),

        Fovy => Ok(17),
        Znear => Ok(18),
        Zfar => Ok(19),

        Light_Pos_X => Ok(20),
        Light_Pos_Y => Ok(21),
        Light_Pos_Z => Ok(22),

        Light_Dir_X => Ok(23),
        Light_Dir_Y => Ok(24),
        Light_Dir_Z => Ok(25),

        Light_Strength => Ok(26),
        Light_Constant_Falloff => Ok(27),
        Light_Linear_Falloff => Ok(28),
        Light_Quadratic_Falloff => Ok(29),
        Light_Cutoff_Angle => Ok(30),

        Custom(name) => match track_name_to_idx.get(name) {
            Some(n) => Ok(*n),
            None => Err(From::from(format! {"Track not found: {}", name})),
        },
    }
}

fn build_track_names(
    dmo_gfx: &mut DmoGfx,
    dmo_data: &DmoData,
    project_root: &Option<PathBuf>,
    embedded: bool,
) -> Result<(Vec<String>, BTreeMap<String, usize>), Box<dyn Error>> {
    // Build the track names and their sync var indexes.

    // First, add the names of the builtin tracks and record their indexes.

    // TODO produce this list from the sync mod where builtins are known
    let builtin_names: Vec<String> = vec![
        "Time".to_owned(),
        "Window_Width".to_owned(),
        "Window_Height".to_owned(),
        "Screen_Width".to_owned(),
        "Screen_Height".to_owned(),
        "Camera_Pos_X".to_owned(),
        "Camera_Pos_Y".to_owned(),
        "Camera_Pos_Z".to_owned(),
        "Camera_Front_X".to_owned(),
        "Camera_Front_Y".to_owned(),
        "Camera_Front_Z".to_owned(),
        "Camera_Up_X".to_owned(),
        "Camera_Up_Y".to_owned(),
        "Camera_Up_Z".to_owned(),
        "Camera_LookAt_X".to_owned(),
        "Camera_LookAt_Y".to_owned(),
        "Camera_LookAt_Z".to_owned(),
        "Fovy".to_owned(),
        "Znear".to_owned(),
        "Zfar".to_owned(),
        "Light_Pos_X".to_owned(),
        "Light_Pos_Y".to_owned(),
        "Light_Pos_Z".to_owned(),
        "Light_Dir_X".to_owned(),
        "Light_Dir_Y".to_owned(),
        "Light_Dir_Z".to_owned(),
        "Light_Strength".to_owned(),
        "Light_Constant_Falloff".to_owned(),
        "Light_Linear_Falloff".to_owned(),
        "Light_Quadratic_Falloff".to_owned(),
        "Light_Cutoff_Angle".to_owned(),
    ];

    let mut track_names: Vec<String> = Vec::new();
    let mut track_name_to_idx: BTreeMap<String, usize> = BTreeMap::new();

    for (idx, name) in builtin_names.iter().enumerate() {
        track_names.push(name.clone());
        track_name_to_idx.insert(name.clone(), idx);
    }

    // Then add the list of custom track names, defined in the rocket xml file, the path of
    // which the user has defined in the demo YAML.

    // Read the Rocket XML and add tracks.

    let text = if dmo_data.context.sync_tracks_path.len() > 0 {
        if let Some(p) = project_root {
            let p = p.join(PathBuf::from(&dmo_data.context.sync_tracks_path));
            if embedded {
                get_template_asset_string(&p)?
            } else {
                file_to_string(&p)?
            }
        } else {
            return Err(Box::new(ToolError::MissingProjectRoot));
        }
    } else {
        String::from(EMPTY_ROCKET)
    };

    let tracks_data: Element = serde_xml::from_str(&text)?;
    let bpm: f64;
    let rpb: u8;
    let tracks: Vec<Element>;

    match tracks_data.members {
        Content::Members(ref x) => {
            let tt = x.get("tracks").ok_or("missing 'tracks'")?;
            let ref e: Element = tt[0];

            let tt = e
                .attributes
                .get("beatsPerMin")
                .ok_or("missing 'beatsPerMin'")?;
            let bpm_s = tt[0].to_owned();
            bpm = bpm_s.parse()?;

            let tt = e
                .attributes
                .get("rowsPerBeat")
                .ok_or("missing 'rowsPerBeat'")?;
            let rpb_s = tt[0].to_owned();
            rpb = rpb_s.parse()?;

            match e.members {
                Content::Members(ref x) => {
                    let a = x.get("track").ok_or("missing 'track'")?;
                    tracks = a.to_vec();
                }
                _ => return Err(From::from("no members in 'track'")),
            }
        }
        _ => return Err(From::from("no members in 'tracks'")),
    }

    // new sync device
    let mut sync_device = SyncDevice::new(bpm, rpb);

    // add tracks and keys
    for t in tracks.iter() {
        let mut sync_track: SyncTrack = SyncTrack::new();

        match t.members {
            Content::Members(ref track) => {
                let keys = track.get("key").ok_or("missing 'key'")?;

                for k in keys.iter() {
                    let a = k.attributes.get("row").ok_or("missing 'row'")?;
                    let row: u32 = a[0].parse()?;

                    let a = k.attributes.get("value").ok_or("missing 'value'")?;
                    let value: f32 = a[0].parse()?;

                    let a = k
                        .attributes
                        .get("interpolation")
                        .ok_or("missing 'interpolation'")?;
                    let key_type: u8 = a[0].parse()?;

                    let key = TrackKey {
                        row: row,
                        value: value,
                        key_type: code_to_key(key_type),
                    };

                    sync_track.add_key(key);
                }
            }
            _ => {}
        }

        sync_device.tracks.push(sync_track);
    }

    // FIXME This only works without a start_idx when the Rocket XML already contains the builtin
    // tracks. Recognize the case when these are missing and use start_idx = track_names.len().

    // add track names to list and index
    for (idx, track) in tracks.iter().enumerate() {
        let n = track.attributes.get("name").ok_or("missing 'name'")?;
        track_names.push(n[0].clone());
        track_name_to_idx.insert(n[0].clone(), idx);
    }

    // Assign the new product
    dmo_gfx.sync.device = sync_device;

    // Make sure there are as many sync vars as track names.
    dmo_gfx
        .context
        .sync_vars
        .add_tracks_up_to(track_names.len());

    Ok((track_names, track_name_to_idx))
}

fn build_shader_sources(dmo_gfx: &mut DmoGfx, dmo_data: &DmoData) {
    for i in dmo_data.context.shader_sources.iter() {
        dmo_gfx
            .context
            .shader_sources
            .push(i.as_bytes().to_vec());
    }
}

fn build_image_sources(dmo_gfx: &mut DmoGfx, dmo_data: &DmoData) {
    use crate::dmo_data::context_data as d;
    use intro_runtime::types as r;

    for i in dmo_data.context.image_sources.iter() {
        let format = match i.format {
            d::PixelFormat::NOOP => r::PixelFormat::NOOP,
            d::PixelFormat::RED_u8 => r::PixelFormat::RED_u8,
            d::PixelFormat::RGB_u8 => r::PixelFormat::RGB_u8,
            d::PixelFormat::RGBA_u8 => r::PixelFormat::RGBA_u8,
        };

        let mut image_gfx = r::Image {
            width: i.width,
            height: i.height,
            format: format,
            raw_pixels: Vec::new(),
        };

        for x in i.raw_pixels.iter() {
            image_gfx.raw_pixels.push(*x);
        }

        dmo_gfx.context.images.push(image_gfx);
    }
}

fn build_settings(dmo_gfx: &mut DmoGfx, dmo_data: &DmoData) {
    let settings = Settings {
        start_full_screen: dmo_data.settings.start_full_screen,
        audio_play_on_start: dmo_data.settings.audio_play_on_start,
        mouse_sensitivity: dmo_data.settings.mouse_sensitivity,
        movement_sensitivity: dmo_data.settings.movement_sensitivity,
        total_length: dmo_data.settings.total_length,
    };
    dmo_gfx.settings = settings;
}

fn build_frame_buffers(dmo_gfx: &mut DmoGfx, dmo_data: &DmoData) -> Result<(), Box<dyn Error>> {
    use crate::dmo_data::context_data as d;

    let mut frame_buffers: Vec<FrameBuffer> = Vec::new();

    for fb in dmo_data.context.frame_buffers.iter() {
        let mut has_image = false;

        let kind = match fb.kind {
            d::BufferKind::NOOP => BufferKind::NOOP,
            d::BufferKind::Empty_Texture => BufferKind::Empty_Texture,
            d::BufferKind::Image_Texture => {
                has_image = true;
                BufferKind::Image_Texture
            }
        };

        let format = match fb.format {
            d::PixelFormat::NOOP => PixelFormat::NOOP,
            d::PixelFormat::RED_u8 => PixelFormat::RED_u8,
            d::PixelFormat::RGB_u8 => PixelFormat::RGB_u8,
            d::PixelFormat::RGBA_u8 => PixelFormat::RGBA_u8,
        };

        if has_image {
            let image_data_idx = dmo_data.context.index.get_image_index(&fb.image_path)?;
            frame_buffers.push(FrameBuffer::new(kind, format, Some(image_data_idx)));
        } else {
            frame_buffers.push(FrameBuffer::new(kind, format, None));
        }
    }

    dmo_gfx.context.frame_buffers = frame_buffers;

    match dmo_gfx.create_frame_buffers() {
        Ok(_) => {}
        Err(e) => return Err(Box::new(ToolError::Runtime(e, "".to_owned()))),
    }

    Ok(())
}

fn build_quad_scenes(
    dmo_gfx: &mut DmoGfx,
    dmo_data: &DmoData,
    track_name_to_idx: &BTreeMap<String, usize>,
) -> Result<(), Box<dyn Error>> {
    use crate::dmo_data as d;

    dmo_gfx.context.quad_scenes = Vec::new();

    for q in dmo_data.context.quad_scenes.iter() {
        let mut layout_to_vars: Vec<UniformMapping> = Vec::new();
        let mut binding_to_buffers: Vec<BufferMapping> = Vec::new();

        for i in q.layout_to_vars.iter() {
            let a = match i {
                d::UniformMapping::NOOP => UniformMapping::NOOP,

                d::UniformMapping::Float(layout_idx, a) => {
                    UniformMapping::Float(*layout_idx, builtin_to_idx(track_name_to_idx, a)? as u8)
                }

                d::UniformMapping::Vec2(layout_idx, a, b) => UniformMapping::Vec2(
                    *layout_idx,
                    builtin_to_idx(track_name_to_idx, a)? as u8,
                    builtin_to_idx(track_name_to_idx, b)? as u8,
                ),

                d::UniformMapping::Vec3(layout_idx, a, b, c) => UniformMapping::Vec3(
                    *layout_idx,
                    builtin_to_idx(track_name_to_idx, a)? as u8,
                    builtin_to_idx(track_name_to_idx, b)? as u8,
                    builtin_to_idx(track_name_to_idx, c)? as u8,
                ),

                d::UniformMapping::Vec4(layout_idx, a, b, c, d) => UniformMapping::Vec4(
                    *layout_idx,
                    builtin_to_idx(track_name_to_idx, a)? as u8,
                    builtin_to_idx(track_name_to_idx, b)? as u8,
                    builtin_to_idx(track_name_to_idx, c)? as u8,
                    builtin_to_idx(track_name_to_idx, d)? as u8,
                ),
            };
            layout_to_vars.push(a);
        }

        for i in q.binding_to_buffers.iter() {
            let a = match i {
                d::BufferMapping::NOOP => BufferMapping::NOOP,

                d::BufferMapping::Sampler2D(layout_idx, buffer_name) => {
                    let buffer_idx = dmo_data.context.index.get_buffer_index(&buffer_name)?;
                    BufferMapping::Sampler2D(*layout_idx, buffer_idx as u8)
                }
            };
            binding_to_buffers.push(a);
        }

        let vert_src_idx = dmo_data.context.index.get_shader_index(&q.vert_src_path)?;
        let frag_src_idx = dmo_data.context.index.get_shader_index(&q.frag_src_path)?;

        dmo_gfx.context.add_quad_scene(
            vert_src_idx,
            frag_src_idx,
            layout_to_vars,
            binding_to_buffers,
        );
    }

    let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];

    match dmo_gfx.create_quads(&mut err_msg_buf) {
        Ok(_) => {}
        Err(e) => {
            let msg = String::from_utf8(err_msg_buf.to_vec())?;
            return Err(Box::new(ToolError::Runtime(e, msg)));
        }
    }

    Ok(())
}

fn build_polygon_context_and_scenes(
    dmo_gfx: &mut DmoGfx,
    dmo_data: &DmoData,
    track_name_to_idx: &BTreeMap<String, usize>,
    project_root: &Option<PathBuf>,
    embedded: bool,
) -> Result<(), Box<dyn Error>> {
    use crate::dmo_data as d;

    // Create a PolygonContext and add models.

    let aspect = dmo_gfx.context.get_window_aspect();

    // Setting the aspect is enough. Camera view vectors, fovy, etc. will be set in the sync
    // vars by calculating the track values defined in the Rocket XML.

    dmo_gfx.context.camera = Camera::new_defaults(aspect as f32);
    dmo_gfx.context.polygon_context = PolygonContext::new_defaults(aspect as f32);

    dmo_data.add_models_to(dmo_gfx, project_root, embedded)?;

    // Create correcponding OpenGL objects.

    let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];

    match dmo_gfx.create_models(&mut err_msg_buf) {
        Ok(_) => {}
        Err(e) => {
            let msg = String::from_utf8(err_msg_buf.to_vec())?;
            return Err(Box::new(ToolError::Runtime(e, msg)));
        }
    }

    // PolygonContext is ready.

    // Add PolygonScenes.

    for (_idx, scene) in dmo_data.context.polygon_scenes.iter().enumerate() {
        let mut polygon_scene = PolygonScene::default();

        for obj_data in scene.scene_objects.iter() {
            let mut scene_object = SceneObject::default();
            scene_object.model_idx = dmo_data.context.index.get_model_index(&obj_data.name)?;

            scene_object.position_var = match &obj_data.position {
                d::ValueVec3::NOOP => ValueVec3::NOOP,

                d::ValueVec3::Fixed(a, b, c) => ValueVec3::Fixed(*a, *b, *c),

                d::ValueVec3::Sync(a, b, c) => ValueVec3::Sync(
                    builtin_to_idx(track_name_to_idx, &a)? as u8,
                    builtin_to_idx(track_name_to_idx, &b)? as u8,
                    builtin_to_idx(track_name_to_idx, &c)? as u8,
                ),
            };

            scene_object.euler_rotation_var = match &obj_data.euler_rotation {
                d::ValueVec3::NOOP => ValueVec3::NOOP,

                d::ValueVec3::Fixed(a, b, c) => ValueVec3::Fixed(*a, *b, *c),

                d::ValueVec3::Sync(a, b, c) => ValueVec3::Sync(
                    builtin_to_idx(track_name_to_idx, &a)? as u8,
                    builtin_to_idx(track_name_to_idx, &b)? as u8,
                    builtin_to_idx(track_name_to_idx, &c)? as u8,
                ),
            };

            scene_object.scale_var = match &obj_data.scale {
                d::ValueFloat::NOOP => ValueFloat::NOOP,
                d::ValueFloat::Fixed(a) => ValueFloat::Fixed(*a),
                d::ValueFloat::Sync(a) => {
                    ValueFloat::Sync(builtin_to_idx(track_name_to_idx, &a)? as u8)
                }
            };

            for i in obj_data.layout_to_vars.iter() {
                let m = match i {
                    d::UniformMapping::NOOP => UniformMapping::NOOP,

                    d::UniformMapping::Float(x, a) => {
                        UniformMapping::Float(*x, builtin_to_idx(track_name_to_idx, &a)? as u8)
                    }

                    d::UniformMapping::Vec2(x, a, b) => UniformMapping::Vec2(
                        *x,
                        builtin_to_idx(track_name_to_idx, &a)? as u8,
                        builtin_to_idx(track_name_to_idx, &b)? as u8,
                    ),

                    d::UniformMapping::Vec3(x, a, b, c) => UniformMapping::Vec3(
                        *x,
                        builtin_to_idx(track_name_to_idx, &a)? as u8,
                        builtin_to_idx(track_name_to_idx, &b)? as u8,
                        builtin_to_idx(track_name_to_idx, &c)? as u8,
                    ),

                    d::UniformMapping::Vec4(x, a, b, c, d) => UniformMapping::Vec4(
                        *x,
                        builtin_to_idx(track_name_to_idx, &a)? as u8,
                        builtin_to_idx(track_name_to_idx, &b)? as u8,
                        builtin_to_idx(track_name_to_idx, &c)? as u8,
                        builtin_to_idx(track_name_to_idx, &d)? as u8,
                    ),
                };

                scene_object.layout_to_vars.push(m);
            }

            for i in obj_data.binding_to_buffers.iter() {
                let m = match i {
                    d::BufferMapping::NOOP => BufferMapping::NOOP,

                    d::BufferMapping::Sampler2D(layout_idx, name) => {
                        let buffer_idx = dmo_data.context.index.get_buffer_index(&name)?;
                        BufferMapping::Sampler2D(*layout_idx, buffer_idx as u8)
                    }
                };

                scene_object.binding_to_buffers.push(m);
            }

            match dmo_gfx.compile_model_shaders(scene_object.model_idx, &mut err_msg_buf) {
                Ok(_) => {}
                Err(e) => {
                    let msg = String::from_utf8(err_msg_buf.to_vec())?;
                    return Err(Box::new(ToolError::Runtime(e, msg)));
                }
            }

            polygon_scene.scene_objects.push(scene_object);
        }

        dmo_gfx.context.polygon_scenes.push(polygon_scene);
    }

    Ok(())
}

fn build_timeline(dmo_gfx: &mut DmoGfx, dmo_data: &DmoData) -> Result<(), Box<dyn Error>> {
    use crate::dmo_data::timeline::DrawOp as D;
    use intro_runtime::timeline::DrawOp as G;

    dmo_gfx.timeline = Timeline::new();

    for track in dmo_data.timeline.tracks.iter() {
        let mut track_gfx = TimeTrack {
            scene_blocks: Vec::new(),
        };

        for block in track.scene_blocks.iter() {
            let mut ops: Vec<G> = Vec::new();

            for i in block.draw_ops.iter() {
                let o = match i {
                    D::NOOP => G::NOOP,

                    D::Draw_Quad_Scene(name) => {
                        let idx = dmo_data.context.index.get_quad_scene_index(name)?;
                        G::Draw_Quad_Scene(idx)
                    }

                    D::Draw_Poly_Scene(name) => {
                        let idx = dmo_data.context.index.get_polygon_scene_index(name)?;
                        G::Draw_Poly_Scene(idx)
                    }

                    D::Clear(r, g, b, a) => G::Clear(*r, *g, *b, *a),

                    D::Target_Buffer(name) => {
                        let idx = dmo_data.context.index.get_buffer_index(name)?;
                        G::Target_Buffer(idx)
                    }

                    D::Target_Buffer_Default => G::Target_Buffer_Default,

                    D::Profile(name) => {
                        let idx = dmo_data.context.index.get_profile_index(name)?;
                        G::Profile(idx)
                    }
                };

                ops.push(o);
            }

            let block_gfx = SceneBlock {
                start: block.start,
                end: block.end,
                draw_ops: ops,
            };

            track_gfx.scene_blocks.push(block_gfx);
        }

        dmo_gfx.timeline.tracks.push(track_gfx);
    }

    Ok(())
}

const EMPTY_ROCKET: &'static str = r#"
<?xml version="1.0" encoding="utf-8"?>
<rootElement>
<tracks rows="10000" startRow="0" endRow="10000" rowsPerBeat="8" beatsPerMin="125">
        <track name="dummy" folded="0" muteKeyCount="0" color="aabbccdd">
                <key row="0" value="0.000000" interpolation="0" />
        </track>
</tracks>
</rootElement>"#;
