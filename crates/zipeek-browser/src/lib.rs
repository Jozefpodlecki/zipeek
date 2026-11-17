#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]

pub mod document;

#[cfg(target_arch = "wasm32")]
pub mod microphone;

#[cfg(target_arch = "wasm32")]
pub mod pedagogy_worker;
