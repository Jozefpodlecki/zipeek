use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/services/ai/bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = completeLocal)]
    async fn complete_local(url: &str, model: &str, messages: JsValue) -> Result<JsValue, JsValue>;
}

pub(crate) async fn complete(
    endpoint: &str,
    model: &str,
    messages: Vec<serde_json::Value>,
) -> Result<String, String> {
    let messages = serde_wasm_bindgen::to_value(&messages).map_err(|error| error.to_string())?;
    complete_local(endpoint, model, messages)
        .await
        .map_err(|error| {
            js_sys::Reflect::get(&error, &"message".into())
                .ok()
                .and_then(|value| value.as_string())
                .or_else(|| error.as_string())
                .unwrap_or_else(|| "Local AI request failed".to_owned())
        })?
        .as_string()
        .ok_or_else(|| "Local AI returned a non-text response".to_owned())
}
