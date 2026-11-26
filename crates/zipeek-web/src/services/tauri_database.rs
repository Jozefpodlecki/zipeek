use super::tauri_ipc_error::message;
use js_sys::{Function, Object, Promise, Reflect};
use serde::de::DeserializeOwned;
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use zipeek_db::contracts::{DatabaseInfo, HistoryRow};

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

pub async fn open() -> Result<DatabaseInfo, String> {
    invoke("database_open", &Object::new()).await
}

pub async fn query<T: DeserializeOwned>(sql: &str, parameters: Vec<Value>) -> Result<T, String> {
    let args = Object::new();
    Reflect::set(&args, &JsValue::from_str("sql"), &JsValue::from_str(sql)).map_err(message)?;
    let values = serde_wasm_bindgen::to_value(&parameters).map_err(|error| error.to_string())?;
    Reflect::set(&args, &JsValue::from_str("parameters"), &values).map_err(message)?;
    invoke("database_query", &args).await
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

pub async fn reset_personal_data() -> Result<(), String> {
    let args = Object::new();
    Reflect::set(&args, &JsValue::from_str("confirmed"), &JsValue::TRUE).map_err(message)?;
    invoke("personal_data_reset", &args).await
}
