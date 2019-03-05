//#![no_std]
// Holding up no_std:
// - std::time in context_gfx

#![allow(non_camel_case_types)]

#![cfg(all(any(target_os = "linux", target_os = "macos", target_os = "windows"), target_arch = "x86_64"))]

extern crate core;// NOTE only when testing without no_std

#[cfg(any(target_os = "linux", target_os = "macos"))]
extern crate libc;

//#[cfg(target_os = "windows")]
//extern crate winapi;
//#[cfg(target_os = "windows")]
//extern crate kernel32;

extern crate gl;
extern crate smallvec;
extern crate rocket_sync;
extern crate intro_3d;

pub const VAR_NUM: usize = 2048;
pub const ERR_MSG_LEN: usize = 1024;

pub mod dmo_gfx;
pub mod context_gfx;

pub mod dmo_sync;
pub mod sync_vars;

pub mod quad_scene_gfx;

pub mod polygon_context;
pub mod polygon_scene;
pub mod timeline;
pub mod camera;
pub mod mouse;
pub mod model;
pub mod mesh;

pub mod frame_buffer;
pub mod uniform_buffer;
pub mod texture;

pub mod data_blob;
pub mod shapes;
pub mod shader;
pub mod types;
pub mod error;
