use core::str;

use std::time::Instant;

use smallvec::SmallVec;

use intro_3d::{Vector3, to_radians};

use crate::context_gfx::ContextGfx;
use crate::camera::Camera;
use crate::mesh::Mesh;
use crate::model::ModelType;
use crate::types::{ValueFloat, ValueVec3};
use crate::dmo_sync::DmoSync;
use crate::error::RuntimeError;
use crate::ERR_MSG_LEN;
use crate::context_gfx::PROFILE_FRAMES;
//use crate::error::RuntimeError;
use crate::error::RuntimeError::*;
use crate::timeline::Timeline;

pub struct DmoGfx {
    pub settings: Settings,
    pub context: ContextGfx,
    pub sync: DmoSync,
    pub timeline: Timeline,
}

pub struct Settings {
    pub start_full_screen: bool,
    pub audio_play_on_start: bool,
    pub mouse_sensitivity: f32,
    pub movement_sensitivity: f32,
    pub total_length: f64,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            start_full_screen: false,
            audio_play_on_start: true,
            mouse_sensitivity: 0.5,
            movement_sensitivity: 0.5,
            total_length: 10.0,
        }
    }
}

// impl Default for DmoGfx {
//     fn default() -> DmoGfx {
//         DmoGfx {
//             settings: Settings::default(),
//             context: ContextGfx::default(),
//             sync: DmoSync::default(),
//             timeline: Timeline::default(),
//         }
//     }
// }

impl DmoGfx {
    pub fn new_with_dimensions(window_width: f64,
                               window_height: f64,
                               screen_width: f64,
                               screen_height: f64,
                               camera: Option<Camera>)
        -> DmoGfx
    {
        DmoGfx {
            settings: Settings::default(),
            context: ContextGfx::new_with_dimensions(window_width,
                                                     window_height,
                                                     screen_width,
                                                     screen_height,
                                                     camera),
            sync: DmoSync::default(),
            timeline: Timeline::default(),
        }
    }

