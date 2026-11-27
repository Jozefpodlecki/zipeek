use js_sys::{Array, Function, Object, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};

use crate::services::tauri::{Command, TauriService, commands, message};

pub(super) fn desktop_mode() -> Result<bool, String> {
    let window = web_sys::window().ok_or("Window unavailable")?;
    let bridge = TauriService::new(&window);
    if let Ok(bridge) = bridge {
        if bridge.is_tauri().map_err(|error| error.to_string())? {
            return Ok(true);
        }
    }
    let desktop_assets = window
        .document()
        .and_then(|document| {
            document
                .query_selector("meta[name=zipeek-platform][content=desktop]")
                .ok()
                .flatten()
        })
        .is_some();
    if desktop_assets {
        return Err("Desktop image operations require the Tauri application. Open the desktop app rather than its frontend URL in a browser.".into());
    }
    Ok(false)
}

async fn invoke<C: Command>(property: Option<(&str, &str)>) -> Result<JsValue, String> {
    let args = match property {
        Some(("prompt", value)) => serde_json::json!({ "prompt": value }),
        None => serde_json::json!({}),
        _ => return Err("Unsupported native image argument".into()),
    };
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .invoke_raw(C::NAME, &args)
        .await
        .map_err(|error| error.to_string())
}

pub(super) async fn health() -> Result<String, String> {
    invoke::<commands::ImageHealth>(None)
        .await?
        .as_string()
        .ok_or_else(|| "Invalid image health response".to_owned())
}

pub(super) async fn generate(prompt: &str) -> Result<String, String> {
    let payload = invoke::<commands::ImageGenerate>(Some(("prompt", prompt))).await?;
    let mime = Reflect::get(&payload, &JsValue::from_str("mime"))
        .map_err(message)?
        .as_string()
        .ok_or("Invalid image content type")?;
    if !matches!(mime.as_str(), "image/png" | "image/svg+xml") {
        return Err("Unsupported image content type".into());
    }
    let content = Reflect::get(&payload, &JsValue::from_str("bytes")).map_err(message)?;
    if !Array::is_array(&content) && !content.is_instance_of::<Uint8Array>() {
        return Err("Invalid native image byte array".into());
    }
    let bytes = Uint8Array::new(&content);
    if bytes.length() == 0 || bytes.length() > 8 * 1024 * 1024 {
        return Err("Invalid native image size".into());
    }
    let parts = Array::new();
    parts.push(&bytes);
    let options = Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str("type"),
        &JsValue::from_str(&mime),
    )
    .map_err(message)?;
    let blob_type = Reflect::get(&js_sys::global(), &JsValue::from_str("Blob"))
        .map_err(message)?
        .dyn_into::<Function>()
        .map_err(|_| "Blob constructor unavailable".to_owned())?;
    let args = Array::new();
    args.push(&parts);
    args.push(&options);
    let blob = Reflect::construct(&blob_type, &args).map_err(message)?;
    let url_type = Reflect::get(&js_sys::global(), &JsValue::from_str("URL")).map_err(message)?;
    let creator = Reflect::get(&url_type, &JsValue::from_str("createObjectURL"))
        .map_err(message)?
        .dyn_into::<Function>()
        .map_err(|_| "Browser blob URL support unavailable".to_owned())?;
    creator
        .call1(&url_type, &blob)
        .map_err(message)?
        .as_string()
        .ok_or_else(|| "Could not create image URL".into())
}

pub(super) fn release_blob_url(url: &str) {
    if !url.starts_with("blob:") {
        return;
    }
    let Ok(url_type) = Reflect::get(&js_sys::global(), &JsValue::from_str("URL")) else {
        return;
    };
    let Ok(method) = Reflect::get(&url_type, &JsValue::from_str("revokeObjectURL")) else {
        return;
    };
    if let Ok(revoke) = method.dyn_into::<Function>() {
        let _ = revoke.call1(&url_type, &JsValue::from_str(url));
    }
}
