use core::mem;

use gl;
use gl::types::*;

use crate::error::RuntimeError;
use crate::error::RuntimeError::*;
use crate::types::{Image, PixelFormat};

#[derive(Clone)]
pub struct Texture {
    width: i32,
    height: i32,
    format: PixelFormat,
    image_data_idx: Option<usize>,
    id: Option<GLuint>,
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

    pub fn create_texture(
        &mut self,
        width: i32,
        height: i32,
        image: Option<&Image>,
    ) -> Result<(), RuntimeError> {
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
                gl::GenerateMipmap(gl::TEXTURE_2D);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as i32,
                );
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
        if let Some(n) = self.id {
            unsafe {
                gl::DeleteTextures(1, &n);
            }
        }
        self.id = None;
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}
