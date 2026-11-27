#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(target_arch = "wasm32")]
mod error;
#[cfg(target_arch = "wasm32")]
mod evidence;
#[cfg(target_arch = "wasm32")]
mod runtime;
#[cfg(target_arch = "wasm32")]
mod wire;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("The pedagogy worker requires wasm32-unknown-unknown");
}

#[cfg(target_arch = "wasm32")]
fn main() {}
