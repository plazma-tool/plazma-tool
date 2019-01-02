use std::error::Error;
use std::time::{Duration, Instant};

use smallvec::SmallVec;

use intro_runtime::ERR_MSG_LEN;
use intro_runtime::dmo_gfx::DmoGfx;
use intro_runtime::dmo_sync::SyncDevice;
use intro_runtime::sync_vars::builtin_to_idx;
use intro_runtime::sync_vars::BuiltIn::*;
use intro_runtime::frame_buffer::{FrameBuffer, BufferKind};
use intro_runtime::types::PixelFormat;
use intro_runtime::error::RuntimeError;
use intro_runtime::types::{BufferMapping, UniformMapping};

use crate::dmo_data::DmoData;
use crate::error::ToolError;

pub struct PreviewState {
    pub t_frame_start: Instant,
    pub t_delta: Duration,
    pub t_frame_target: Duration,

    pub draw_anyway: bool,
    pub should_recompile: bool,

    pub dmo_gfx: DmoGfx,
}

impl PreviewState {

    pub fn new(window_width: f64, window_height: f64,
               screen_width: f64, screen_height: f64)
               -> Result<PreviewState, Box<Error>>
    {
        let mut state = PreviewState {
            t_frame_start: Instant::now(),
            t_delta: Duration::new(0, 0),
            t_frame_target: Duration::from_millis(16),

            draw_anyway: false,
            should_recompile: false,

            dmo_gfx: DmoGfx::default(),
        };

        state.set_window_resolution(window_width, window_height);
        state.set_screen_resolution(screen_width, screen_height);

        Ok(state)
    }

    pub fn build(&mut self, dmo_data: &DmoData) -> Result<(), Box<Error>> {

        // Manually for now. First add objects, then create OpenGL objects.

        // Add frame buffers

        self.dmo_gfx.context.frame_buffers.push(
            FrameBuffer::new(BufferKind::Empty_Texture,
                             PixelFormat::RGBA_u8,
                             None)
        );

        // Add quad scenes

        for quad_scene_data in dmo_data.context.quad_scenes.iter() {
            if quad_scene_data.frag_src_path.ends_with("circle.frag") {
                let mut layout_to_vars: SmallVec<[UniformMapping; 64]> = SmallVec::new();

                layout_to_vars.push(UniformMapping::Float(0,
                                                          builtin_to_idx(Time) as u8));

                layout_to_vars.push(UniformMapping::Vec2(1,
                                                         builtin_to_idx(Window_Width) as u8,
                                                         builtin_to_idx(Window_Height) as u8));

                layout_to_vars.push(UniformMapping::Vec2(2,
                                                         builtin_to_idx(Screen_Width) as u8,
                                                         builtin_to_idx(Screen_Height) as u8));

                self.dmo_gfx
                    .context
                    .add_quad_scene(&quad_scene_data.vert_src,
                                    &quad_scene_data.frag_src,
                                    layout_to_vars,
                                    SmallVec::new());
            }
            else if quad_scene_data.frag_src_path.ends_with("cross.frag") {
                let mut layout_to_vars: SmallVec<[UniformMapping; 64]> = SmallVec::new();

                layout_to_vars.push(UniformMapping::Float(0,
                                                          builtin_to_idx(Time) as u8));

                layout_to_vars.push(UniformMapping::Vec2(1,
                                                         builtin_to_idx(Window_Width) as u8,
                                                         builtin_to_idx(Window_Height) as u8));

                layout_to_vars.push(UniformMapping::Vec2(2,
                                                         builtin_to_idx(Screen_Width) as u8,
                                                         builtin_to_idx(Screen_Height) as u8));

                let mut binding_to_buffers: SmallVec<[BufferMapping; 64]> = SmallVec::new();

                binding_to_buffers.push(BufferMapping::Sampler2D(0, 0));

                self.dmo_gfx
                    .context
                    .add_quad_scene(&quad_scene_data.vert_src,
                                    &quad_scene_data.frag_src,
                                    layout_to_vars,
                                    binding_to_buffers);
            }
        }

        // Create quads

        let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];

        match self.dmo_gfx.create_quads(&mut err_msg_buf) {
            Ok(_) => {},
            Err(e) => {
                let msg = String::from_utf8(err_msg_buf.to_vec())?;
                return Err(Box::new(ToolError::Runtime(e, msg)));
            }
        }

        // Create framebuffers

        match self.dmo_gfx.create_frame_buffers() {
            Ok(_) => {},
            Err(e) => return Err(Box::new(ToolError::Runtime(e, "".to_owned())))
        }

        Ok(())
    }

    pub fn recompile_dmo(&mut self) {

        // Compile a single QuadScene for now.

        if self.should_recompile {
            let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];
            match self.dmo_gfx.compile_quad_scene(0, &mut err_msg_buf) {
                Ok(_) => {
                    println!("Recompiled");
                },
                Err(e) => {
                    // FIXME return error
                    let msg = String::from_utf8(err_msg_buf.to_vec()).unwrap();
                    println!("Recompile error:\n{}", msg);
                    //return Err(Box::new(ToolError::Runtime(e, msg)));
                }
            }

            self.should_recompile = false;
        }

    }

    pub fn draw(&self) {
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
        self.dmo_gfx.context.set_window_resolution(wx, wy);
        match self.dmo_gfx.recreate_framebuffers() {
            Ok(_) => {},
            Err(e) => return Err(From::from(format!("{:?}", e))),
        };
        self.draw_anyway = true;

        let window_aspect = wx as f32 / wy as f32;
        // self.camera.aspect = window_aspect;
        // self.camera.update_projection();

        Ok(())
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

}
