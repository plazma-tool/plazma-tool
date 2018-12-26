// FIXME need the stdlib for a println!() on Windows
//#![no_std]
#![feature(link_llvm_intrinsics)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

#![cfg(all(any(target_os = "linux", target_os = "macos", target_os = "windows"), target_arch = "x86_64"))]

extern crate core;// only when testing without no_std

#[cfg(any(target_os = "linux", target_os = "macos"))]
extern crate libc;

#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;

extern crate gl;
extern crate smallvec;
extern crate rocket_sync;
extern crate intro_3d;

pub mod dmo;
pub mod shapes;
pub mod bytecode;
pub mod sync;
pub mod shader;
pub mod jit;
pub mod error;
