pub(crate) mod browser;
mod policy;
mod view;

use js_sys::Promise;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::future_to_promise;
use web_sys::Window;
use yew::Renderer;

use crate::{
    AppEnvironment,
    services::{tauri_database, tauri_ipc_error::message},
};
use policy::{CONFIRMATION_KEY, RESET_HASH};

pub async fn acquire_app_storage_session() -> Result<(), String> {
    unsafe { browser::acquire_storage_session().await }
        .map(|_| ())
        .map_err(message)
}

pub fn request(window: &Window) -> Result<(), String> {
    browser::session_storage(window)?
        .set_item(CONFIRMATION_KEY, "confirmed")
        .map_err(message)?;
    reload_at(window, RESET_HASH)
}

pub fn reload_at(window: &Window, hash: &str) -> Result<(), String> {
    let location = window.location();
    let mut url = location.href().map_err(message)?;
    url.truncate(url.find('#').unwrap_or(url.len()));
    url.push_str(hash);
    location.replace(&url).map_err(message)?;
    location.reload().map_err(message)
}

pub fn mount_if_requested() -> Result<bool, String> {
    let window = web_sys::window().ok_or("Window unavailable")?;
    if window.location().hash().map_err(message)? != RESET_HASH {
        return Ok(false);
    }
    let document = window.document().ok_or("Document unavailable")?;
    let root = document
        .get_element_by_id("root")
        .ok_or("Application root unavailable")?;
    let native = AppEnvironment::detect_tauri(&window).map_err(|error| error.to_string())?;
    if !native
        && document
            .query_selector("meta[name=zipeek-platform][content=desktop]")
            .map_err(message)?
            .is_some()
    {
        return Err(
            "Desktop reset requires the native Tauri bridge. Restart the desktop app and retry."
                .into(),
        );
    }
    let session = browser::session_storage(&window)?;
    let confirmed = session
        .get_item(CONFIRMATION_KEY)
        .map_err(message)?
        .as_deref()
        == Some("confirmed");
    session.remove_item(CONFIRMATION_KEY).map_err(message)?;
    Renderer::<view::ResetRunner>::with_root_and_props(
        root,
        view::Props {
            window,
            native,
            confirmed,
        },
    )
    .render();
    Ok(true)
}

async fn reset_data(window: &Window, native: bool) -> Result<(), String> {
    let local = window
        .local_storage()
        .map_err(message)?
        .ok_or("Local storage unavailable")?;
    let session = browser::session_storage(window)?;
    let directory = browser::private_directory(&window.navigator()).await?;
    browser::clear_private_data(&directory).await?;
    if native {
        tauri_database::reset_personal_data().await?;
    }
    browser::clear_storage(&local)?;
    browser::clear_storage(&session)?;
    Ok(())
}

pub async fn reset(window: &Window, native: bool) -> Result<(), String> {
    let locks = browser::property(window.navigator().as_ref(), "locks")?;
    if !locks.is_object() {
        return Err("This browser cannot safely reset app storage. Update it and retry.".into());
    }
    let options = browser::options("ifAvailable", &JsValue::TRUE)?;
    let window = window.clone();
    let callback = Closure::<dyn FnMut(JsValue) -> Promise>::new(move |lock: JsValue| {
        if lock.is_null() || lock.is_undefined() {
            return Promise::reject(&JsValue::from_str(
                "Close other Zipeek tabs or windows, then retry.",
            ));
        }
        let window = window.clone();
        future_to_promise(async move {
            reset_data(&window, native)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(|error| JsValue::from_str(&error))
        })
    });
    let result = browser::call(
        &locks,
        "request",
        &[
            JsValue::from_str("zipeek-app-storage-session"),
            options,
            callback.as_ref().clone(),
        ],
    )?;
    browser::wait(result).await?;
    Ok(())
}
