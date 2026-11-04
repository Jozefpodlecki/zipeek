use wasm_logger::{Config, init};
use yew::Renderer;
use zipeek::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    console_error_panic_hook::set_once();
    let environment = AppEnvironment::new()?;
    let level = environment
        .try_get_log_level_from_local_storage("RUST_LOG")
        .unwrap_or(if cfg!(debug_assertions) {
            log::Level::Debug
        } else {
            log::Level::Info
        });
    init(Config::new(level));

    Renderer::<App>::with_root_and_props(environment.root.clone().into(), environment).render();
    Ok(())
}
