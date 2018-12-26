use core::{mem, ptr, str};
use smallvec::SmallVec;

// TODO std::time is not very no_std
use std::time::{Duration, Instant};

use gl;
use gl::types::*;

use jit::JitFn;

use sync::DmoSync;
use shader::{compile_shader, link_program};
use bytecode::push_f32;
use intro_3d::{Vector3, Matrix4, to_radians};
use shapes::*;

use error::{RuntimeError, ERR_MSG_LEN};
use error::RuntimeError::*;

pub const VAR_NUM: usize = 2048;
pub const PROFILE_FRAMES: usize = 60;
pub const PROFILE_EVENTS: usize = 10;

/// Holds the data we need to access when running the code.
/// `.context` and `.operators` are private so that we can rebuild the JIT fn when they change.
pub struct Dmo {
    context: Context,
    operators: SmallVec<[Operator; 64]>,
    pub sync: DmoSync,
    jit_fn: JitFn,
}

pub struct Context {
    pub vars: [f64; VAR_NUM],
    pub shader_sources: SmallVec<[SmallVec<[u8; 1024]>; 64]>,
    pub images: SmallVec<[Image; 4]>,
    pub quad_scenes: SmallVec<[QuadScene; 64]>,
    pub polygon_scenes: SmallVec<[PolygonScene; 64]>,
    pub polygon_context: PolygonContext,
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

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub raw_pixels: SmallVec<[u8; 1024]>,
}

#[derive(Copy, Clone)]
pub enum PixelFormat {
    NOOP,
    RED_u8,
    RGB_u8,
    RGBA_u8,
}

pub struct QuadScene {
    /// an ID number to use with the `Draw_Quad_Scene(u8)` operator
    pub id: u8,
    /// Load the vertex shader using this index from `Context.shader_sources[]`
    pub vert_src_idx: usize,
    /// Load the fragment shader using this index from `Context.shader_sources[]`
    pub frag_src_idx: usize,
    /// Maps uniform layout index to sync var index
    pub layout_to_vars: SmallVec<[UniformMapping; 64]>,
    /// Maps uniform layout binding to frame buffer index
    pub binding_to_buffers: SmallVec<[BufferMapping; 64]>,

    pub quad: Option<Quad>,
}

pub struct PolygonScene {
    pub scene_objects: SmallVec<[SceneObject; 4]>,
}

pub struct SceneObject {
    pub model_idx: usize,

    pub position: Vector3,
    pub euler_rotation: Vector3,
    pub scale: f32,

    pub position_var: ValueVec3,
    pub euler_rotation_var: ValueVec3,
    pub scale_var: ValueFloat,

    pub layout_to_vars: SmallVec<[UniformMapping; 64]>,
    pub binding_to_buffers: SmallVec<[BufferMapping; 64]>,

    /// Model matrix to use when drawing the model retreived with `model_idx`
    /// from `PolygonContext.models`.
    pub model_matrix: [[f32; 4]; 4],
}

pub enum ValueVec3 {
    NOOP,
    Sync(u8, u8, u8),
    Fixed(f32, f32, f32),
}

pub enum ValueFloat {
    NOOP,
    Sync(u8),
    Fixed(f32),
}

pub struct PolygonContext {
    pub view_position: Vector3,
    pub view_front: Vector3,
    pub view_up: Vector3,

    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],

    pub view_position_var_idx: [usize; 3],
    pub view_front_var_idx: [usize; 3],

    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    pub models: SmallVec<[Model; 4]>,
}

pub struct Quad {
    pub program: GLuint,
    pub vao: GLuint,
    pub vbo: GLuint,
}

/// Order of attributes is significant, we want this to translate to a specific
/// data layout in memory and use byte offsets when setting the vertex
/// attributes.
#[derive(Clone)]
pub struct Vertex {
    pub position:  [GLfloat; 3],
    pub normal:    [GLfloat; 3],
    pub texcoords: [GLfloat; 2],
    //pub tangent:   [GLfloat; 3],
    //pub bitangent: [GLfloat; 3],
}

#[derive(Clone)]
pub struct Texture {
    width: i32,
    height: i32,
    format: PixelFormat,
    image_data_idx: Option<usize>,
    id: Option<GLuint>,
}

pub struct Mesh {
    pub vert_src_idx: usize,
    pub frag_src_idx: usize,

    pub vertices: SmallVec<[Vertex; 8]>,
    pub indices: SmallVec<[u32; 8]>,
    pub textures: SmallVec<[Texture; 2]>,

    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
}

#[derive(Copy, Clone)]
pub enum ModelType {
    NOOP,
    Cube,
    Obj,
}

pub struct Model {
    pub model_type: ModelType,
    // pub textures_loaded: SmallVec<[Texture; 2]>, // is this needed?
    pub meshes: SmallVec<[Mesh; 2]>,
}

impl Model {
    pub fn new(model_type: ModelType) -> Model {
        let mut m = Model::default();
        m.model_type = model_type;
        m
    }

