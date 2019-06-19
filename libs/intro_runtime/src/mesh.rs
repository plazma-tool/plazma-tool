#[cfg(any(target_os = "linux", target_os = "macos"))]
use libc::c_void;

#[cfg(target_os = "windows")]
use winapi::c_void;

use std::{mem, ptr};

use gl;
use gl::types::*;

use crate::context_gfx::ContextGfx;
use crate::error::RuntimeError;
use crate::error::RuntimeError::*;
use crate::model::ModelViewProjection;
use crate::shader::{compile_shader, link_program};
use crate::shapes::{CUBE_ELEMENTS, CUBE_VERTICES};
use crate::texture::Texture;
use crate::types::{BufferMapping, UniformMapping, Vertex};
use crate::ERR_MSG_LEN;

pub struct Mesh {
    pub vert_src_idx: usize,
    pub frag_src_idx: usize,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,

    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
}

impl Default for Mesh {
    fn default() -> Mesh {
        Mesh {
            vert_src_idx: 0,
            frag_src_idx: 0,

            vertices: Vec::new(),
            indices: Vec::new(),
            textures: Vec::new(),

            program: 0,

            vao: 0,
            vbo: 0,
            ebo: 0,
        }
    }
}

impl Mesh {
    pub fn new(
        vertices: &[Vertex],
        indices: &[u32],
        textures: &[Texture],
        vert_src: &str,
        frag_src: &str,
        err_msg_buf: &mut [u8; ERR_MSG_LEN],
    ) -> Result<Mesh, RuntimeError> {
        // TODO Possibly avoid this copy by organizing a two-step creation
        // process, first set the data and then compile the shaders and so on.

        // Have to make new ones with from_slice() because of the borrow.

        let mut mesh = Mesh {
            vert_src_idx: 0,
            frag_src_idx: 0,
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            textures: textures.to_vec(),
            program: 0,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        mesh.compile_program(vert_src, frag_src, err_msg_buf)?;

        // vao, vbo, ebo
        unsafe {
            gl::GenVertexArrays(1, &mut mesh.vao);
            gl::GenBuffers(1, &mut mesh.vbo);
            gl::GenBuffers(1, &mut mesh.ebo);

            // upload the vertex data
            gl::BindBuffer(gl::ARRAY_BUFFER, mesh.vbo);
            // Vertex struct's attribute order is arranged so that its memory
            // layout can be used as a byte array and we can pass a pointer to it here.
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.vertices.len() * mem::size_of::<Vertex>()) as isize,
                mesh.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Set vertex attribute pointers. Last parameter is byte offset.
            // Offset calculation hard-coded here, as a reliable offsetof() seems to be an open issue.
            // https://github.com/rust-lang/rfcs/issues/710

            gl::BindVertexArray(mesh.vao);

            // position
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                ptr::null(),
            );
            // normal
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                (3 * mem::size_of::<GLfloat>()) as *const c_void,
            );
            // texture coord
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                ((3 + 3) * mem::size_of::<GLfloat>()) as *const c_void,
            );
            // // tangent
            // gl::EnableVertexAttribArray(3);
            // gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE,
            //                         mem::size_of::<Vertex>() as GLsizei,
            //                         mem::transmute((3+3+2) * mem::size_of::<GLfloat>()));
            // // bitangent
            // gl::EnableVertexAttribArray(4);
            // gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE,
            //                         mem::size_of::<Vertex>() as GLsizei,
            //                         mem::transmute((3+3+2+3) * mem::size_of::<GLfloat>()));

            // unbind
            gl::BindVertexArray(0);

            // ebo indices
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.indices.len() * mem::size_of::<u32>()) as isize,
                mesh.indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
        }

        Ok(mesh)
    }

    pub fn new_cube(
        vert_src: &str,
        frag_src: &str,
        err_msg_buf: &mut [u8; ERR_MSG_LEN],
    ) -> Result<Mesh, RuntimeError> {
        let mut vertices: Vec<Vertex> = Vec::new();

        for v in CUBE_VERTICES.iter() {
            vertices.push(Vertex {
                position: [v[0], v[1], v[2]],
                normal: [v[3], v[4], v[5]],
                texcoords: [v[6], v[7]],
                //tangent:   [0.0, 0.0, 0.0],
                //bitangent: [0.0, 0.0, 0.0],
            });
        }

        let mut indices: Vec<u32> = Vec::new();
        indices.extend_from_slice(&CUBE_ELEMENTS);

        let textures: Vec<Texture> = Vec::new();

        let cube = Mesh::new(
            &vertices,
            &indices,
            &textures,
            vert_src,
            frag_src,
            err_msg_buf,
        )?;
        Ok(cube)
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

    pub fn draw(
        &self,
        context: &ContextGfx,
        layout_to_vars: &[UniformMapping],
        binding_to_buffers: &[BufferMapping],
        mvp: &ModelViewProjection,
        camera_pos: &[f32; 3],
    ) -> Result<(), RuntimeError> {
        // bind vao, use shader
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::UseProgram(self.program);
        }

        // send in uniforms
        // 0 = mat4 model
        // 1 = mat4 view
        // 2 = mat4 projection
        // 3 = vec3 camera pos

        unsafe {
            gl::UniformMatrix4fv(0, 1, gl::FALSE, mvp.model[0].as_ptr());
            gl::UniformMatrix4fv(1, 1, gl::FALSE, mvp.view[0].as_ptr());
            gl::UniformMatrix4fv(2, 1, gl::FALSE, mvp.projection[0].as_ptr());
            gl::Uniform3f(3, camera_pos[0], camera_pos[1], camera_pos[2]);
        }

        // TODO this is the same as in QuadScene::draw()

        unsafe {
            // Mapping sync var indexes to uniform layout indexes
            for item in layout_to_vars.iter() {
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
        }

        // TODO this is the same as in QuadScene::draw()

        unsafe {
            // Bind a buffer as texture
            for item in binding_to_buffers.iter() {
                use crate::types::BufferMapping::*;
                match *item {
                    NOOP => {}

                    Sampler2D(binding_idx, buffer_idx) => {
                        if (buffer_idx as usize) < context.frame_buffers.len() {
                            if binding_idx <= gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS as u8 {
                                if let Some(fbo) = context.frame_buffers[buffer_idx as usize].fbo {
                                    gl::ActiveTexture(gl::TEXTURE0 + u32::from(binding_idx));
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
        }

        // FIXME remove textures?
        // // bind the textures
        // for (binding_idx, t) in self.textures.iter().enumerate() {
        //     t.bind(binding_idx as u8)?;
        // }

        unsafe {
            // draw the mesh

            if !self.indices.is_empty() {
                // If the indices list is not empty, assume the vertices are EBO
                // indexed array.
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.indices.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            } else {
                // Otherwise, draw the vertices as a triangle array.
                gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32);
            }

            // unbind VAO
            gl::BindVertexArray(0);
            // unbind any bound texture
            gl::ActiveTexture(gl::TEXTURE0);
        }
        Ok(())
    }

    pub fn gl_cleanup(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
        for tex in self.textures.iter_mut() {
            tex.gl_cleanup();
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}
