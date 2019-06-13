use std::{mem, ptr};

use gl;
use gl::types::*;

use crate::error::RuntimeError;
use crate::error::RuntimeError::*;
use crate::types::{Image, PixelFormat};

pub struct FrameBuffer {
    width: i32, // GLint = i32
    height: i32,
    kind: BufferKind,
    format: PixelFormat,
    pub image_data_idx: Option<usize>,
    pub fbo: Option<GLuint>,
    texture_buffer: Option<GLuint>,
    render_buffer: Option<GLuint>,
}

impl FrameBuffer {
    pub fn new(
        kind: BufferKind,
        format: PixelFormat,
        image_data_idx: Option<usize>,
    ) -> FrameBuffer {
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

    pub fn create_buffer(
        &mut self,
        width: i32,
        height: i32,
        image: Option<&Image>,
    ) -> Result<(), RuntimeError> {
        self.width = width;
        self.height = height;

        match self.kind {
            BufferKind::NOOP => return Ok(()),
            _ => {}
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
            BufferKind::NOOP => {}

            BufferKind::Empty_Texture => unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    format as i32,
                    self.width,
                    self.height,
                    0,
                    format,
                    data_type,
                    ptr::null(),
                );
            },

            BufferKind::Image_Texture => {
                if let Some(img) = image {
                    // TODO this could use image.format as well
                    unsafe {
                        gl::TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            format as i32,
                            img.width as i32,
                            img.height as i32,
                            0,
                            format,
                            data_type,
                            mem::transmute(img.raw_pixels.as_ptr()),
                        );
                    }
                } else {
                    return Err(FrameBufferPixelDataIsMissing);
                }
            }
        }

        // finish the texture settings and complete the framebuffer

        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            // attach it to the framebuffer
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture_buffer,
                0,
            );

            // generate a render buffer
            gl::GenRenderbuffers(1, &mut render_buffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, render_buffer);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            // attach it to the framebuffer
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                render_buffer,
            );

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
            unsafe {
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo);
            }
        } else {
            panic!("Buffer hasn't been created");
        }
    }

    pub fn bind_for_reading(&self) {
        if let Some(fbo) = self.fbo {
            unsafe {
                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            }
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
        if let Some(n) = self.fbo {
            unsafe {
                gl::DeleteFramebuffers(1, &n);
            }
        }
        if let Some(n) = self.texture_buffer {
            unsafe {
                gl::DeleteTextures(1, &n);
            }
        }
        if let Some(n) = self.render_buffer {
            unsafe {
                gl::DeleteRenderbuffers(1, &n);
            }
        }

        self.fbo = None;
        self.texture_buffer = None;
        self.render_buffer = None;
    }
}

/// Specifies the frame buffer kind to be generated
pub enum BufferKind {
    NOOP,
    Empty_Texture,
    Image_Texture,
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}