    pub fn draw(&self,
                context: &Context,
                layout_to_vars: &SmallVec<[UniformMapping; 64]>,
                binding_to_buffers: &SmallVec<[BufferMapping; 64]>,
                model: &[[f32; 4]; 4],
                view: &[[f32; 4]; 4],
                projection: &[[f32; 4]; 4],
                camera_pos: &[f32; 3])
                -> Result<(), RuntimeError>
    {
        for m in self.meshes.iter() {
            m.draw(context, layout_to_vars, binding_to_buffers, model, view, projection, camera_pos)?;
        }
        Ok(())
    }

    pub fn gl_cleanup(&mut self) {
        for mut mesh in self.meshes.iter_mut() {
            mesh.gl_cleanup();
        }
    }
}

impl Mesh {
    pub fn new(vertices: &SmallVec<[Vertex; 8]>,
               indices: &SmallVec<[u32; 8]>,
               textures: &SmallVec<[Texture; 2]>,
               vert_src: &str,
               frag_src: &str,
               err_msg_buf: &mut [u8; ERR_MSG_LEN])
               -> Result<Mesh, RuntimeError>
    {
        // TODO Possibly avoid this copy by organizing a two-step creation
        // process, first set the data and then compile the shaders and so on.

        // Have to make new ones with from_slice() because of the borrow.

        let mut mesh = Mesh {
            vert_src_idx: 0,
            frag_src_idx: 0,
            vertices: vertices.clone(),
            indices: SmallVec::from_slice(indices),
            textures: textures.clone(),
            program: 0,
            vao: 0, vbo: 0, ebo: 0,
        };

        mesh.compile_shaders(vert_src, frag_src, err_msg_buf)?;

        // vao, vbo, ebo
        unsafe {
            gl::GenVertexArrays(1, &mut mesh.vao);
            gl::GenBuffers(1, &mut mesh.vbo);
            gl::GenBuffers(1, &mut mesh.ebo);

            // upload the vertex data
            gl::BindBuffer(gl::ARRAY_BUFFER, mesh.vbo);
            // Vertex struct's attribute order is arranged so that its memory
            // layout can be used as a byte array and we can pass a pointer to it here.
            gl::BufferData(gl::ARRAY_BUFFER,
                           (mesh.vertices.len() * mem::size_of::<Vertex>()) as isize,
                           mem::transmute( mesh.vertices.as_ptr() ),
                           gl::STATIC_DRAW);

            // Set vertex attribute pointers. Last parameter is byte offset.
            // Offset calculation hard-coded here, as a reliable offsetof() seems to be an open issue.
            // https://github.com/rust-lang/rfcs/issues/710

            gl::BindVertexArray(mesh.vao);

            // position
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE,
                                    mem::size_of::<Vertex>() as GLsizei,
                                    ptr::null());
            // normal
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE,
                                    mem::size_of::<Vertex>() as GLsizei,
                                    mem::transmute(3 * mem::size_of::<GLfloat>()));
            // texture coord
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE,
                                    mem::size_of::<Vertex>() as GLsizei,
                                    mem::transmute((3+3) * mem::size_of::<GLfloat>()));
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
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (mesh.indices.len() * mem::size_of::<u32>()) as isize,
                           mem::transmute( mesh.indices.as_ptr() ),
                           gl::STATIC_DRAW);
        }

        Ok(mesh)
    }

    pub fn new_cube(vert_src: &str,
                    frag_src: &str,
                    err_msg_buf: &mut [u8; ERR_MSG_LEN])
                    -> Result<Mesh, RuntimeError>
    {
        let mut vertices: SmallVec<[Vertex; 8]> = SmallVec::new();

        for v in CUBE_VERTICES.iter() {
            vertices.push(Vertex {
                position:  [v[0], v[1], v[2]],
                normal:    [v[3], v[4], v[5]],
                texcoords: [v[6], v[7]],
                //tangent:   [0.0, 0.0, 0.0],
                //bitangent: [0.0, 0.0, 0.0],
            });
        }

        let mut indices: SmallVec<[u32; 8]> = SmallVec::new();
        indices.extend_from_slice(&CUBE_ELEMENTS);

        let textures: SmallVec<[Texture; 2]> = SmallVec::new();

        let cube = Mesh::new(
            &vertices, &indices, &textures,
            vert_src, frag_src,
            err_msg_buf
        )?;
        Ok(cube)
    }

    pub fn compile_shaders(&mut self,
                           vert_src: &str,
                           frag_src: &str,
                           err_msg_buf: &mut [u8; ERR_MSG_LEN])
                           -> Result<(), RuntimeError>
    {
        let vs = compile_shader(vert_src, gl::VERTEX_SHADER, err_msg_buf)?;
        let fs = compile_shader(frag_src, gl::FRAGMENT_SHADER, err_msg_buf)?;
        self.program = link_program(vs, fs, err_msg_buf)?;
        Ok(())
    }

    pub fn draw(&self,
                context: &Context,
                layout_to_vars: &SmallVec<[UniformMapping; 64]>,
                binding_to_buffers: &SmallVec<[BufferMapping; 64]>,
                model: &[[f32; 4]; 4],
                view: &[[f32; 4]; 4],
                projection: &[[f32; 4]; 4],
                camera_pos: &[f32; 3])
                -> Result<(), RuntimeError>
    {
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
            gl::UniformMatrix4fv(0, 1, gl::FALSE, model[0].as_ptr());
            gl::UniformMatrix4fv(1, 1, gl::FALSE, view[0].as_ptr());
            gl::UniformMatrix4fv(2, 1, gl::FALSE, projection[0].as_ptr());
            gl::Uniform3f(3, camera_pos[0], camera_pos[1], camera_pos[2]);
        }

        // TODO this is the same as in QuadScene::draw()

        unsafe {
            // Mapping sync var indexes to uniform layout indexes
            for item in layout_to_vars.iter() {
                use self::UniformMapping::*;
                match *item {
                    NOOP => {},

                    Float(layout_idx, var_idx) => {
                        gl::Uniform1f(layout_idx as i32,
                                      context.vars[var_idx as usize] as f32);
                    },

                    Vec2(layout_idx, var1, var2) => {
                        gl::Uniform2f(layout_idx as i32,
                                      context.vars[var1 as usize] as f32,
                                      context.vars[var2 as usize] as f32);
                    },

                    Vec3(layout_idx, var1, var2, var3) => {
                        gl::Uniform3f(layout_idx as i32,
                                      context.vars[var1 as usize] as f32,
                                      context.vars[var2 as usize] as f32,
                                      context.vars[var3 as usize] as f32);
                    },

                    Vec4(layout_idx, var1, var2, var3, var4) => {
                        gl::Uniform4f(layout_idx as i32,
                                      context.vars[var1 as usize] as f32,
                                      context.vars[var2 as usize] as f32,
                                      context.vars[var3 as usize] as f32,
                                      context.vars[var4 as usize] as f32);
                    },
                }
            }
        }

        // TODO this is the same as in QuadScene::draw()

        unsafe {
            // Bind a buffer as texture
            for item in binding_to_buffers.iter() {
                use self::BufferMapping::*;
                match *item {
                    NOOP => {},

                    Sampler2D(binding_idx, buffer_idx) => {
                        if (buffer_idx as usize) < context.frame_buffers.len() {
                            if binding_idx <= gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS as u8 {
                                if let Some(fbo) = context.frame_buffers[buffer_idx as usize].fbo {
                                    gl::ActiveTexture(gl::TEXTURE0 + (binding_idx as GLuint));
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
                    },
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

            if self.indices.len() > 0 {
                // If the indices list is not empty, assume the vertices are EBO
                // indexed array.
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
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
        for mut tex in self.textures.iter_mut() {
            tex.gl_cleanup();
        }
    }
}

impl Texture {
    pub fn new(format: PixelFormat, image_data_idx: Option<usize>) -> Texture {
        Texture {
            width: 0,
            height: 0,
            format: format,
            image_data_idx: image_data_idx,
            id: None,
        }
    }

    pub fn create_texture(&mut self,
                          width: i32,
                          height: i32,
                          image: Option<&Image>)
                          -> Result<(), RuntimeError>
    {
        self.width = width;
        self.height = height;

        let format = match self.format {
            PixelFormat::NOOP => return Err(FrameBufferPixelFormatIsMissing),
            PixelFormat::RED_u8 => gl::RED,
            PixelFormat::RGB_u8 => gl::RGB,
            PixelFormat::RGBA_u8 => gl::RGBA,
        };
        let data_type = gl::UNSIGNED_BYTE;

        let mut tex_id: GLuint = 0;

        // FIXME handle case when image is None but texture has image_data_idx

        if let Some(img) = image {
            // TODO this could use image.format as well
            unsafe {
                gl::GenTextures(1, &mut tex_id);
                self.id = Some(tex_id);
                gl::BindTexture(gl::TEXTURE_2D, tex_id);

                gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32,
                               img.width as i32, img.height as i32, 0, format, data_type,
                               mem::transmute( img.raw_pixels.as_ptr() ));
                gl::GenerateMipmap(gl::TEXTURE_2D);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }
        } else {
            return Err(TexturePixelDataIsMissing);
        }

        Ok(())
    }

    pub fn bind(&self, binding_idx: u8) -> Result<(), RuntimeError> {
        if binding_idx <= gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS as u8 {
            if let Some(id) = self.id {
                unsafe {
                    // activate texture unit and bind the texture
                    gl::ActiveTexture(gl::TEXTURE0 + (binding_idx as GLuint));
                    gl::BindTexture(gl::TEXTURE_2D, id);
                }
            } else {
                return Err(NoId);
            }
        } else {
            return Err(TextureBindingIdxIsOverTheHardwareLimit);
        }
        Ok(())
    }

    pub fn gl_cleanup(&mut self) {
        if let Some(n) = self.id { unsafe { gl::DeleteTextures(1, &n); } }
        self.id = None;
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}

/// Represents instructions for building the JIT fn. We will iterate over a
/// `Vec<Operator>`.
pub enum Operator {
    /// No operation
    NOOP,
    /// Exit the main loop if time is greater than this value
    Exit(f64),
    /// Draw a quad (TriangleStrip)
    Draw_Quad_Scene(u8),
    /// If sync var equals the value, draw a quad scene.
    /// (var_idx, value, scene_idx)
    If_Var_Equal_Draw_Quad(u8, f64, u8),
    /// If sync var equals the value, draw a polygon scene.
    /// (var_idx, value, scene_idx)
    If_Var_Equal_Draw_Polygon(u8, f64, u8),
    /// ClearColor: red, green, blue, alpha as 0-255 integers
    Clear(u8, u8, u8, u8),
    /// Bind a frame buffer for rendering
    Target_Buffer(u8),
    /// Bind the default frame buffer for rendering
    Target_Buffer_Default,
    /// Record the elapsed frame time, the u8 is the text label index
    Profile_Event(u8),
}

/// Map a uniform type: (layout_idx, vars idx...)
pub enum UniformMapping {
    NOOP,
    Float(u8, u8),
    Vec2(u8, u8, u8),
    Vec3(u8, u8, u8, u8),
    Vec4(u8, u8, u8, u8, u8),
}

/// Map a frame buffer: (layout_idx, buffer_idx)
pub enum BufferMapping {
    NOOP,
    Sampler2D(u8, u8),
}

pub struct FrameBuffer {
    width: i32, // GLint = i32
    height: i32,
    kind: BufferKind,
    format: PixelFormat,
    image_data_idx: Option<usize>,
    fbo: Option<GLuint>,
    texture_buffer: Option<GLuint>,
    render_buffer: Option<GLuint>,
}

pub struct UniformBuffer {
    ubo: Option<GLuint>,
    data: SmallVec<[u8; 16]>,
    byte_size: usize,
}

impl FrameBuffer {
    pub fn new(kind: BufferKind, format: PixelFormat, image_data_idx: Option<usize>) -> FrameBuffer {
        FrameBuffer {
            width: 0,
            height: 0,
            kind: kind,
            format: format,
            image_data_idx: image_data_idx,
            fbo: None,
            texture_buffer: None,
            render_buffer: None,
        }
    }

    pub fn create_buffer(&mut self,
                         width: i32,
                         height: i32,
                         image: Option<&Image>)
                         -> Result<(), RuntimeError>
    {
        self.width = width;
        self.height = height;

        match self.kind {
            BufferKind::NOOP => return Ok(()),
            _ => {},
        }

        // start creating the framebuffer
        let mut fbo: GLuint = 0;
        let mut texture_buffer: GLuint = 0;
        let mut render_buffer: GLuint = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            // generate a texture buffer
            gl::GenTextures(1, &mut texture_buffer);
            gl::BindTexture(gl::TEXTURE_2D, texture_buffer);
        }

        let format = match self.format {
            PixelFormat::NOOP => return Err(FrameBufferPixelFormatIsMissing),
            PixelFormat::RED_u8 => gl::RED,
            PixelFormat::RGB_u8 => gl::RGB,
            PixelFormat::RGBA_u8 => gl::RGBA,
        };
        let data_type = gl::UNSIGNED_BYTE;

        // handle the cases of the framebuffer variants
        match self.kind {
            BufferKind::NOOP => {},

            BufferKind::Empty_Texture => {
                unsafe { gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32,
                                        self.width, self.height, 0, format, data_type,
                                        ptr::null()); }
            },

            BufferKind::Image_Texture => {
                if let Some(img) = image {
                    // TODO this could use image.format as well
                    unsafe { gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32,
                                            img.width as i32, img.height as i32, 0, format, data_type,
                                            mem::transmute( img.raw_pixels.as_ptr() )); }
                } else {
                    return Err(FrameBufferPixelDataIsMissing);
                }
            },
        }

        // finish the texture settings and complete the framebuffer

        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            // attach it to the framebuffer
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                                     gl::TEXTURE_2D, texture_buffer, 0);

            // generate a render buffer
            gl::GenRenderbuffers(1, &mut render_buffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, render_buffer);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            // attach it to the framebuffer
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER,
                                        gl::DEPTH_STENCIL_ATTACHMENT,
                                        gl::RENDERBUFFER,
                                        render_buffer);

            // check status
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                return Err(FrameBufferIsNotComplete);
            }

            // finished, unbind the framebuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        self.fbo = Some(fbo);
        self.texture_buffer = Some(texture_buffer);
        self.render_buffer = Some(render_buffer);

        Ok(())
    }

    pub fn bind_for_drawing(&self) {
        if let Some(fbo) = self.fbo {
            unsafe { gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo); }
        } else {
            panic!("Buffer hasn't been created");
        }
    }

    pub fn bind_for_reading(&self) {
        if let Some(fbo) = self.fbo {
            unsafe { gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo); }
        } else {
            panic!("Buffer hasn't been created");
        }
    }

    pub fn bind_as_texture(&self, binding_idx: u8) -> Result<(), RuntimeError> {
        if binding_idx <= gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS as u8 {
            if let Some(fbo) = self.fbo {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0 + (binding_idx as GLuint));
                    gl::BindTexture(gl::TEXTURE_2D, fbo);
                }
            } else {
                return Err(NoFbo);
            }
        } else {
            return Err(TextureBindingIdxIsOverTheHardwareLimit);
        }
        Ok(())
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn gl_cleanup(&mut self) {
        if let Some(n) = self.fbo            { unsafe { gl::DeleteFramebuffers(1, &n); } }
        if let Some(n) = self.texture_buffer { unsafe { gl::DeleteTextures(1, &n); } }
        if let Some(n) = self.render_buffer  { unsafe { gl::DeleteRenderbuffers(1, &n); } }

        self.fbo = None;
        self.texture_buffer = None;
        self.render_buffer = None;
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}

