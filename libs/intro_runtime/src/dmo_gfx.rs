use core::str;

use std::time::Instant;

use smallvec::SmallVec;

use crate::context_gfx::ContextGfx;
use crate::dmo_sync::DmoSync;
use crate::error::RuntimeError;
use crate::ERR_MSG_LEN;
use crate::context_gfx::PROFILE_FRAMES;
//use crate::error::RuntimeError;
use crate::error::RuntimeError::*;

pub struct DmoGfx {
    pub context: ContextGfx,
    pub sync: DmoSync,
    // TODO timeline, which will be used to build the draw_ops
    //pub timeline: Timeline,
}

impl Default for DmoGfx {
    fn default() -> DmoGfx {
        DmoGfx {
            context: ContextGfx::default(),
            sync: DmoSync::default(),
        }
    }
}

impl DmoGfx {
    pub fn draw(&self) {
        self.context.impl_target_buffer_default();
        self.context.impl_clear(0, 255, 0, 0);
        self.context.impl_draw_quad_scene(0);
    }

    pub fn update_vars(&mut self) -> Result<(), RuntimeError> {
        self.sync.update_vars(&mut self.context)
    }

    pub fn update_shader_src(&mut self, idx: usize, frag_src: &str) -> Result<(), RuntimeError> {
        if idx < self.context.shader_sources.len() {
            let mut s = SmallVec::new();
            for i in frag_src.as_bytes().iter() {
                s.push(*i);
            }
            self.context.shader_sources[idx] = s;
        } else {
            return Err(ShaderSourceIdxIsOutOfBounds);
        }
        Ok(())
    }

    pub fn update_time_frame_start(&mut self, t: Instant) {
        self.context.t_frame_start = t;
        self.context.profile_event_idx = 0;
    }

    pub fn update_time_frame_end(&mut self, t: Instant) {
        self.context.t_frame_end = t;
        if self.context.profile_frame_idx < PROFILE_FRAMES - 1 {
            self.context.profile_frame_idx += 1;
        } else {
            self.context.profile_frame_idx = 0;
        }
    }

    pub fn compile_quad_scene(&mut self,
                              scene_idx: usize,
                              err_msg_buf: &mut [u8; ERR_MSG_LEN])
                              -> Result<(), RuntimeError>
    {
        if scene_idx >= self.context.quad_scenes.len() {
            return Err(SceneIdxIsOutOfBounds);
        }

        let vert_src_idx = self.context.quad_scenes[scene_idx].vert_src_idx;
        let frag_src_idx = self.context.quad_scenes[scene_idx].frag_src_idx;

        if let Some(ref mut quad) = self.context.quad_scenes[scene_idx].quad {
            let ref s = self.context.shader_sources[vert_src_idx];
            let vert_src = str::from_utf8(s).unwrap();
            let ref s = self.context.shader_sources[frag_src_idx];
            let frag_src = str::from_utf8(s).unwrap();

            quad.compile_program(vert_src, frag_src, err_msg_buf)?;
        }

        Ok(())
    }

}
