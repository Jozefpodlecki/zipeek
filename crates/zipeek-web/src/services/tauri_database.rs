use super::tauri_ipc_error::message;
use js_sys::{Function, Object, Promise, Reflect};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone, Deserialize)]
pub struct NativeDictionaryInfo {
    pub engine: String,
    pub native_engine: String,
    pub storage_compatibility: String,
    pub row_count: u32,
}

pub fn available() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(tauri) = Reflect::get(window.as_ref(), &JsValue::from_str("__TAURI__")) else {
        return false;
    };
    let Ok(core) = Reflect::get(&tauri, &JsValue::from_str("core")) else {
        return false;
    };
    Reflect::get(&core, &JsValue::from_str("invoke"))
        .ok()
        .is_some_and(|value| value.is_function())
}

async fn invoke<T: DeserializeOwned>(command: &str, args: &Object) -> Result<T, String> {
    let window = web_sys::window().ok_or("Window not available")?;
    let tauri = Reflect::get(window.as_ref(), &JsValue::from_str("__TAURI__")).map_err(message)?;
    let core = Reflect::get(&tauri, &JsValue::from_str("core")).map_err(message)?;
    let function = Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(message)?
        .dyn_into::<Function>()
        .map_err(|_| "Tauri IPC unavailable".to_owned())?;
    let promise = function
        .call2(&core, &JsValue::from_str(command), args)
        .map_err(message)?
        .dyn_into::<Promise>()
        .map_err(|_| "Expected IPC Promise".to_owned())?;
    let value = JsFuture::from(promise).await.map_err(message)?;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn open() -> Result<NativeDictionaryInfo, String> {
    invoke("database_open", &Object::new()).await
}

pub async fn query<T: DeserializeOwned>(sql: &str, parameters: Vec<Value>) -> Result<T, String> {
    let args = Object::new();
    Reflect::set(&args, &JsValue::from_str("sql"), &JsValue::from_str(sql)).map_err(message)?;
    let values = serde_wasm_bindgen::to_value(&parameters).map_err(|error| error.to_string())?;
    Reflect::set(&args, &JsValue::from_str("parameters"), &values).map_err(message)?;
    invoke("database_query", &args).await
}

#[derive(Deserialize)]
pub struct HistoryRow {
    pub session_id: String,
    pub evidence: String,
}

pub async fn commit_history(session_id: &str, evidence: &str) -> Result<(), String> {
    let args = Object::new();
    Reflect::set(
        &args,
        &JsValue::from_str("sessionId"),
        &JsValue::from_str(session_id),
    )
    .map_err(message)?;
    Reflect::set(
        &args,
        &JsValue::from_str("evidence"),
        &JsValue::from_str(evidence),
    )
    .map_err(message)?;
    invoke("history_commit", &args).await
}

pub async fn history_rows() -> Result<Vec<HistoryRow>, String> {
    invoke("history_rows", &Object::new()).await
}

pub async fn erase_history() -> Result<(), String> {
    invoke("history_erase", &Object::new()).await
}
