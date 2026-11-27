#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

#[cfg(target_arch = "wasm32")]
pub mod hanzi_references;

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
pub mod appearance;
#[cfg(target_arch = "wasm32")]
pub mod components;
#[cfg(target_arch = "wasm32")]
pub mod contexts;
#[cfg(target_arch = "wasm32")]
mod env;
#[cfg(target_arch = "wasm32")]
mod font_preference;
#[cfg(target_arch = "wasm32")]
pub mod hooks;
#[cfg(target_arch = "wasm32")]
pub mod models;
#[cfg(target_arch = "wasm32")]
pub mod pages;
#[cfg(target_arch = "wasm32")]
mod preferences;
#[cfg(target_arch = "wasm32")]
mod reducers;
#[cfg(target_arch = "wasm32")]
pub mod route;
#[cfg(target_arch = "wasm32")]
mod services;
#[cfg(target_arch = "wasm32")]
mod setup;
#[cfg(target_arch = "wasm32")]
pub mod theme;

#[cfg(target_arch = "wasm32")]
pub use app::*;
#[cfg(target_arch = "wasm32")]
pub use env::*;
#[cfg(target_arch = "wasm32")]
pub use services::app_reset::{
    acquire_app_storage_session, mount_if_requested as mount_reset_if_requested,
};
#[cfg(target_arch = "wasm32")]
pub use setup::{setup, show_fatal_error};
