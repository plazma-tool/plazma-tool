use core::str;

// TODO std::time is not very no_std
use std::time::{Instant, Duration};

use smallvec::SmallVec;

use gl;

use crate::ERR_MSG_LEN;
use crate::types::{UniformMapping, Image};
use crate::sync_vars::SyncVars;
use crate::quad_scene_gfx::QuadSceneGfx;
use crate::frame_buffer::FrameBuffer;
use crate::sync_vars::builtin_to_idx;
use crate::sync_vars::BuiltIn::*;

pub const PROFILE_FRAMES: usize = 60;
pub const PROFILE_EVENTS: usize = 10;

pub struct ContextGfx {
    /// Variables such as "time".
    pub sync_vars: SyncVars,

    /// 1kb x 64 shaders on the stack, larger shaders or more of them on the heap.
    pub shader_sources: SmallVec<[SmallVec<[u8; 1024]>; 64]>,
    pub images: SmallVec<[Image; 4]>,
    pub quad_scenes: SmallVec<[QuadSceneGfx; 64]>,
    pub frame_buffers: SmallVec<[FrameBuffer; 64]>,

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

impl Default for ContextGfx {
    fn default() -> ContextGfx {
        ContextGfx::new(0.0,// time
                        1024.0, 768.0,// window width and height
                        1024.0, 768.0,// screen width and height
                        SmallVec::new(),// shader sources
                        SmallVec::new(),// images
                        SmallVec::new(),// quad scenes
                        SmallVec::new()// frame buffers
        )
    }
}

impl ContextGfx {
    pub fn new(time: f64,
               window_width: f64,
               window_height: f64,
               screen_width: f64,
               screen_height: f64,
               shader_sources: SmallVec<[SmallVec<[u8; 1024]>; 64]>,
               images: SmallVec<[Image; 4]>,
               quad_scenes: SmallVec<[QuadSceneGfx; 64]>,
               frame_buffers: SmallVec<[FrameBuffer; 64]>)
               -> ContextGfx
    {
        let mut sync_vars = SyncVars::default();

        sync_vars.set_builtin(Time, time);
        sync_vars.set_builtin(Window_Width, window_width);
        sync_vars.set_builtin(Window_Height, window_height);
        sync_vars.set_builtin(Screen_Width, screen_width);
        sync_vars.set_builtin(Screen_Height, screen_height);

        let empty_profile: [[f32; PROFILE_EVENTS]; PROFILE_FRAMES] = [[0.0; PROFILE_EVENTS]; PROFILE_FRAMES];

        ContextGfx {
            sync_vars: sync_vars,

            shader_sources: shader_sources,
            images: images,
            quad_scenes: quad_scenes,

            frame_buffers: frame_buffers,

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
        for mut scene in self.quad_scenes.iter_mut() {
            scene.gl_cleanup();
        }
        for mut buffer in self.frame_buffers.iter_mut() {
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
        (self.sync_vars.get_builtin(Window_Width),
         self.sync_vars.get_builtin(Window_Height))
    }

    pub fn set_screen_resolution(&mut self, width: f64, height: f64) {
        self.sync_vars.set_builtin(Screen_Width, width);
        self.sync_vars.set_builtin(Screen_Height, height);
    }

    pub fn get_screen_resolution(&self) -> (f64, f64) {
        (self.sync_vars.get_builtin(Screen_Width),
         self.sync_vars.get_builtin(Screen_Height))
    }

    pub fn get_last_work_buffer(&self) -> &FrameBuffer {
        let n = self.frame_buffers.len();
        &self.frame_buffers[n - 1]
    }

    pub fn add_quad_scene(&mut self, vert_src: &str, frag_src: &str) {
        self.shader_sources.push(SmallVec::from_slice(vert_src.as_bytes()));
        let vert_src_idx = self.shader_sources.len() - 1;
        self.shader_sources.push(SmallVec::from_slice(frag_src.as_bytes()));
        let frag_src_idx = self.shader_sources.len() - 1;
        let n = self.quad_scenes.len();
        let quad_scene_id = if n > 0 { n - 1 } else { 0 };

        let mut quad_scene = QuadSceneGfx::new(quad_scene_id as u8, vert_src_idx, frag_src_idx);

        quad_scene.layout_to_vars.push(UniformMapping::Float(0,
                                                             builtin_to_idx(Time) as u8));

        quad_scene.layout_to_vars.push(UniformMapping::Vec2(1,
                                                            builtin_to_idx(Window_Width) as u8,
                                                            builtin_to_idx(Window_Height) as u8));

        quad_scene.layout_to_vars.push(UniformMapping::Vec2(3,
                                                            builtin_to_idx(Screen_Width) as u8,
                                                            builtin_to_idx(Screen_Height) as u8));

        let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];
        match quad_scene.create_quad(vert_src, frag_src, &mut err_msg_buf) {
            Ok(_) => {},
            Err(e) => println!("Error! Message:\n{}", str::from_utf8(&err_msg_buf).unwrap()),
        };
        self.quad_scenes.push(quad_scene);
    }

    pub fn impl_exit(&mut self, limit: f64) {
        if self.get_time() > limit {
            self.is_running = false;
        }
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
                unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, fbo); }
            } else {
                panic!("This buffer hasn't been created: {}", buffer_idx);
            }
        } else {
            panic!("Buffer index doesn't exist: {}", buffer_idx);
        }
    }

    pub fn impl_clear(&self, red: u8, green: u8, blue: u8, alpha: u8) {
        let (f_red, f_green, f_blue, f_alpha) = ((red   as f32 / 255.0),
                                                 (green as f32 / 255.0),
                                                 (blue  as f32 / 255.0),
                                                 (alpha as f32 / 255.0));
        unsafe {
            gl::ClearColor(f_red, f_green, f_blue, f_alpha);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn impl_target_buffer_default(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
    }

    pub fn impl_profile_event(&mut self, label_idx: usize) {
        if self.profile_frame_idx < PROFILE_FRAMES && self.profile_event_idx < PROFILE_EVENTS {
            let t_delta: Duration = self.t_frame_start.elapsed();
            // t_delta as nanosec
            let nanos: u64 = (t_delta.as_secs() * 1_000_000_000) + (t_delta.subsec_nanos() as u64);
            // as millisec
            let millis: f32 = (nanos as f32) / (1_000_000 as f32);

            self.profile_times[self.profile_frame_idx][self.profile_event_idx] = millis;
            self.profile_event_idx += 1;

            if self.max_profile_event_idx < self.profile_event_idx {
                self.max_profile_event_idx = self.profile_event_idx;
            }
        }
    }
}

impl Drop for ContextGfx {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}
