use std::error::Error;
use std::time::{Duration, Instant};
use std::path::PathBuf;

use plasma::utils::file_to_string;

pub struct PreviewState {
    pub time: f32,
    pub t_frame_start: Instant,
    pub t_delta: Duration,
    pub t_frame_target: Duration,

    pub is_running: bool,
    pub is_paused: bool,
    pub draw_anyway: bool,

    pub window_resolution: [f32; 2],

    pub vertex_shader_src: String,
    pub fragment_shader_src: String,
    pub should_recompile: bool,
}

impl PreviewState {

    pub fn new() -> Result<PreviewState, Box<Error>> {
        let state = PreviewState {
            time: 0.0,
            t_frame_start: Instant::now(),
            t_delta: Duration::new(0, 0),
            t_frame_target: Duration::from_millis(16),
            is_running: true,
            is_paused: true,
            draw_anyway: false,
            window_resolution: [1024.0_f32, 768.0_f32],
            vertex_shader_src: file_to_string(&PathBuf::from("./data/screen_quad.vert")).unwrap(),
            fragment_shader_src: file_to_string(&PathBuf::from("./data/shader.frag")).unwrap(),
            should_recompile: false,
        };

        Ok(state)
    }

    pub fn update_time(&mut self) {
        self.t_frame_start = Instant::now();
        self.time += 16.0;
    }

    pub fn get_is_running(&self) -> bool {
        self.is_running
    }

    pub fn set_is_running(&mut self, value: bool) {
        self.is_running = value
    }

    pub fn get_is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn set_is_paused(&mut self, value: bool) {
        self.is_paused = value;
    }

    pub fn set_vertex_shader_src(&mut self, src: String) {
        self.vertex_shader_src = src;
    }

    pub fn set_fragment_shader_src(&mut self, src: String) {
        self.fragment_shader_src = src;
        self.should_recompile = true;
    }

}
