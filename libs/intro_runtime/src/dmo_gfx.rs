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

    pub fn create_quads(&mut self,
                        err_msg_buf: &mut [u8; ERR_MSG_LEN])
                        -> Result<(), RuntimeError>
    {
        for scene in self.context.quad_scenes.iter_mut() {

            let vert_src = match self.context.shader_sources.get(scene.vert_src_idx) {
                Some(a) => str::from_utf8(&a).unwrap(),
                None => return Err(FailedToCreateNoSuchVertSrcIdx),
            };

            let frag_src = match self.context.shader_sources.get(scene.frag_src_idx) {
                Some(a) => str::from_utf8(&a).unwrap(),
                None => return Err(FailedToCreateNoSuchFragSrcIdx),
            };

            scene.create_quad(vert_src, frag_src, err_msg_buf)?;
        }

        Ok(())
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

    pub fn create_frame_buffers(&mut self) -> Result<(), RuntimeError> {
        let (wx, wy) = self.context.get_window_resolution();

        for buffer in self.context.frame_buffers.iter_mut() {
            if let Some(idx) = buffer.image_data_idx {
                let image = match self.context.images.get(idx) {
                    Some(x) => x,
                    None => return Err(ImageIndexIsOutOfBounds),
                };
                buffer.create_buffer(wx as i32, wy as i32, Some(&image))?;
            } else {
                buffer.create_buffer(wx as i32, wy as i32, None)?;
            }
        }
        Ok(())
    }

    // FIXME there are two cases: re-creating framebuffers which are window-sized
    // (need new dimensions) and buffers which are fixed size (image texture).

    pub fn recreate_framebuffers(&mut self) -> Result<(), RuntimeError> {
        let (wx, wy) = self.context.get_window_resolution();
        for buffer in self.context.frame_buffers.iter_mut() {
            buffer.gl_cleanup();
            if let Some(idx) = buffer.image_data_idx {
                let image = match self.context.images.get(idx) {
                    Some(x) => x,
                    None => return Err(ImageIndexIsOutOfBounds),
                };
                buffer.create_buffer(wx as i32, wy as i32, Some(&image))?;
            } else {
                buffer.create_buffer(wx as i32, wy as i32, None)?;
            }
        }
        Ok(())
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

}
