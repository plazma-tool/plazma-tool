use std::time::{Duration, Instant};

use gl;

use crate::camera::Camera;
use crate::frame_buffer::FrameBuffer;
use crate::mouse::Mouse;
use crate::polygon_context::PolygonContext;
use crate::polygon_scene::PolygonScene;
use crate::quad_scene_gfx::QuadSceneGfx;
use crate::sync_vars::BuiltIn::*;
use crate::sync_vars::SyncVars;
use crate::types::{BufferMapping, Image, UniformMapping};

pub const PROFILE_FRAMES: usize = 60;
pub const PROFILE_EVENTS: usize = 10;

pub struct ContextGfx {
    /// Variables such as "time".
    pub sync_vars: SyncVars,

    pub shader_sources: Vec<Vec<u8>>,
    // TODO rename to image_sources
    pub images: Vec<Image>,
    pub frame_buffers: Vec<FrameBuffer>,

    pub quad_scenes: Vec<QuadSceneGfx>,

    pub polygon_scenes: Vec<PolygonScene>,
    pub polygon_context: PolygonContext,

    pub camera: Camera,
    pub mouse: Mouse,

    /// Profile events for 60 frames, max 100 events per frame.
    pub profile_times: [[f32; PROFILE_EVENTS]; PROFILE_FRAMES],
    /// Current frame counter for profiling.
    pub profile_frame_idx: usize,
    /// Current profile event counter.
    pub profile_event_idx: usize,
    /// Count of actually profiled number of events.
    pub max_profile_event_idx: usize,

    pub t_frame_start: Instant,
    pub t_frame_end: Instant,

    pub is_running: bool,
}

impl ContextGfx {
    pub fn new_with_dimensions(
        window_width: f64,
        window_height: f64,
        screen_width: f64,
        screen_height: f64,
        camera: Option<Camera>,
    ) -> ContextGfx {
        ContextGfx::new(
            0.0, // time
            window_width,
            window_height,
            screen_width,
            screen_height,
            Vec::new(),                // shader sources
            Vec::new(),                // images
            Vec::new(),                // quad scenes
            Vec::new(),                // polygon scenes
            PolygonContext::default(), // polygon context
            Vec::new(),                // frame buffers
            camera,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: f64,
        window_width: f64,
        window_height: f64,
        screen_width: f64,
        screen_height: f64,
        shader_sources: Vec<Vec<u8>>,
        images: Vec<Image>,
        quad_scenes: Vec<QuadSceneGfx>,
        polygon_scenes: Vec<PolygonScene>,
        polygon_context: PolygonContext,
        frame_buffers: Vec<FrameBuffer>,
        camera: Option<Camera>,
    ) -> ContextGfx {
        let camera = if let Some(c) = camera {
            c
        } else {
            Camera::new_defaults(window_width as f32 / window_height as f32)
        };

        let mouse = Mouse::new(0.05);

        let mut sync_vars = SyncVars::default();

        sync_vars.set_builtin(Time, time);
        sync_vars.set_builtin(Window_Width, window_width);
        sync_vars.set_builtin(Window_Height, window_height);

        // FIXME The aspect of the quad sticks and doesn't cover the window when it is
        // resized. The effect could be a good thing for implementing a fixed ideal aspect ratio
        // which can be defined as a setting in the YAML description.

        sync_vars.set_builtin(Screen_Width, screen_width);
        sync_vars.set_builtin(Screen_Height, screen_height);

        let empty_profile: [[f32; PROFILE_EVENTS]; PROFILE_FRAMES] =
            [[0.0; PROFILE_EVENTS]; PROFILE_FRAMES];

        ContextGfx {
            sync_vars,

            shader_sources,
            images,
            quad_scenes,

            frame_buffers,

            polygon_scenes,
            polygon_context,

            camera,
            mouse,

            profile_times: empty_profile,
            profile_frame_idx: 0,
            profile_event_idx: 0,
            max_profile_event_idx: 0,

            t_frame_start: Instant::now(),
            t_frame_end: Instant::now(),

            is_running: true,
        }
    }

    pub fn gl_cleanup(&mut self) {
        for scene in self.quad_scenes.iter_mut() {
            scene.gl_cleanup();
        }
        for buffer in self.frame_buffers.iter_mut() {
            buffer.gl_cleanup();
        }
    }

    pub fn set_time(&mut self, time: f64) {
        self.sync_vars.set_builtin(Time, time);
    }

    pub fn get_time(&self) -> f64 {
        self.sync_vars.get_builtin(Time)
    }

    pub fn set_window_resolution(&mut self, width: f64, height: f64) {
        self.sync_vars.set_builtin(Window_Width, width);
        self.sync_vars.set_builtin(Window_Height, height);
    }

    pub fn get_window_resolution(&self) -> (f64, f64) {
        (
            self.sync_vars.get_builtin(Window_Width),
            self.sync_vars.get_builtin(Window_Height),
        )
    }

    pub fn get_window_aspect(&self) -> f64 {
        let (wx, wy) = self.get_window_resolution();
        wx / wy
    }

    pub fn set_screen_resolution(&mut self, width: f64, height: f64) {
        self.sync_vars.set_builtin(Screen_Width, width);
        self.sync_vars.set_builtin(Screen_Height, height);
    }

    pub fn get_screen_resolution(&self) -> (f64, f64) {
        (
            self.sync_vars.get_builtin(Screen_Width),
            self.sync_vars.get_builtin(Screen_Height),
        )
    }

    pub fn set_camera_sync(&mut self) {
        self.sync_vars
            .set_builtin(Camera_Pos_X, f64::from(self.camera.position.x));
        self.sync_vars
            .set_builtin(Camera_Pos_Y, f64::from(self.camera.position.y));
        self.sync_vars
            .set_builtin(Camera_Pos_Z, f64::from(self.camera.position.z));
        self.sync_vars
            .set_builtin(Camera_Front_X, f64::from(self.camera.front.x));
        self.sync_vars
            .set_builtin(Camera_Front_Y, f64::from(self.camera.front.y));
        self.sync_vars
            .set_builtin(Camera_Front_Z, f64::from(self.camera.front.z));
    }

    pub fn get_last_work_buffer(&self) -> &FrameBuffer {
        let n = self.frame_buffers.len();
        &self.frame_buffers[n - 1]
    }

    pub fn add_quad_scene(
        &mut self,
        vert_src_idx: usize,
        frag_src_idx: usize,
        layout_to_vars: Vec<UniformMapping>,
        binding_to_buffers: Vec<BufferMapping>,
    ) {
        let n = self.quad_scenes.len();
        let quad_scene_id = if n > 0 { n - 1 } else { 0 };

        let mut quad_scene = QuadSceneGfx::new(quad_scene_id as u8, vert_src_idx, frag_src_idx);

        quad_scene.layout_to_vars = layout_to_vars;
        quad_scene.binding_to_buffers = binding_to_buffers;

        self.quad_scenes.push(quad_scene);
    }

    pub fn impl_draw_quad_scene(&self, scene_idx: usize) {
        if let Some(ref scene) = self.quad_scenes.get(scene_idx) {
            scene.draw(&self).unwrap();
        } else {
            panic!("Quad scene index doesn't exist: {}", scene_idx);
        }
    }

    pub fn impl_target_buffer(&self, buffer_idx: usize) {
        if let Some(buffer) = self.frame_buffers.get(buffer_idx) {
            if let Some(fbo) = buffer.fbo {
                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
                }
            } else {
                panic!("This buffer hasn't been created: {}", buffer_idx);
            }
        } else {
            panic!("Buffer index doesn't exist: {}", buffer_idx);
        }
    }

