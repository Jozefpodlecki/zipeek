use js_sys::Reflect;
use wasm_bindgen::JsValue;

pub(super) fn describe_capture_error(error: &JsValue) -> String {
    let name = Reflect::get(error, &JsValue::from_str("name"))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default();
    match name.as_str() {
        "NotAllowedError" => {
            "Microphone access was denied. Allow microphone access in browser settings to record."
                .to_owned()
        }
        "NotFoundError" => {
            "No microphone was found. Connect a microphone and try again.".to_owned()
        }
        _ => Reflect::get(error, &JsValue::from_str("message"))
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_else(|| "Microphone recording could not start.".to_owned()),
    }
}
