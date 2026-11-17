use super::tauri_ipc_error::message as error_message;
use js_sys::{Function, Object, Promise, Reflect};
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatus {
    pub service: String,
    pub label: String,
    pub phase: String,
    pub running: bool,
    pub healthy: bool,
    pub owned: bool,
    pub pid: Option<u32>,
    pub message: String,
}

async fn invoke(command: &str, service: &str) -> Result<ServiceStatus, String> {
    let window = window().ok_or("Browser window unavailable")?;
    let tauri =
        Reflect::get(window.as_ref(), &JsValue::from_str("__TAURI__")).map_err(error_message)?;
    if !tauri.is_object() {
        return Err("Service control is only available in the Tauri desktop app".into());
    }
    let core = Reflect::get(&tauri, &JsValue::from_str("core")).map_err(error_message)?;
    let function = Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(error_message)?
        .dyn_into::<Function>()
        .map_err(|_| "Tauri invoke function unavailable".to_owned())?;
    let args = Object::new();
    Reflect::set(
        &args,
        &JsValue::from_str("service"),
        &JsValue::from_str(service),
    )
    .map_err(error_message)?;
    let promise = function
        .call2(&core, &JsValue::from_str(command), &args)
        .map_err(error_message)?
        .dyn_into::<Promise>()
        .map_err(|_| "Tauri invoke did not return a Promise".to_owned())?;
    let result = JsFuture::from(promise).await.map_err(error_message)?;
    serde_wasm_bindgen::from_value(result).map_err(|error| error.to_string())
}

pub async fn start(service: &str) -> Result<ServiceStatus, String> {
    invoke("start_service", service).await
}

pub async fn stop(service: &str) -> Result<ServiceStatus, String> {
    invoke("stop_service", service).await
}

pub async fn status(service: &str) -> Result<ServiceStatus, String> {
    invoke("service_status", service).await
}