    pub fn impl_clear(&self, red: u8, green: u8, blue: u8, alpha: u8) {
        let (f_red, f_green, f_blue, f_alpha) = (
            (f32::from(red) / 255.0),
            (f32::from(green) / 255.0),
            (f32::from(blue) / 255.0),
            (f32::from(alpha) / 255.0),
        );
        unsafe {
            gl::ClearColor(f_red, f_green, f_blue, f_alpha);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn impl_target_buffer_default(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn impl_profile_event(&mut self, _label_idx: usize) {
        if self.profile_frame_idx < PROFILE_FRAMES && self.profile_event_idx < PROFILE_EVENTS {
            let t_delta: Duration = self.t_frame_start.elapsed();
            // t_delta as nanosec
            let nanos: u64 =
                (t_delta.as_secs() * 1_000_000_000) + u64::from(t_delta.subsec_nanos());
            // as millisec
            let millis: f32 = (nanos as f32) / 1_000_000_f32;

            self.profile_times[self.profile_frame_idx][self.profile_event_idx] = millis;
            self.profile_event_idx += 1;

            if self.max_profile_event_idx < self.profile_event_idx {
                self.max_profile_event_idx = self.profile_event_idx;
            }
        }
    }

    pub fn impl_draw_polygon_scene(&self, scene_idx: usize) {
        if let Some(ref scene) = self.polygon_scenes.get(scene_idx) {
            scene.draw(&self).unwrap();
        } else {
            panic!("Polygon scene index doesn't exist: {}", scene_idx);
        }
    }

    /*
    pub fn impl_if_var_equal_draw_quad(&self, var_idx: usize, value: f64, scene_idx: usize) {
        if self.vars[var_idx] == value {
            self.impl_draw_quad_scene(scene_idx);
        }
    }

    pub fn impl_if_var_equal_draw_polygon(&self, var_idx: usize, value: f64, scene_idx: usize) {
        if self.vars[var_idx] == value {
            self.impl_draw_polygon_scene(scene_idx);
        }
    }
    */
}

impl Drop for ContextGfx {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}