    pub fn draw(&mut self)
    {
        use crate::timeline::DrawOp::*;

        for op in self.timeline.draw_ops_at_time(self.context.get_time()) {
            match op {
                NOOP => {},
                Draw_Quad_Scene(x) => self.context.impl_draw_quad_scene(x),
                Draw_Poly_Scene(x) => self.context.impl_draw_polygon_scene(x),
                Clear(r, g, b, a) => self.context.impl_clear(r, g, b, a),
                Target_Buffer(x) => self.context.impl_target_buffer(x),
                Target_Buffer_Default => self.context.impl_target_buffer_default(),
                Profile(x) => self.context.impl_profile_event(x),
            }
        }

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

    pub fn create_frame_buffers(&mut self) -> Result<(), RuntimeError>
    {
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

    pub fn recreate_framebuffers(&mut self) -> Result<(), RuntimeError>
    {
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

    pub fn update_vars(&mut self) -> Result<(), RuntimeError>
    {
        self.sync.update_vars(&mut self.context)
    }

    pub fn get_shader_src(&mut self,
                          idx: usize)
        -> Result<SmallVec<[u8; 1024]>, RuntimeError>
    {
        if idx < self.context.shader_sources.len() {
            let mut s = SmallVec::new();
            for i in self.context.shader_sources[idx].iter() {
                s.push(*i);
            }
            return Ok(s);
        } else {
            return Err(ShaderSourceIdxIsOutOfBounds);
        }
    }

    pub fn update_shader_src(&mut self,
                             idx: usize,
                             frag_src: &str)
                             -> Result<(), RuntimeError>
    {
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

    pub fn update_shader_src_from_vec(&mut self,
                                      idx: usize,
                                      frag_src_vec: &SmallVec<[u8; 1024]>)
        -> Result<(), RuntimeError>
    {
        if idx < self.context.shader_sources.len() {
            let mut s = SmallVec::new();
            for i in frag_src_vec.iter() {
                s.push(*i);
            }
            self.context.shader_sources[idx] = s;
        } else {
            return Err(ShaderSourceIdxIsOutOfBounds);
        }
        Ok(())
    }

    pub fn update_time_frame_start(&mut self, t: Instant)
    {
        self.context.t_frame_start = t;
        self.context.profile_event_idx = 0;
    }

    pub fn update_time_frame_end(&mut self, t: Instant)
    {
        self.context.t_frame_end = t;
        if self.context.profile_frame_idx < PROFILE_FRAMES - 1 {
            self.context.profile_frame_idx += 1;
        } else {
            self.context.profile_frame_idx = 0;
        }
    }

    pub fn create_models(&mut self,
                         err_msg_buf: &mut [u8; ERR_MSG_LEN])
                         -> Result<(), RuntimeError>
    {
        for model in self.context.polygon_context.models.iter_mut() {

            let mut new_meshes: SmallVec<[Mesh; 2]> = SmallVec::new();

            for mesh in model.meshes.iter_mut() {
                let vert_src = match self.context.shader_sources.get(mesh.vert_src_idx) {
                    Some(a) => str::from_utf8(&a).unwrap(),
                    None => return Err(FailedToCreateNoSuchVertSrcIdx),
                };

                let frag_src = match self.context.shader_sources.get(mesh.frag_src_idx) {
                    Some(a) => str::from_utf8(&a).unwrap(),
                    None => return Err(FailedToCreateNoSuchFragSrcIdx),
                };

                let model_type = &model.model_type;
                match model_type {
                    &ModelType::NOOP => {},

                    &ModelType::Cube => {
                        let mut m = Mesh::new_cube(&vert_src, &frag_src, err_msg_buf)?;
                        // Keep the idx values so that the asset manager can
                        // find the mesh that belongs to a path when the shader
                        // file changes.
                        m.vert_src_idx = mesh.vert_src_idx;
                        m.frag_src_idx = mesh.frag_src_idx;
                        new_meshes.push(m);
                    },

                    &ModelType::Obj => {
                        let mut m = Mesh::new(&mesh.vertices,
                                              &mesh.indices,
                                              &mesh.textures,
                                              &vert_src,
                                              &frag_src,
                                              err_msg_buf)?;
                        // Keep the idx
                        m.vert_src_idx = mesh.vert_src_idx;
                        m.frag_src_idx = mesh.frag_src_idx;
                        new_meshes.push(m);
                    },
                }
            }

            model.meshes = new_meshes;
        }
        Ok(())
    }

    pub fn compile_model_shaders(&mut self, model_idx: usize, err_msg_buf: &mut [u8; ERR_MSG_LEN])
        -> Result<(), RuntimeError>
    {
        for mesh in self.context.polygon_context.models[model_idx].meshes.iter_mut() {
            let ref s = self.context.shader_sources[mesh.vert_src_idx];
            let vert_src = str::from_utf8(s).unwrap();
            let ref s = self.context.shader_sources[mesh.frag_src_idx];
            let frag_src = str::from_utf8(s).unwrap();
            mesh.compile_program(vert_src, frag_src, err_msg_buf)?;
        }
        Ok(())
    }

    pub fn update_polygon_context_projection(&mut self, aspect: f32) {
        self.context.polygon_context.update_projection_matrix(aspect);
    }

    /// Update global view and projection matrix of the PolygonContext, and
    /// update individual model matrices.
    pub fn update_polygon_context(&mut self) -> Result<(), RuntimeError>
    {
        use crate::sync_vars::BuiltIn::*;

        self.context.polygon_context.view_position =
            Vector3::new(self.context.sync_vars.get_builtin(Camera_Pos_X) as f32,
                         self.context.sync_vars.get_builtin(Camera_Pos_Y) as f32,
                         self.context.sync_vars.get_builtin(Camera_Pos_Z) as f32);

        self.context.polygon_context.view_front =
            Vector3::new(self.context.sync_vars.get_builtin(Camera_Front_X) as f32,
                         self.context.sync_vars.get_builtin(Camera_Front_Y) as f32,
                         self.context.sync_vars.get_builtin(Camera_Front_Z) as f32);

        self.context.polygon_context.fovy = self.context.sync_vars.get_builtin(Fovy) as f32;
        self.context.polygon_context.znear = self.context.sync_vars.get_builtin(Znear) as f32;
        self.context.polygon_context.zfar = self.context.sync_vars.get_builtin(Zfar) as f32;

        self.context.polygon_context.update_view_matrix();

        for scene in self.context.polygon_scenes.iter_mut() {
            for mut scene_object in scene.scene_objects.iter_mut() {
                match scene_object.position_var {
                    ValueVec3::NOOP => {},
                    ValueVec3::Sync(x, y, z) => {
                        scene_object.position =
                            Vector3::new(self.context.sync_vars.get_index(x as usize)? as f32,
                                         self.context.sync_vars.get_index(y as usize)? as f32,
                                         self.context.sync_vars.get_index(z as usize)? as f32);
                    },
                    ValueVec3::Fixed(x, y, z) => {
                        scene_object.position = Vector3::new(x, y, z);
                    },
                }

                match scene_object.euler_rotation_var {
                    ValueVec3::NOOP => {},
                    ValueVec3::Sync(x, y, z) => {
                        scene_object.euler_rotation =
                            Vector3::new(to_radians(self.context.sync_vars.get_index(x as usize)? as f32),
                                         to_radians(self.context.sync_vars.get_index(y as usize)? as f32),
                                         to_radians(self.context.sync_vars.get_index(z as usize)? as f32));
                    },
                    ValueVec3::Fixed(x, y, z) => {
                        scene_object.euler_rotation = Vector3::new(to_radians(x),
                                                                   to_radians(y),
                                                                   to_radians(z));
                    },
                }

                match scene_object.scale_var {
                    ValueFloat::NOOP => {},
                    ValueFloat::Sync(x) => {
                        scene_object.scale = self.context.sync_vars.get_index(x as usize)? as f32;
                    },
                    ValueFloat::Fixed(x) => {
                        scene_object.scale = x;
                    },
                }

                scene_object.update_model_matrix();
            }
        }

        Ok(())
    }
}
