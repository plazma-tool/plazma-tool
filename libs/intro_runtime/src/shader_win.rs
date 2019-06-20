use std::{ptr, str};
use std::ffi::CString;
use std::convert::TryInto;

use gl;
use gl::types::*;

use crate::error::RuntimeError;
use crate::error::RuntimeError::*;
use crate::ERR_MSG_LEN;

// Shaders fail on Windows in one chunk somewhere between 19832 - 22153 bytes
// 2^14 = 16384
// 2^15 = 32768
const MAX_CHUNK_SIZE: usize = 16383;// 2^14 - 1

pub fn compile_shader(
    src: &str,
    ty: GLenum,
    err_msg_buf: &mut [u8; ERR_MSG_LEN],
) -> Result<GLuint, RuntimeError> {
    println!("compile_shader() begin");
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader

        println!("len: {}", src.len());
        
        if src.len() < MAX_CHUNK_SIZE {
            let src_cstring = match CString::new(src) {
                Ok(x) => x,
                Err(_) => return Err(ShaderCompilationFailed),
            };
        
            println!("ShaderSource() one chunk");
            gl::ShaderSource(
                shader,
                1,
                [src_cstring.as_ptr() as *const i8].as_ptr(),
                [i32::from(-1)].as_ptr(),
            );
        } else {
            
            // construct chunks of nul-terminated c-strings, max 16k in size, src should not contain '\0'
            let a = src.split("\n").map(|i| {
                let mut s = String::from(i);
                s.push('\n');
                CString::new(s).unwrap()
            }).collect::<Vec<_>>();
            let chunks = a.iter().map(|i| i.as_ptr() as *const i8).collect::<Vec<_>>();
            //let chunks: Vec<*const i8> = a.map(|i| i.as_ptr() as *const i8).collect::<Vec<_>>();

            // will be all -1 since we are nul-terminating
            let lengths: Vec<i32> = src.split("\n").map(|i| i32::from(-1)).collect::<Vec<_>>();

            println!("ShaderSource() {} chunks", chunks.len());
            gl::ShaderSource(
                shader,
                chunks.len().try_into().unwrap(),
                chunks.as_ptr(),
                lengths.as_ptr(),
            );

        }

        println!("CompileShader()");
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = i32::from(gl::FALSE); // i32 = GLint
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        
        println!("status: {}", status);
        
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
    println!("compile_shader() return");
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