/// Specifies the frame buffer kind to be generated
pub enum BufferKind {
    NOOP,
    Empty_Texture,
    Image_Texture,
}

impl UniformBuffer {
    pub fn new() -> UniformBuffer {
        UniformBuffer {
            ubo: None,
            data: SmallVec::new(),
            byte_size: 0,
        }
    }

    pub fn create_buffer(&mut self, byte_size: usize) -> Result<(), RuntimeError> {
        self.byte_size = byte_size;
        for _ in 0..byte_size {
            self.data.push(0);
        }

        let mut ubo: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
            // create the uniform buffer with empty data
            gl::BufferData(gl::UNIFORM_BUFFER, byte_size as isize, ptr::null(), gl::DYNAMIC_DRAW);
            // unbind
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
        self.ubo = Some(ubo);
        Ok(())
    }

    pub fn update_buffer_data(&self) -> Result<(), RuntimeError> {
        if let Some(ubo) = self.ubo {
            unsafe {
                gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
                gl::BufferSubData(gl::UNIFORM_BUFFER,
                                  0,
                                  self.byte_size as isize,
                                  mem::transmute( self.data.as_ptr() ));
                gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
            }
        } else {
            return Err(NoUbo);
        }
        Ok(())
    }

    pub fn set_f32_array(&mut self,
                         start_offset: usize,
                         data: &SmallVec<[f32; 16]>)
                         -> Result<(), RuntimeError>
    {
        for (data_idx, n) in data.iter().enumerate() {
            // Convert n (f32) to a [u8; 4].
            // Using a different size SmallVec to suit push_f32() argument.
            let mut v: SmallVec<[u8; 64]> = SmallVec::new();
            push_f32(&mut v, *n);

            // in layout std140, a float array is padded as vec4 (16 bytes) for each item
            let n_offset = start_offset + data_idx*16;
            if (n_offset + 3) < self.data.len() {
                for i in 0..4 {
                    self.data[n_offset + i] = v[i];
                }
            } else {
                return Err(RuntimeError::DataIdxIsOutOfBounds);
            }

        }
        Ok(())
    }

