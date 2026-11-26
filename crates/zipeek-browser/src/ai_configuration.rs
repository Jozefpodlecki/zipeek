use std::cell::RefCell;

use js_sys::{JSON, Reflect};
use wasm_bindgen::JsValue;
use web_sys::{Url, window};

const SETTINGS_KEY: &str = "zipeek.preferences.v1";
const DEFAULT_URL: &str = "http://127.0.0.1:1234/v1/";

thread_local! {
    static API_KEY: RefCell<String> = const { RefCell::new(String::new()) };
}

pub struct AiConfiguration {
    pub url: String,
    pub model: String,
    pub key_present: bool,
}

fn settings() -> Result<(web_sys::Storage, JsValue), String> {
    let storage = window()
        .ok_or_else(|| "Browser window unavailable".to_owned())?
        .local_storage()
        .map_err(|_| "Browser storage unavailable".to_owned())?
        .ok_or_else(|| "Browser storage unavailable".to_owned())?;
    let content = storage
        .get_item(SETTINGS_KEY)
        .map_err(|_| "Browser storage unavailable".to_owned())?
        .unwrap_or_else(|| "{}".to_owned());
    let document = JSON::parse(&content).map_err(|_| "Invalid settings document".to_owned())?;
    if !document.is_object() || js_sys::Array::is_array(&document) || document.is_null() {
        return Err("Invalid settings document".to_owned());
    }
    Ok((storage, document))
}

fn setting(document: &JsValue, key: &str) -> String {
    Reflect::get(document, &JsValue::from_str(key))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default()
}

pub fn configuration() -> AiConfiguration {
    let (url, model) = settings()
        .map(|(_, document)| {
            let url = setting(&document, "ai_url");
            (
                if url.is_empty() {
                    DEFAULT_URL.to_owned()
                } else {
                    url
                },
                setting(&document, "ai_model"),
            )
        })
        .unwrap_or_else(|_| (DEFAULT_URL.to_owned(), String::new()));
    AiConfiguration {
        url,
        model,
        key_present: API_KEY.with(|key| !key.borrow().is_empty()),
    }
}

pub fn save_configuration(url: &str, model: &str, key: &str) -> Result<(), String> {
    let parsed = Url::new(url).map_err(|_| "Invalid AI endpoint URL".to_owned())?;
    if !matches!(parsed.protocol().as_str(), "http:" | "https:") {
        return Err("Only HTTP(S) AI endpoints are supported".to_owned());
    }
    if !parsed.username().is_empty() || !parsed.password().is_empty() {
        return Err("Do not place credentials in the endpoint URL".to_owned());
    }
    let model = model.trim();
    if model.is_empty() {
        return Err("A model identifier is required".to_owned());
    }
    let (storage, document) = settings()
        .map_err(|_| "Browser storage unavailable; settings could not be saved".to_owned())?;
    Reflect::set(
        &document,
        &JsValue::from_str("ai_url"),
        &JsValue::from_str(&parsed.href()),
    )
    .map_err(|_| "Browser storage unavailable; settings could not be saved".to_owned())?;
    Reflect::set(
        &document,
        &JsValue::from_str("ai_model"),
        &JsValue::from_str(model),
    )
    .map_err(|_| "Browser storage unavailable; settings could not be saved".to_owned())?;
    let serialized = JSON::stringify(&document)
        .map_err(|_| "Browser storage unavailable; settings could not be saved".to_owned())?
        .as_string()
        .ok_or_else(|| "Browser storage unavailable; settings could not be saved".to_owned())?;
    storage
        .set_item(SETTINGS_KEY, &serialized)
        .map_err(|_| "Browser storage unavailable; settings could not be saved".to_owned())?;
    if !key.trim().is_empty() {
        API_KEY.with(|value| *value.borrow_mut() = key.trim().to_owned());
    }
    Ok(())
}

pub fn api_key_for_request() -> String {
    API_KEY.with(|key| key.borrow().clone())
}

pub fn clear_key() {
    API_KEY.with(|key| key.borrow_mut().clear());
}
