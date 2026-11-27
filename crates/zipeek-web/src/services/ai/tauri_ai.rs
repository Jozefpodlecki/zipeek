use serde_json::{Value, json};

use crate::services::tauri::{TauriService, commands};

pub(super) fn enabled() -> bool {
    web_sys::window().is_some_and(|window| TauriService::available(&window))
}

pub(super) async fn complete(
    endpoint: &str,
    model: &str,
    messages: Vec<Value>,
    key: &str,
    schema: Option<Value>,
    max_tokens: Option<u32>,
) -> Result<String, String> {
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::AiComplete>(&json!({
            "endpoint": endpoint, "model": model, "messages": messages, "key": key,
            "schema": schema, "maxTokens": max_tokens,
        }))
        .await
        .map_err(|error| error.to_string())
}

pub(super) async fn models(endpoint: &str, key: &str) -> Result<Vec<String>, String> {
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::AiModels>(&json!({"endpoint": endpoint, "key": key}))
        .await
        .map_err(|error| error.to_string())
}
