#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

pub mod document;

#[cfg(target_arch = "wasm32")]
pub mod microphone;

#[cfg(target_arch = "wasm32")]
pub mod pedagogy_worker;

pub mod speech_wav;

#[cfg(target_arch = "wasm32")]
pub mod local_speech;

#[cfg(target_arch = "wasm32")]
pub mod charts;

#[cfg(target_arch = "wasm32")]
pub mod ai_configuration;

#[cfg(target_arch = "wasm32")]
pub mod clipboard;

#[cfg(target_arch = "wasm32")]
pub mod recognition_workspace;

pub mod recognition_state;
