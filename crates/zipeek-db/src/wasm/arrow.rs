use super::{DbError, error::js_error};
use crate::load_module;
use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub struct Arrow(JsValue);

impl Arrow {
    pub async fn load(url: &str) -> Result<Self, DbError> {
        let module = load_module(url)
            .await
            .map_err(|error| js_error("Loading Arrow", error))?;
        let initialize = Reflect::get(&module, &JsValue::from_str("initializeArrow"))
            .map_err(|error| js_error("Resolving Arrow initializer", error))?
            .dyn_into::<Function>()
            .map_err(|error| js_error("Accessing Arrow initializer", error))?;
        let result = initialize
            .call0(&module)
            .map_err(|error| js_error("Initializing Arrow", error))?;
        JsFuture::from(Promise::resolve(&result))
            .await
            .map_err(|error| js_error("Initializing Arrow", error))?;
        Ok(Self(module))
    }

    pub fn module(&self) -> &JsValue {
        &self.0
    }
}
