use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Navigator, Storage, Window};

use super::policy::{owns_private_entry, owns_storage_key};
use crate::services::tauri_ipc_error::message;

#[wasm_bindgen(inline_js = r#"
export async function acquireAppStorageSession() {
    if (!navigator.locks) return;
    await new Promise((ready, reject) => {
        navigator.locks.request('zipeek-app-storage-session', {mode: 'shared'}, () => {
            ready();
            return new Promise(release => window.addEventListener('pagehide', release, {once: true}));
        }).catch(reject);
    });
}
"#)]
unsafe extern "C" {
    #[wasm_bindgen(catch, js_name = acquireAppStorageSession)]
    pub async fn acquire_storage_session() -> Result<JsValue, JsValue>;
}

pub fn property(target: &JsValue, key: &str) -> Result<JsValue, String> {
    Reflect::get(target, &JsValue::from_str(key)).map_err(message)
}

pub fn call(target: &JsValue, name: &str, values: &[JsValue]) -> Result<JsValue, String> {
    let function = property(target, name)?
        .dyn_into::<Function>()
        .map_err(|_| format!("Browser storage does not support {name}"))?;
    let args = Array::new();
    for value in values {
        args.push(value);
    }
    function.apply(target, &args).map_err(message)
}

pub async fn wait(value: JsValue) -> Result<JsValue, String> {
    JsFuture::from(Promise::resolve(&value))
        .await
        .map_err(message)
}

pub fn options(key: &str, value: &JsValue) -> Result<JsValue, String> {
    let options = Object::new();
    Reflect::set(&options, &JsValue::from_str(key), value).map_err(message)?;
    Ok(options.into())
}

pub async fn private_directory(navigator: &Navigator) -> Result<JsValue, String> {
    let storage = property(navigator.as_ref(), "storage")?;
    wait(call(&storage, "getDirectory", &[])?).await
}

async fn owned_private_entries(directory: &JsValue) -> Result<Vec<String>, String> {
    let iterator = call(directory, "keys", &[])?;
    let mut names = Vec::new();
    loop {
        let next = wait(call(&iterator, "next", &[])?).await?;
        if property(&next, "done")?.as_bool() == Some(true) {
            return Ok(names);
        }
        let name = property(&next, "value")?
            .as_string()
            .ok_or("Invalid private storage entry")?;
        if owns_private_entry(&name) {
            names.push(name);
        }
    }
}

pub async fn clear_private_data(directory: &JsValue) -> Result<(), String> {
    for name in owned_private_entries(directory).await? {
        wait(call(
            directory,
            "removeEntry",
            &[
                JsValue::from_str(&name),
                options("recursive", &JsValue::TRUE)?,
            ],
        )?)
        .await?;
    }
    if !owned_private_entries(directory).await?.is_empty() {
        return Err("Zipeek private storage was not completely cleared".into());
    }
    Ok(())
}

fn owned_keys(storage: &Storage) -> Result<Vec<String>, String> {
    let mut keys = Vec::new();
    for index in 0..storage.length().map_err(message)? {
        if let Some(key) = storage.key(index).map_err(message)?
            && owns_storage_key(&key)
        {
            keys.push(key);
        }
    }
    Ok(keys)
}

pub fn clear_storage(storage: &Storage) -> Result<(), String> {
    for key in owned_keys(storage)? {
        storage.remove_item(&key).map_err(message)?;
    }
    if !owned_keys(storage)?.is_empty() {
        return Err("Zipeek settings were not completely cleared".into());
    }
    Ok(())
}

pub fn session_storage(window: &Window) -> Result<Storage, String> {
    window
        .session_storage()
        .map_err(message)?
        .ok_or_else(|| "Session storage is unavailable".into())
}