    pub fn bind_as_uniform_block(&self, binding_idx: u8) -> Result<(), RuntimeError> {
        if binding_idx <= gl::MAX_UNIFORM_BUFFER_BINDINGS as u8 {
            if let Some(ubo) = self.ubo {
                unsafe {
                    gl::BindBufferBase(gl::UNIFORM_BUFFER,
                                       binding_idx as GLuint,
                                       ubo);
                }
            } else {
                unsafe { gl::BindBuffer(gl::UNIFORM_BUFFER, 0); }
                return Err(NoUbo);
            }
        } else {
            unsafe { gl::BindBuffer(gl::UNIFORM_BUFFER, 0); }
            return Err(UniformBlockBindingIdxIsOverTheHardwareLimit);
        }
        Ok(())
    }
}

impl Default for Dmo {
    fn default() -> Dmo {
        Dmo {
            context: Context::default(),
            operators: SmallVec::new(),
            sync: DmoSync::default(),
            jit_fn: JitFn::default(),
        }
    }
}

impl Default for Context {
    fn default() -> Context {
        Context::new(0.0,// time
                     1024, 768,// window width and height
                     1024, 768,// screen width and height
                     SmallVec::new(),// shader sources
                     SmallVec::new(),// images
                     SmallVec::new(),// quad scenes
                     SmallVec::new(),// polygon scenes
                     PolygonContext::default(),// polygon context
                     SmallVec::new()// frame buffers
        )
    }
}

