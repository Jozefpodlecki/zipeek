#[cfg(target_arch = "wasm32")]
mod browser;
mod error;
mod evidence;
mod runtime;
mod wire;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("The pedagogy worker requires wasm32-unknown-unknown");
}

#[cfg(target_arch = "wasm32")]
fn main() {}
