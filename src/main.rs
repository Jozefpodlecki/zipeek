mod app;
mod components;
mod services;
mod models;
mod pages;
mod contexts;
mod route;

use std::error::Error;

use app::App;
use log::Level;
use wasm_logger::{init, Config};
use web_sys::window;
use yew::Renderer;

use crate::app::AppProps;

fn main() -> Result<(), Box<dyn Error>> {
    let mut log_level = if cfg!(debug_assertions) {
        Level::Debug
    } else {
        Level::Error
    };

    let window = window().expect("Windows object not found");
    let local_storage = window.local_storage().ok().flatten().expect("Local storage not found");
    let document = window.document().expect("Document object not found");
    let body = document.body().expect("Document body not found");
    let navigator = window.navigator();
    
    let log_level_str = local_storage
        .get_item("RUST_LOG")
        .ok()
        .flatten();

    if let Some(level_str) = log_level_str {
        log_level = match level_str.to_lowercase().as_str() {
            "error" => Level::Error,
            "warn" | "warning" => Level::Warn,
            "info" => Level::Info,
            "debug" => Level::Debug,
            "trace" => Level::Trace,
            _ => log_level,
        };
    }

    init(Config::new(log_level));
    console_error_panic_hook::set_once();
    
    let app_name = env!("CARGO_PKG_NAME").into();
    let version = env!("CARGO_PKG_VERSION").into();

    let props = AppProps {
        window,
        document,
        body: body.clone(),
        local_storage,
        navigator,
        app_name,
        version
    };

    let renderer = Renderer::<App>::with_root_and_props(body.into(), props);
    renderer.render();

    Ok(())
}
