#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("zipeek-web runs in a browser; build for wasm32-unknown-unknown");
}

#[cfg(target_arch = "wasm32")]
use wasm_logger::{Config, init};
#[cfg(target_arch = "wasm32")]
use yew::Renderer;
#[cfg(target_arch = "wasm32")]
use zipeek_web::*;

#[cfg(target_arch = "wasm32")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    console_error_panic_hook::set_once();
    if mount_reset_if_requested().map_err(|message| {
        show_fatal_error(&message);
        message
    })? {
        return Ok(());
    }
    wasm_bindgen_futures::spawn_local(async {
        if let Err(message) = acquire_app_storage_session().await {
            show_fatal_error(&message);
            return;
        }
        if let Err(error) = start_application() {
            show_fatal_error(&error.to_string());
        }
    });
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn start_application() -> Result<(), Box<dyn std::error::Error>> {
    let environment = AppEnvironment::new().map_err(|error| {
        show_fatal_error(&format!("Application environment failed: {error}"));
        error
    })?;
    let level = environment
        .try_get_log_level_from_local_storage("RUST_LOG")
        .unwrap_or(if cfg!(debug_assertions) {
            log::Level::Debug
        } else {
            log::Level::Info
        });
    init(Config::new(level));

    setup(&environment);
    Renderer::<App>::with_root_and_props(environment.root.clone().into(), environment).render();
    Ok(())
}
