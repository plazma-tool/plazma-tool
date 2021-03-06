use std::{ptr, str, ffi};

use gl;
use gl::types::*;

use crate::error::RuntimeError;
use crate::error::RuntimeError::*;
use crate::ERR_MSG_LEN;

pub fn compile_shader(
    src: &str,
    ty: GLenum,
    err_msg_buf: &mut [u8; ERR_MSG_LEN],
) -> Result<GLuint, RuntimeError> {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);

        // Attempt to compile the shader

        let shader_source = ffi::CString::new(src.as_bytes()).unwrap();

        gl::ShaderSource(
            shader,
            1,
            [shader_source.as_ptr()].as_ptr(),
            ptr::null(),
        );

        gl::CompileShader(shader);

        // Get the compile status
        let mut status = i32::from(gl::FALSE); // i32 = GLint
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != i32::from(gl::TRUE) {
            let mut len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf: Vec<u8> = Vec::new();
            // Takes current size (i.e. 0) and grows the vec for additional items.
            buf.reserve((len as usize) - 1); // subtract 1 to skip the trailing null character
                                             // Sets the current size.
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            for (n, i) in buf.iter().enumerate() {
                if n < err_msg_buf.len() {
                    err_msg_buf[n] = *i;
                }
            }

            return Err(ShaderCompilationFailed);
        }
    }
    Ok(shader)
}

pub fn link_program(
    vs: GLuint,
    fs: GLuint,
    err_msg_buf: &mut [u8; ERR_MSG_LEN],
) -> Result<GLuint, RuntimeError> {
    let program;
    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = i32::from(gl::FALSE); // i32 = GLint
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != i32::from(gl::TRUE) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf: Vec<u8> = Vec::new();
            // Takes current size (i.e. 0) and grows the vec for additional items.
            buf.reserve((len as usize) - 1); // subtract 1 to skip the trailing null character
                                             // Sets the current size.
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            for (n, i) in buf.iter().enumerate() {
                if n < err_msg_buf.len() {
                    err_msg_buf[n] = *i;
                }
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
