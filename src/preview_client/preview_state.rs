use std::error::Error;
use std::time::{Duration, Instant};
use std::path::PathBuf;

use crate::utils::file_to_string;

pub struct PreviewState {
    pub time: f32,
    pub t_frame_start: Instant,
    pub t_delta: Duration,
    pub t_frame_target: Duration,

    pub is_running: bool,
    pub is_paused: bool,
    pub draw_anyway: bool,

    pub window_resolution: [f32; 2],

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

}
