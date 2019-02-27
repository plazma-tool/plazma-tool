use core::str;
use std::error::Error;
use std::time::{Duration, Instant};

use glutin::{VirtualKeyCode, ElementState, MouseButton};

use smallvec::SmallVec;

use intro_3d::Vector3;
use intro_runtime::ERR_MSG_LEN;
use intro_runtime::dmo_gfx::{DmoGfx, Settings};
use intro_runtime::polygon_context::PolygonContext;
use intro_runtime::dmo_sync::SyncDevice;
use intro_runtime::timeline::{Timeline, TimeTrack, SceneBlock};
use intro_runtime::sync_vars::BuiltIn::*;
use intro_runtime::frame_buffer::{FrameBuffer, BufferKind};
use intro_runtime::polygon_scene::{PolygonScene, SceneObject};
use intro_runtime::camera::Camera;
use intro_runtime::mouse::MouseButton as Btn;
use intro_runtime::types::{PixelFormat, ValueVec3, ValueFloat, BufferMapping, UniformMapping};
use intro_runtime::error::RuntimeError;

use crate::dmo_data::DmoData;
use crate::dmo_data::builtin_to_idx;
use crate::error::ToolError;

pub struct PreviewState {
    pub t_frame_start: Instant,
    pub t_delta: Duration,
    pub t_frame_target: Duration,

    pub pressed_keys: [bool; 1024],
    pub explore_mode: bool,

    pub draw_anyway: bool,
    pub should_recompile: bool,
    pub movement_speed: f32,

    pub dmo_gfx: DmoGfx,
}

impl PreviewState {

    pub fn new(window_width: f64, window_height: f64)
        -> Result<PreviewState, Box<Error>>
    {
        // NOTE screen width and height value has to be the same as window width and height when
        // starting, otherwise the quads that cover the screen will sample framebuffers according
        // to a different aspect ratio and resulting in the image being streched as passed on from
        // one quad pass to another.

        let state = PreviewState {
            t_frame_start: Instant::now(),
            t_delta: Duration::new(0, 0),
            t_frame_target: Duration::from_millis(16),

            pressed_keys: [false; 1024],
            explore_mode: false,

            draw_anyway: false,
            should_recompile: false,
            movement_speed: 0.5,

            dmo_gfx: DmoGfx::new_with_dimensions(window_width,
                                                 window_height,
                                                 window_width,
                                                 window_height,
                                                 None),
        };

        Ok(state)
    }

    fn build_settings(dmo_gfx: &mut DmoGfx,
                      dmo_data: &DmoData)
    {
        let settings = Settings {
            start_full_screen: dmo_data.settings.start_full_screen,
            audio_play_on_start: dmo_data.settings.audio_play_on_start,
            mouse_sensitivity: dmo_data.settings.mouse_sensitivity,
            movement_sensitivity: dmo_data.settings.movement_sensitivity,
            total_length: dmo_data.settings.total_length,
        };
        dmo_gfx.settings = settings;
    }

    fn build_frame_buffers(dmo_gfx: &mut DmoGfx,
                           dmo_data: &DmoData)
                           -> Result<(), Box<Error>>
    {
        use crate::dmo_data::context_data as d;

        let mut frame_buffers: SmallVec<[FrameBuffer; 64]> = SmallVec::new();

        for fb in dmo_data.context.frame_buffers.iter() {
            let kind = match fb.kind {
                d::BufferKind::NOOP => BufferKind::NOOP,
                d::BufferKind::Empty_Texture => BufferKind::Empty_Texture,
                d::BufferKind::Image_Texture => BufferKind::Image_Texture,
            };

            let format = match fb.format {
                d::PixelFormat::NOOP => PixelFormat::NOOP,
                d::PixelFormat::RED_u8 => PixelFormat::RED_u8,
                d::PixelFormat::RGB_u8 => PixelFormat::RGB_u8,
                d::PixelFormat::RGBA_u8 => PixelFormat::RGBA_u8,
            };

            frame_buffers.push(FrameBuffer::new(kind, format, None));
        }

        dmo_gfx.context.frame_buffers = frame_buffers;

        match dmo_gfx.create_frame_buffers() {
            Ok(_) => {},
            Err(e) => return Err(Box::new(ToolError::Runtime(e, "".to_owned())))
        }

        Ok(())
    }

