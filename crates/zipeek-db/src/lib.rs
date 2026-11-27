#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

pub mod contracts;
mod evidence_summary;

pub use evidence_summary::{EvidenceError, PracticeOutcomeSummary};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod native;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use native::{DbError, GeneratorDatabase};

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod js_bridge {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(inline_js = r#"
        export async function loadModule(url) {
            return import(url);
        }
    "#)]
    unsafe extern "C" {
        #[wasm_bindgen(catch, js_name = loadModule)]
        async fn js_load_module(url: &str) -> Result<JsValue, JsValue>;
    }

    pub async fn load_module(url: &str) -> Result<JsValue, JsValue> {
        unsafe { js_load_module(url).await }
    }
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use js_bridge::load_module;
