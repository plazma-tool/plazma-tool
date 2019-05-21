use core::{ptr, str};
use smallvec::SmallVec;

use gl;
use gl::types::*;

use crate::ERR_MSG_LEN;
use crate::error::RuntimeError;
use crate::error::RuntimeError::*;

pub fn compile_shader(src: &str,
                      ty: GLenum,
                      err_msg_buf: &mut [u8; ERR_MSG_LEN])
                      -> Result<GLuint, RuntimeError>
{
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader

        // have to take the length so that we don't have to pass it as a null-terminated c-string
        let len = src.len() as i32;
        gl::ShaderSource(shader, 1, [src.as_ptr() as *const i8].as_ptr(), [len].as_ptr());

        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf: SmallVec<[u8; 1024]> = SmallVec::new();
            // Takes current size (i.e. 0) and grows the vec for additional items.
            buf.reserve((len as usize) - 1); // subtract 1 to skip the trailing null character
            // Sets the current size.
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

            let mut n = 0;
            for i in buf.iter() {
                if n < err_msg_buf.len() {
                    err_msg_buf[n] = *i;
                }
                n += 1;
            }

            return Err(ShaderCompilationFailed);
        }
    }
    Ok(shader)
}

pub fn link_program(vs: GLuint,
                    fs: GLuint,
                    err_msg_buf: &mut [u8; ERR_MSG_LEN])
                    -> Result<GLuint, RuntimeError>
{
    let program;
    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf: SmallVec<[u8; 1024]> = SmallVec::new();
            // Takes current size (i.e. 0) and grows the vec for additional items.
            buf.reserve((len as usize) - 1); // subtract 1 to skip the trailing null character
            // Sets the current size.
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

            let mut n = 0;
            for i in buf.iter() {
                if n < err_msg_buf.len() {
                    err_msg_buf[n] = *i;
                }
                n += 1;
            }

            return Err(ShaderLinkingFailed);
        }

        // Shaders are linked into the program, they can be deleted. Meshes only
        // have to store the program id.
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
    }
    Ok(program)
}
