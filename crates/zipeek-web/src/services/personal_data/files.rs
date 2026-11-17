use std::{cell::RefCell, rc::Rc};

use futures::channel::oneshot;
use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Document, HtmlAnchorElement, HtmlInputElement, Url};

use super::{backup, error::PersonalDataError};

pub fn download(document: &Document, raw: &str) -> Result<(), PersonalDataError> {
    backup::parse(raw)?;
    let data = Array::new();
    data.push(&JsValue::from_str(raw));
    let blob = Blob::new_with_str_sequence(&data).map_err(|error| {
        PersonalDataError::File(format!("Cannot construct JSON file: {error:?}"))
    })?;
    let url = Url::create_object_url_with_blob(&blob).map_err(|error| {
        PersonalDataError::File(format!("Cannot create download URL: {error:?}"))
    })?;
    let link: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|error| {
            PersonalDataError::File(format!("Cannot create download link: {error:?}"))
        })?
        .dyn_into()
        .map_err(|_| PersonalDataError::File("Download element unavailable".into()))?;
    link.set_href(&url);
    link.set_download("zipeek-learning.json");
    link.click();
    let revoke = Closure::once(move || {
        if let Err(error) = Url::revoke_object_url(&url) {
            log::warn!("Could not release backup URL: {error:?}");
        }
    });
    if let Some(window) = document.default_view() {
        if window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                revoke.as_ref().unchecked_ref(),
                1000,
            )
            .is_ok()
        {
            revoke.forget();
        }
    }
    Ok(())
}

pub async fn pick(document: &Document) -> Result<String, PersonalDataError> {
    let input: HtmlInputElement = document
        .create_element("input")
        .map_err(|error| PersonalDataError::File(format!("Cannot create file picker: {error:?}")))?
        .dyn_into()
        .map_err(|_| PersonalDataError::File("File picker unavailable".into()))?;
    input.set_type("file");
    input.set_accept(".json,application/json");
    let (sender, receiver) = oneshot::channel();
    let pending = Rc::new(RefCell::new(Some(sender)));
    let chosen_input = input.clone();
    let change_sender = pending.clone();
    let on_change = Closure::wrap(Box::new(move |_: web_sys::Event| {
        let file = chosen_input.files().and_then(|files| files.get(0));
        if let Some(sender) = change_sender.borrow_mut().take() {
            let _ = sender.send(file);
        }
    }) as Box<dyn FnMut(_)>);
    let cancel_sender = pending;
    let on_cancel = Closure::wrap(Box::new(move |_: web_sys::Event| {
        if let Some(sender) = cancel_sender.borrow_mut().take() {
            let _ = sender.send(None);
        }
    }) as Box<dyn FnMut(_)>);
    input.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())?;
    input.add_event_listener_with_callback("cancel", on_cancel.as_ref().unchecked_ref())?;
    input.click();
    let file = receiver
        .await
        .map_err(|_| PersonalDataError::File("File picker cancelled".into()))?;
    input.remove_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())?;
    input.remove_event_listener_with_callback("cancel", on_cancel.as_ref().unchecked_ref())?;
    let Some(file) = file else {
        return Ok(String::new());
    };
    if file.size() > backup::MAX_IMPORT_BYTES as f64 {
        return Err(PersonalDataError::InvalidBackup(
            "Backup exceeds the 2 MB limit".into(),
        ));
    }
    let blob: &Blob = file.unchecked_ref();
    let contents = JsFuture::from(blob.text())
        .await?
        .as_string()
        .ok_or_else(|| PersonalDataError::File("Backup text unavailable".into()))?;
    backup::parse(&contents)?;
    Ok(contents)
}