impl Default for PolygonScene {
    fn default() -> PolygonScene {
        PolygonScene {
            scene_objects: SmallVec::new(),
        }
    }
}

impl Default for SceneObject {
    fn default() -> SceneObject {
        SceneObject {
            model_idx: 0,

            position: Vector3::from_slice(&[0.0; 3]),
            euler_rotation: Vector3::from_slice(&[0.0; 3]),
            scale: 1.0,

            position_var: ValueVec3::Fixed(0.0, 0.0, 0.0),
            euler_rotation_var: ValueVec3::Fixed(0.0, 0.0, 0.0),
            scale_var: ValueFloat::Fixed(1.0),

            layout_to_vars: SmallVec::new(),
            binding_to_buffers: SmallVec::new(),

            // identity matrix
            model_matrix: [[1.0, 0.0, 0.0, 0.0],
                           [0.0, 1.0, 0.0, 0.0],
                           [0.0, 0.0, 1.0, 0.0],
                           [0.0, 0.0, 0.0, 1.0]],
        }
    }
}

impl Default for PolygonContext {
    fn default() -> PolygonContext {
        PolygonContext {
            view_position: Vector3::from_slice(&[0.0; 3]),
            view_front: Vector3::from_slice(&[0.0; 3]),
            view_up: Vector3::new(0.0, 1.0, 0.0),

            view_matrix: [[0.0; 4]; 4],
            projection_matrix: [[0.0; 4]; 4],

            view_position_var_idx: [0; 3],
            view_front_var_idx: [0; 3],

            fovy: 0.0,
            znear: 0.0,
            zfar: 0.0,

            models: SmallVec::new(),
        }
    }
}

