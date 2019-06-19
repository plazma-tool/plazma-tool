#[cfg(any(target_os = "linux", target_os = "macos"))]
use libc::c_void;

#[cfg(target_os = "windows")]
use winapi::c_void;

use std::{mem, ptr, str};

use gl;
use gl::types::*;

use crate::context_gfx::ContextGfx;
use crate::error::RuntimeError;
use crate::error::RuntimeError::*;
use crate::shader::{compile_shader, link_program};
use crate::shapes::*;
use crate::types::*;
use crate::ERR_MSG_LEN;

pub struct QuadSceneGfx {
    /// an ID number to use with the `Draw_Quad_Scene(u8)` operator
    pub idx: u8,
    /// Load the vertex shader using this index from `Context.shader_sources[]`
    pub vert_src_idx: usize,
    /// Load the fragment shader using this index from `Context.shader_sources[]`
    pub frag_src_idx: usize,
    /// Maps uniform layout index to sync var index
    pub layout_to_vars: Vec<UniformMapping>,
    /// Maps uniform layout binding to frame buffer index
    pub binding_to_buffers: Vec<BufferMapping>,
    /// The OpenGL object.
    pub quad: Option<Quad>,
}

pub struct Quad {
    pub program: GLuint,
    pub vao: GLuint,
    pub vbo: GLuint,
}

impl QuadSceneGfx {
    pub fn new(idx: u8, vert_src_idx: usize, frag_src_idx: usize) -> QuadSceneGfx {
        QuadSceneGfx {
            idx,
            vert_src_idx,
            frag_src_idx,
            layout_to_vars: Vec::new(),
            binding_to_buffers: Vec::new(),
            quad: None,
        }
    }

    pub fn create_quad(
        &mut self,
        vert_src: &str,
        frag_src: &str,
        err_msg_buf: &mut [u8; ERR_MSG_LEN],
    ) -> Result<(), RuntimeError> {
        self.quad = Some(Quad::new(vert_src, frag_src, err_msg_buf)?);
        Ok(())
    }

    pub fn draw(&self, context: &ContextGfx) -> Result<(), RuntimeError> {
        match self.quad {
            Some(ref quad) => {
                unsafe {
                    // Use shader
                    gl::UseProgram(quad.program);

                    // Mapping sync var indexes to uniform layout indexes
                    for item in self.layout_to_vars.iter() {
                        use crate::types::UniformMapping::*;
                        match *item {
                            NOOP => {}

                            Float(layout_idx, var_idx) => {
                                gl::Uniform1f(
                                    i32::from(layout_idx),
                                    context.sync_vars.get_index(var_idx as usize)? as f32,
                                );
                            }

                            Vec2(layout_idx, var1, var2) => {
                                gl::Uniform2f(
                                    i32::from(layout_idx),
                                    context.sync_vars.get_index(var1 as usize)? as f32,
                                    context.sync_vars.get_index(var2 as usize)? as f32,
                                );
                            }

                            Vec3(layout_idx, var1, var2, var3) => {
                                gl::Uniform3f(
                                    i32::from(layout_idx),
                                    context.sync_vars.get_index(var1 as usize)? as f32,
                                    context.sync_vars.get_index(var2 as usize)? as f32,
                                    context.sync_vars.get_index(var3 as usize)? as f32,
                                );
                            }

                            Vec4(layout_idx, var1, var2, var3, var4) => {
                                gl::Uniform4f(
                                    i32::from(layout_idx),
                                    context.sync_vars.get_index(var1 as usize)? as f32,
                                    context.sync_vars.get_index(var2 as usize)? as f32,
                                    context.sync_vars.get_index(var3 as usize)? as f32,
                                    context.sync_vars.get_index(var4 as usize)? as f32,
                                );
                            }
                        }
                    }

                    // Bind a buffer as texture
                    for item in self.binding_to_buffers.iter() {
                        use crate::types::BufferMapping::*;
                        match *item {
                            NOOP => {}

                            Sampler2D(binding_idx, buffer_idx) => {
                                if (buffer_idx as usize) < context.frame_buffers.len() {
                                    if binding_idx <= gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS as u8 {
                                        if let Some(fbo) =
                                            context.frame_buffers[buffer_idx as usize].fbo
                                        {
                                            gl::ActiveTexture(
                                                gl::TEXTURE0 + u32::from(binding_idx),
                                            );
                                            gl::BindTexture(gl::TEXTURE_2D, fbo);
                                        } else {
                                            return Err(NoFbo);
                                        }
                                    } else {
                                        return Err(TextureBindingIdxIsOverTheHardwareLimit);
                                    }
                                } else {
                                    return Err(TextureBindingIdxDoesntExist);
                                }
                            }
                        }
                    }

                    gl::BindVertexArray(quad.vao);
                    gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                    gl::BindVertexArray(0);
                }
            }

            None => return Err(NoQuad),
        }

        Ok(())
    }

    pub fn gl_cleanup(&mut self) {
        if let Some(ref quad) = self.quad {
            quad.gl_cleanup();
        }
        self.quad = None;
    }
}

impl Quad {
    pub fn new(
        vert_src: &str,
        frag_src: &str,
        err_msg_buf: &mut [u8; ERR_MSG_LEN],
    ) -> Result<Quad, RuntimeError> {
        let mut quad = Quad {
            program: 0,
            vao: 0,
            vbo: 0,
        };
        quad.compile_program(vert_src, frag_src, err_msg_buf)?;

        unsafe {
            gl::GenVertexArrays(1, &mut quad.vao);
            gl::GenBuffers(1, &mut quad.vbo);

            // VAO
            gl::BindVertexArray(quad.vao);

            // VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, quad.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (QUAD_VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &QUAD_VERTICES[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );

            // Vertex data

            // layout (location = 0) in vec2 pos;
            let pos_attr: GLuint = 0;
            gl::EnableVertexAttribArray(pos_attr);
            gl::VertexAttribPointer(
                pos_attr,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * mem::size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );

            // layout (location = 1) in vec2 tex;
            let tex_attr: GLuint = 1;
            gl::EnableVertexAttribArray(tex_attr);
            gl::VertexAttribPointer(
                tex_attr,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * mem::size_of::<GLfloat>() as GLsizei,
                (2 * mem::size_of::<GLfloat>()) as *const c_void,
            );

            gl::BindVertexArray(0);
        }

        Ok(quad)
    }

    pub fn compile_program(
        &mut self,
        vert_src: &str,
        frag_src: &str,
        err_msg_buf: &mut [u8; ERR_MSG_LEN],
    ) -> Result<(), RuntimeError> {
        let vs = compile_shader(vert_src, gl::VERTEX_SHADER, err_msg_buf)?;
        let fs = compile_shader(frag_src, gl::FRAGMENT_SHADER, err_msg_buf)?;
        self.program = link_program(vs, fs, err_msg_buf)?;
        Ok(())
    }

    pub fn gl_cleanup(&self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl Drop for Quad {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}