    fn build_quad_scenes(dmo_gfx: &mut DmoGfx,
                         dmo_data: &DmoData)
                         -> Result<(), Box<Error>>
    {
        use crate::dmo_data as d;

        dmo_gfx.context.quad_scenes = SmallVec::new();

        for q in dmo_data.context.quad_scenes.iter()
        {
            let mut layout_to_vars: SmallVec<[UniformMapping; 64]> = SmallVec::new();
            let mut binding_to_buffers: SmallVec<[BufferMapping; 64]> = SmallVec::new();

            for i in q.layout_to_vars.iter() {
                let a = match i {
                    d::UniformMapping::NOOP => UniformMapping::NOOP,

                    d::UniformMapping::Float(layout_idx, a) =>
                        UniformMapping::Float(*layout_idx,
                                              d::builtin_to_idx(a) as u8),

                    d::UniformMapping::Vec2(layout_idx, a, b) =>
                        UniformMapping::Vec2(*layout_idx,
                                             d::builtin_to_idx(a) as u8,
                                             d::builtin_to_idx(b) as u8),

                    d::UniformMapping::Vec3(layout_idx, a, b, c) =>
                        UniformMapping::Vec3(*layout_idx,
                                             d::builtin_to_idx(a) as u8,
                                             d::builtin_to_idx(b) as u8,
                                             d::builtin_to_idx(c) as u8),

                    d::UniformMapping::Vec4(layout_idx, a, b, c, d) =>
                        UniformMapping::Vec4(*layout_idx,
                                             d::builtin_to_idx(a) as u8,
                                             d::builtin_to_idx(b) as u8,
                                             d::builtin_to_idx(c) as u8,
                                             d::builtin_to_idx(d) as u8),
                };
                layout_to_vars.push(a);
            }

            for i in q.binding_to_buffers.iter() {
                let a = match i {
                    d::BufferMapping::NOOP => BufferMapping::NOOP,

                    d::BufferMapping::Sampler2D(layout_idx, buffer_name) => {
                        let buffer_idx = dmo_data.context.index.get_buffer_index(&buffer_name)?;
                        BufferMapping::Sampler2D(*layout_idx, buffer_idx as u8)
                    },
                };
                binding_to_buffers.push(a);
            }

            let vert_src_idx = dmo_data.context.index.get_shader_index(&q.vert_src_path)?;
            let frag_src_idx = dmo_data.context.index.get_shader_index(&q.frag_src_path)?;

            dmo_gfx
                .context
                .add_quad_scene(vert_src_idx,
                                frag_src_idx,
                                layout_to_vars,
                                binding_to_buffers);
        }

        let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];

        match dmo_gfx.create_quads(&mut err_msg_buf) {
            Ok(_) => {},
            Err(e) => {
                let msg = String::from_utf8(err_msg_buf.to_vec())?;
                return Err(Box::new(ToolError::Runtime(e, msg)));
            }
        }

