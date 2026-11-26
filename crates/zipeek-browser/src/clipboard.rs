use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

pub async fn copy_hanzi(text: &str) -> Result<(), JsValue> {
    if text.trim().is_empty() {
        return Err(JsValue::from_str("No Chinese text to copy"));
    }
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("Browser window unavailable"))?;
    let clipboard = window.navigator().clipboard();
    let value = JsValue::from(clipboard.clone());
    if value.is_undefined() || value.is_null() {
        return Err(JsValue::from_str(
            "Clipboard is unavailable. Use HTTPS or localhost.",
        ));
    }
    JsFuture::from(clipboard.write_text(text)).await?;
    Ok(())
}
