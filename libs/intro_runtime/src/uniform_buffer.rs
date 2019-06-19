#[cfg(any(target_os = "linux", target_os = "macos"))]
use libc::c_void;

#[cfg(target_os = "windows")]
use winapi::ctypes::c_void;

use std::ptr;

use gl;
use gl::types::*;

use crate::data_blob::push_f32;
use crate::error::RuntimeError;
use crate::error::RuntimeError::*;

pub struct UniformBuffer {
    ubo: Option<GLuint>,
    data: Vec<u8>,
    byte_size: usize,
}

impl Default for UniformBuffer {
    fn default() -> UniformBuffer {
        UniformBuffer {
            ubo: None,
            data: Vec::new(),
            byte_size: 0,
        }
    }
}

impl UniformBuffer {
    pub fn new() -> UniformBuffer {
        UniformBuffer::default()
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
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                byte_size as isize,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
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
                gl::BufferSubData(
                    gl::UNIFORM_BUFFER,
                    0,
                    self.byte_size as isize,
                    self.data.as_ptr() as *const c_void,
                );
                gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
            }
        } else {
            return Err(NoUbo);
        }
        Ok(())
    }

    pub fn set_f32_array(&mut self, start_offset: usize, data: &[f32]) -> Result<(), RuntimeError> {
        for (data_idx, n) in data.iter().enumerate() {
            // Convert n (f32) to a [u8; 4].
            // Using a different size (Small)Vec to suit push_f32() argument.
            let mut v: Vec<u8> = Vec::new();
            push_f32(&mut v, *n);

            // in layout std140, a float array is padded as vec4 (16 bytes) for each item
            let n_offset = start_offset + data_idx * 16;
            if (n_offset + 3) < self.data.len() {
                self.data[n_offset..(4 + n_offset)].clone_from_slice(&v[..4])
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
                    gl::BindBufferBase(gl::UNIFORM_BUFFER, u32::from(binding_idx), ubo); // u32 = GLuint
                }
            } else {
                unsafe {
                    gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
                }
                return Err(NoUbo);
            }
        } else {
            unsafe {
                gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
            }
            return Err(UniformBlockBindingIdxIsOverTheHardwareLimit);
        }
        Ok(())
    }
}