        Ok(())
    }

    fn build_shader_sources(dmo_gfx: &mut DmoGfx,
                            dmo_data: &DmoData)
    {
        for i in dmo_data.context.shader_sources.iter() {
            dmo_gfx.context.shader_sources.push(SmallVec::from_slice(i.as_bytes()));
        }
    }

    fn build_polygon_context_and_scenes(dmo_gfx: &mut DmoGfx,
                                        dmo_data: &DmoData)
                                        -> Result<(), Box<Error>>
    {
        use crate::dmo_data as d;

        // Create a PolygonContext and add models.

        let aspect = dmo_gfx.context.get_window_aspect();

        let camera = Camera::new(45.0,
                                 aspect as f32,
                                 Vector3::from_slice(&dmo_data.context.polygon_context.view_position),
                                 Some(Vector3::from_slice(&dmo_data.context.polygon_context.view_front)),
                                 Vector3::from_slice(&dmo_data.context.polygon_context.view_up),
                                 0.0,
                                 90.0);

        dmo_gfx.context.camera = camera;

        dmo_gfx.context.polygon_context = PolygonContext::new(
            Vector3::from_slice(&dmo_data.context.polygon_context.view_position),
            Vector3::from_slice(&dmo_data.context.polygon_context.view_front),
            Vector3::from_slice(&dmo_data.context.polygon_context.view_up),
            dmo_data.context.polygon_context.fovy,
            dmo_data.context.polygon_context.znear,
            dmo_data.context.polygon_context.zfar,
            aspect as f32
        );

        dmo_data.add_models_to(dmo_gfx)?;

        // Create correcponding OpenGL objects.

        let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];

        match dmo_gfx.create_models(&mut err_msg_buf) {
            Ok(_) => {},
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

                    d::ValueVec3::Sync(a, b, c) =>
                        ValueVec3::Sync(builtin_to_idx(&a) as u8,
                                        builtin_to_idx(&b) as u8,
                                        builtin_to_idx(&c) as u8),
                };

                scene_object.euler_rotation_var = match &obj_data.euler_rotation {
                    d::ValueVec3::NOOP => ValueVec3::NOOP,

                    d::ValueVec3::Fixed(a, b, c) => ValueVec3::Fixed(*a, *b, *c),

                    d::ValueVec3::Sync(a, b, c) =>
                        ValueVec3::Sync(builtin_to_idx(&a) as u8,
                                        builtin_to_idx(&b) as u8,
                                        builtin_to_idx(&c) as u8),
                };

                scene_object.scale_var = match &obj_data.scale {
                    d::ValueFloat::NOOP => ValueFloat::NOOP,
                    d::ValueFloat::Fixed(a) => ValueFloat::Fixed(*a),
                    d::ValueFloat::Sync(a) => ValueFloat::Sync(builtin_to_idx(&a) as u8),
                };

                for i in obj_data.layout_to_vars.iter() {
                    let m = match i {
                        d::UniformMapping::NOOP => UniformMapping::NOOP,

                        d::UniformMapping::Float(x, a) =>
                            UniformMapping::Float(*x, builtin_to_idx(&a) as u8),

                        d::UniformMapping::Vec2(x, a, b) =>
                            UniformMapping::Vec2(*x,
                                                 builtin_to_idx(&a) as u8,
                                                 builtin_to_idx(&b) as u8),

                        d::UniformMapping::Vec3(x, a, b, c) =>
                            UniformMapping::Vec3(*x,
                                                 builtin_to_idx(&a) as u8,
                                                 builtin_to_idx(&b) as u8,
                                                 builtin_to_idx(&c) as u8),

                        d::UniformMapping::Vec4(x, a, b, c, d) =>
                            UniformMapping::Vec4(*x,
                                                 builtin_to_idx(&a) as u8,
                                                 builtin_to_idx(&b) as u8,
                                                 builtin_to_idx(&c) as u8,
                                                 builtin_to_idx(&d) as u8),
                    };

                    scene_object.layout_to_vars.push(m);
                }

                for i in obj_data.binding_to_buffers.iter() {
                    let m = match i {

                        d::BufferMapping::NOOP => BufferMapping::NOOP,

                        d::BufferMapping::Sampler2D(layout_idx, name) => {
                            let buffer_idx = dmo_data.context.index.get_buffer_index(&name)?;
                            BufferMapping::Sampler2D(*layout_idx, buffer_idx as u8)
                        },

                    };

                    scene_object.binding_to_buffers.push(m);
                }

                match dmo_gfx.compile_model_shaders(scene_object.model_idx, &mut err_msg_buf) {
                    Ok(_) => {},
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

    fn build_timeline(dmo_gfx: &mut DmoGfx,
                      dmo_data: &DmoData)
                      -> Result<(), Box<Error>>
    {
        use crate::dmo_data::timeline::DrawOp as D;
        use intro_runtime::timeline::DrawOp as G;

        dmo_gfx.timeline = Timeline::new();

        for track in dmo_data.timeline.tracks.iter() {
            let mut track_gfx = TimeTrack {
                scene_blocks: SmallVec::new(),
            };

            for block in track.scene_blocks.iter() {
                let mut ops: SmallVec<[G; 32]> = SmallVec::new();

                for i in block.draw_ops.iter() {
                    let o = match i {
                        D::NOOP => G::NOOP,

                        D::Draw_Quad_Scene(name) => {
                            let idx = dmo_data.context.index.get_quad_scene_index(name)?;
                            G::Draw_Quad_Scene(idx)
                        },

                        D::Draw_Poly_Scene(name) => {
                            let idx = dmo_data.context.index.get_polygon_scene_index(name)?;
                            G::Draw_Poly_Scene(idx)
                        },

                        D::Clear(r, g, b, a) => G::Clear(*r, *g, *b, *a),

                        D::Target_Buffer(name) => {
                            let idx = dmo_data.context.index.get_buffer_index(name)?;
                            G::Target_Buffer(idx)
                        },

                        D::Target_Buffer_Default => G::Target_Buffer_Default,

                        D::Profile(name) => {
                            let idx = dmo_data.context.index.get_profile_index(name)?;
                            G::Profile(idx)
                        },
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

    pub fn build_dmo_gfx_from_yml_str(&mut self,
                                      yml_str: &str,
                                      read_shader_paths: bool,
                                      window_width: f64,
                                      window_height: f64,
                                      screen_width: f64,
                                      screen_height: f64,
                                      camera: Option<Camera>)
        -> Result<(), Box<Error>>
    {
        let dmo_data: DmoData = DmoData::new_from_yml_str(&yml_str, read_shader_paths)?;
        let mut dmo_gfx: DmoGfx = DmoGfx::new_with_dimensions(window_width,
                                                              window_height,
                                                              screen_width,
                                                              screen_height,
                                                              camera);

        PreviewState::build_shader_sources(&mut dmo_gfx, &dmo_data);
        PreviewState::build_settings(&mut dmo_gfx, &dmo_data);
        PreviewState::build_frame_buffers(&mut dmo_gfx, &dmo_data)?;
        PreviewState::build_quad_scenes(&mut dmo_gfx, &dmo_data)?;
        PreviewState::build_polygon_context_and_scenes(&mut dmo_gfx, &dmo_data)?;
        PreviewState::build_timeline(&mut dmo_gfx, &dmo_data)?;

        self.dmo_gfx = dmo_gfx;

        self.should_recompile = true;
        self.draw_anyway = true;

        Ok(())
    }

    // fn add_shader_src(&mut self, path: &PathBuf) -> usize {
    //     let src = file_to_string(path).unwrap();
    //     self.dmo_gfx.context.shader_sources.push(SmallVec::from_slice(src.as_bytes()));
    //     return self.dmo_gfx.context.shader_sources.len() - 1;
    // }

    pub fn recompile_dmo(&mut self) -> Result<(), Box<Error>> {
        if !self.should_recompile {
            return Ok(());
        }

        // Compile shaders of all QuadScenes

        let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];

        for scene_idx in 0..self.dmo_gfx.context.quad_scenes.len() {
            match self.dmo_gfx.compile_quad_scene(scene_idx as usize, &mut err_msg_buf) {
                Ok(_) => {},
                Err(e) => {
                    let msg = String::from_utf8(err_msg_buf.to_vec())?;
                    return Err(Box::new(ToolError::Runtime(e, msg)));
                }
            }
        }

        // Compile shaders of all Meshes of all Models

        for model_idx in 0..self.dmo_gfx.context.polygon_context.models.len() {
            match self.dmo_gfx.compile_model_shaders(model_idx, &mut err_msg_buf) {
                Ok(_) => {},
                Err(e) => {
                    let msg = String::from_utf8(err_msg_buf.to_vec())?;
                    return Err(Box::new(ToolError::Runtime(e, msg)));
                }
            }

        }

        println!("Recompiled");
        self.should_recompile = false;
        // TODO draw_anyway needed?
        //self.draw_anyway = true;

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
            d.time += 16;// 1s / 60 frames
            d.set_row_from_time();
        }
    }

    pub fn update_time_frame_end(&mut self) {
        self.dmo_gfx.update_time_frame_end(Instant::now());
    }

    pub fn update_vars(&mut self) -> Result<(), RuntimeError> {
        // When playing, sync values from Rocket and
        // update the widget values to synced values.
        if !self.get_is_paused() {
            self.dmo_gfx.update_vars()?;
        }

        Ok(())
    }

    pub fn callback_window_resized(&mut self, wx: f64, wy: f64) -> Result<(), Box<Error>> {
        info!{"wx: {}, wy: {}", wx, wy};

        self.dmo_gfx.context.set_window_resolution(wx, wy);
        match self.dmo_gfx.recreate_framebuffers() {
            Ok(_) => {},
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
        self.dmo_gfx.context.mouse.update_mouse_moved(mouse_x, (wy as i32) - mouse_y);

        if self.explore_mode && self.dmo_gfx.context.mouse.pressed[0] {
            self.dmo_gfx.context.camera.do_pitch_and_yaw_from_mouse_delta(
                self.dmo_gfx.context.mouse.delta_x,
                self.dmo_gfx.context.mouse.delta_y
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

        if self.dmo_gfx.context.camera.fovy_angle >= 1.0 && self.dmo_gfx.context.camera.fovy_angle <= 45.0 {
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
        (self.t_frame_target.as_secs() * 1_000_000_000) + (self.t_frame_target.subsec_nanos() as u64)
    }

    pub fn set_key_pressed(&mut self, vcode: VirtualKeyCode, pressed: bool) {
        let n = vcode as usize;
        if n < self.pressed_keys.len() {
            self.pressed_keys[n] = pressed;
        }
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
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Pos_Z) as f32);
        position
    }

    pub fn get_context_camera_front(&self) -> Vector3 {
        let front: Vector3 = Vector3::new(
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_X) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_Y) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_Z) as f32);
        front
    }

    pub fn update_camera_from_keys(&mut self) {
        if !self.explore_mode {
            return;
        }

        if self.pressed_keys[VirtualKeyCode::W as usize] {
            self.dmo_gfx.context.camera.move_forward(self.movement_speed);
            self.draw_anyway = true;
        }
        if self.pressed_keys[VirtualKeyCode::S as usize] {
            self.dmo_gfx.context.camera.move_backward(self.movement_speed);
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