impl Default for Model {
    fn default() -> Model {
        Model {
            model_type: ModelType::Cube,
            meshes: SmallVec::new(),
        }
    }
}

impl Default for Mesh {
    fn default() -> Mesh {
        Mesh {
            vert_src_idx: 0,
            frag_src_idx: 0,

            vertices: SmallVec::new(),
            indices: SmallVec::new(),
            textures: SmallVec::new(),

            program: 0,

            vao: 0,
            vbo: 0,
            ebo: 0,
        }
    }
}

impl Context {
    pub fn new(time: f64,
               window_width: u32,
               window_height: u32,
               screen_width: u32,
               screen_height: u32,
               shader_sources: SmallVec<[SmallVec<[u8; 1024]>; 64]>,
               images: SmallVec<[Image; 4]>,
               quad_scenes: SmallVec<[QuadScene; 64]>,
               polygon_scenes: SmallVec<[PolygonScene; 64]>,
               polygon_context: PolygonContext,
               frame_buffers: SmallVec<[FrameBuffer; 64]>) -> Context {

        let mut vars: [f64; VAR_NUM] = [0.0; VAR_NUM];
        vars[0] = time;
        vars[1] = window_width as f64;
        vars[2] = window_height as f64;
        vars[3] = screen_width as f64;
        vars[4] = screen_height as f64;

        let empty_profile: [[f32; PROFILE_EVENTS]; PROFILE_FRAMES] = [[0.0; PROFILE_EVENTS]; PROFILE_FRAMES];

        Context {
            vars: vars,
            shader_sources: shader_sources,
            images: images,
            quad_scenes: quad_scenes,
            polygon_scenes: polygon_scenes,
            polygon_context: polygon_context,
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
        self.vars[0] = time;
    }

    pub fn get_time(&self) -> f64 {
        self.vars[0]
    }

    pub fn set_window_resolution(&mut self, width: f64, height: f64) {
        self.vars[1] = width;
        self.vars[2] = height;
    }

    pub fn get_window_resolution(&self) -> (f64, f64) {
        // 1: width, 2: height
        (self.vars[1], self.vars[2])
    }

    pub fn set_screen_resolution(&mut self, width: f64, height: f64) {
        self.vars[3] = width;
        self.vars[4] = height;
    }

    pub fn get_screen_resolution(&self) -> (f64, f64) {
        // 3: width, 4: height
        (self.vars[3], self.vars[4])
    }

    pub fn get_last_work_buffer(&self) -> &FrameBuffer {
        let n = self.frame_buffers.len();
        &self.frame_buffers[n - 1]
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

    pub fn impl_draw_polygon_scene(&self, scene_idx: usize) {
        if let Some(ref scene) = self.polygon_scenes.get(scene_idx) {
            scene.draw(&self).unwrap();
        } else {
            panic!("Polygon scene index doesn't exist: {}", scene_idx);
        }
    }

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

impl Drop for Context {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}

impl Dmo {
    /// Two steps are necessary. After `new()`, call `.build_jit_fn()`.
    pub fn new(context: Context, operators: SmallVec<[Operator; 64]>, sync: DmoSync) -> Dmo {
        Dmo {
            context: context,
            operators: operators,
            sync: sync,
            jit_fn: JitFn::default(),
        }
    }

    pub fn set_operators_and_build_jit(&mut self, operators: SmallVec<[Operator; 64]>) {
        self.operators = operators;
        self.build_jit_fn();
    }

    pub fn set_context_var(&mut self, idx: usize, value: f64) -> Result<(), RuntimeError> {
        if idx < VAR_NUM {
            self.context.vars[idx] = value;
        } else {
            return Err(ContextIndexIsOutOfBounds);
        }
        Ok(())
    }

    pub fn get_context_var(&self, idx: usize) -> Result<f64, RuntimeError> {
        if idx < VAR_NUM {
            return Ok(self.context.vars[idx]);
        } else {
            return Err(ContextIndexIsOutOfBounds);
        }
    }

    pub fn get_profile_frame_idx(&self) -> usize {
        self.context.profile_frame_idx
    }

    pub fn get_profile_event_idx(&self) -> usize {
        self.context.profile_event_idx
    }

    pub fn get_max_profile_event_idx(&self) -> usize {
        self.context.max_profile_event_idx
    }

    pub fn get_profile_times_ptr(&self) -> &[[f32; PROFILE_EVENTS]; PROFILE_FRAMES] {
        &self.context.profile_times
    }

    // FIXME refactor to error type, there can be 0 framebuffers

    pub fn get_last_work_buffer(&self) -> &FrameBuffer {
        self.context.get_last_work_buffer()
    }

    /// This must happen after `dmo` is assigned, so that the JIT is
    /// built with the pointer address of the new `dmo.context`.
    pub fn build_jit_fn(&mut self) {
        self.jit_fn = JitFn::new(1, &mut self.context, &self.operators);
    }

    pub fn run_jit_fn(&mut self) {
        self.jit_fn.run(&mut self.context)
    }

    pub fn create_quads(&mut self,
                        err_msg_buf: &mut [u8; ERR_MSG_LEN])
                        -> Result<(), RuntimeError>
    {
        for mut scene in self.context.quad_scenes.iter_mut() {

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

    pub fn create_models(&mut self,
                         err_msg_buf: &mut [u8; ERR_MSG_LEN])
                         -> Result<(), RuntimeError>
    {
        for mut model in self.context.polygon_context.models.iter_mut() {

            let mut new_meshes: SmallVec<[Mesh; 2]> = SmallVec::new();

            for mut mesh in model.meshes.iter_mut() {
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

    pub fn compile_quad_scene(&mut self, scene_idx: usize, err_msg_buf: &mut [u8; ERR_MSG_LEN])
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

    pub fn compile_model_shaders(&mut self, model_idx: usize, err_msg_buf: &mut [u8; ERR_MSG_LEN])
        -> Result<(), RuntimeError>
    {
        for mut mesh in self.context.polygon_context.models[model_idx].meshes.iter_mut() {
            let ref s = self.context.shader_sources[mesh.vert_src_idx];
            let vert_src = str::from_utf8(s).unwrap();
            let ref s = self.context.shader_sources[mesh.frag_src_idx];
            let frag_src = str::from_utf8(s).unwrap();
            mesh.compile_shaders(vert_src, frag_src, err_msg_buf)?;
        }
        Ok(())
    }

    pub fn create_frame_buffers(&mut self) -> Result<(), RuntimeError> {
        let (wx, wy) = self.context.get_window_resolution();

        for mut buffer in self.context.frame_buffers.iter_mut() {
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
        for mut buffer in  self.context.frame_buffers.iter_mut() {
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

    pub fn get_is_running(&self) -> bool {
        self.context.is_running
    }

    pub fn set_is_running(&mut self, value: bool) {
        self.context.is_running = value;
    }

    pub fn set_window_resolution(&mut self, width: f64, height: f64) {
        self.context.set_window_resolution(width, height);
    }

    pub fn get_window_resolution(&self) -> (f64, f64) {
        self.context.get_window_resolution()
    }

    pub fn set_screen_resolution(&mut self, width: f64, height: f64) {
        self.context.set_screen_resolution(width, height);
    }

    pub fn get_screen_resolution(&self) -> (f64, f64) {
        self.context.get_screen_resolution()
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

    pub fn update_polygon_context_projection(&mut self, aspect: f32) {
        self.context.polygon_context.update_projection_matrix(aspect);
    }

    /// Update global view and projection matrix of the PolygonContext, and
    /// update individual model matrices.
    pub fn update_polygon_context(&mut self) {

        let v = self.context.polygon_context.view_position_var_idx;

        self.context.polygon_context.view_position =
            Vector3::new(self.context.vars[v[0]] as f32,
                         self.context.vars[v[1]] as f32,
                         self.context.vars[v[2]] as f32);

        let v = self.context.polygon_context.view_front_var_idx;

        self.context.polygon_context.view_front =
            Vector3::new(self.context.vars[v[0]] as f32,
                         self.context.vars[v[1]] as f32,
                         self.context.vars[v[2]] as f32);

        self.context.polygon_context.update_view_matrix();

        for mut scene in self.context.polygon_scenes.iter_mut() {
            for mut scene_object in scene.scene_objects.iter_mut() {
                match scene_object.position_var {
                    ValueVec3::NOOP => {},
                    ValueVec3::Sync(x, y, z) => {
                        scene_object.position =
                            Vector3::new(self.context.vars[x as usize] as f32,
                                         self.context.vars[y as usize] as f32,
                                         self.context.vars[z as usize] as f32);
                    },
                    ValueVec3::Fixed(x, y, z) => {
                        scene_object.position = Vector3::new(x, y, z);
                    },
                }

                match scene_object.euler_rotation_var {
                    ValueVec3::NOOP => {},
                    ValueVec3::Sync(x, y, z) => {
                        scene_object.euler_rotation =
                            Vector3::new(to_radians(self.context.vars[x as usize] as f32),
                                         to_radians(self.context.vars[y as usize] as f32),
                                         to_radians(self.context.vars[z as usize] as f32));
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
                        scene_object.scale = self.context.vars[x as usize] as f32;
                    },
                    ValueFloat::Fixed(x) => {
                        scene_object.scale = x;
                    },
                }

                scene_object.update_model_matrix();
            }
        }
    }
}

impl QuadScene {
    pub fn new(id: u8, vert_src_idx: usize, frag_src_idx: usize) -> QuadScene {
        QuadScene {
            id: id,
            vert_src_idx: vert_src_idx,
            frag_src_idx: frag_src_idx,
            layout_to_vars: SmallVec::new(),
            binding_to_buffers: SmallVec::new(),
            quad: None,
        }
    }

    pub fn create_quad(&mut self, vert_src: &str, frag_src: &str, err_msg_buf: &mut [u8; ERR_MSG_LEN])
                       -> Result<(), RuntimeError>
    {
        self.quad = Some(Quad::new(vert_src, frag_src, err_msg_buf)?);
        Ok(())
    }

    pub fn draw(&self, context: &Context) -> Result<(), RuntimeError> {
        match self.quad {
            Some(ref quad) => { unsafe {
                // Use shader
                gl::UseProgram(quad.program);

                // Mapping sync var indexes to uniform layout indexes
                for item in self.layout_to_vars.iter() {
                    use self::UniformMapping::*;
                    match *item {
                        NOOP => {},

                        Float(layout_idx, var_idx) => {
                            gl::Uniform1f(layout_idx as i32,
                                          context.vars[var_idx as usize] as f32);
                        },

                        Vec2(layout_idx, var1, var2) => {
                            gl::Uniform2f(layout_idx as i32,
                                          context.vars[var1 as usize] as f32,
                                          context.vars[var2 as usize] as f32);
                        },

                        Vec3(layout_idx, var1, var2, var3) => {
                            gl::Uniform3f(layout_idx as i32,
                                          context.vars[var1 as usize] as f32,
                                          context.vars[var2 as usize] as f32,
                                          context.vars[var3 as usize] as f32);
                        },

                        Vec4(layout_idx, var1, var2, var3, var4) => {
                            gl::Uniform4f(layout_idx as i32,
                                          context.vars[var1 as usize] as f32,
                                          context.vars[var2 as usize] as f32,
                                          context.vars[var3 as usize] as f32,
                                          context.vars[var4 as usize] as f32);
                        },
                    }
                }

                // Bind a buffer as texture
                for item in self.binding_to_buffers.iter() {
                    use self::BufferMapping::*;
                    match *item {
                        NOOP => {},

                        Sampler2D(binding_idx, buffer_idx) => {
                            if (buffer_idx as usize) < context.frame_buffers.len() {
                                if binding_idx <= gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS as u8 {
                                    if let Some(fbo) = context.frame_buffers[buffer_idx as usize].fbo {
                                        gl::ActiveTexture(gl::TEXTURE0 + (binding_idx as GLuint));
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
                        },
                    }
                }

                gl::BindVertexArray(quad.vao);
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                gl::BindVertexArray(0);
            } },

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
    pub fn new(vert_src: &str, frag_src: &str, err_msg_buf: &mut [u8; ERR_MSG_LEN]) -> Result<Quad, RuntimeError> {
        let vs = compile_shader(vert_src, gl::VERTEX_SHADER, err_msg_buf)?;
        let fs = compile_shader(frag_src, gl::FRAGMENT_SHADER, err_msg_buf)?;
        let program = link_program(vs, fs, err_msg_buf)?;

        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            // VAO
            gl::BindVertexArray(vao);

            // VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (QUAD_VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                           mem::transmute(&QUAD_VERTICES[0]),
                           gl::STATIC_DRAW);

            // Vertex data

            // layout (location = 0) in vec2 pos;
            let pos_attr: GLuint = 0;
            gl::EnableVertexAttribArray(pos_attr);
            gl::VertexAttribPointer(pos_attr, 2, gl::FLOAT, gl::FALSE,
                                    4 * mem::size_of::<GLfloat>() as GLsizei,
                                    ptr::null());

            // layout (location = 1) in vec2 tex;
            let tex_attr: GLuint = 1;
            gl::EnableVertexAttribArray(tex_attr);
            gl::VertexAttribPointer(tex_attr, 2, gl::FLOAT, gl::FALSE,
                                    4 * mem::size_of::<GLfloat>() as GLsizei,
                                    mem::transmute(2 * mem::size_of::<GLfloat>()));

            gl::BindVertexArray(0);
        }

        Ok(Quad {
            program: program,
            vao: vao,
            vbo: vbo,
        })
    }

    pub fn compile_program(&mut self, vert_src: &str, frag_src: &str, err_msg_buf: &mut [u8; ERR_MSG_LEN])
                           -> Result<(), RuntimeError>
    {
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

impl PolygonContext {
    pub fn update_view_matrix(&mut self) {
        let m = Matrix4::look_at_rh(&self.view_position,
                                    &{&self.view_position + &self.view_front},
                                    &self.view_up);
        self.view_matrix = m.as_column_slice();
    }

    pub fn update_projection_matrix(&mut self, aspect: f32) {
        let a = Matrix4::new_perspective(aspect,
                                         to_radians(self.fovy),
                                         self.znear,
                                         self.zfar);
        self.projection_matrix = a.as_column_slice();
    }
}

impl PolygonScene {
    pub fn draw(&self, context: &Context) -> Result<(), RuntimeError> {
        for o in self.scene_objects.iter() {
            if let Some(ref model) = context.polygon_context.models.get(o.model_idx) {
                model.draw(context,
                           &o.layout_to_vars,
                           &o.binding_to_buffers,
                           &o.model_matrix,
                           &context.polygon_context.view_matrix,
                           &context.polygon_context.projection_matrix,
                           &context.polygon_context.view_position.as_slice())?;
            }
        }
        Ok(())
    }
}

impl SceneObject {
    pub fn update_model_matrix(&mut self) {
        let a = Matrix4::new_homogeneous(&self.position,
                                         &self.euler_rotation,
                                         self.scale);
        self.model_matrix = a.as_column_slice();
    }
}
