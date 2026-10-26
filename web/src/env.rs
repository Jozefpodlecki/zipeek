use std::{rc::Rc, str::FromStr};

use log::Level;
use thiserror::Error;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, Navigator, Storage, Window, window};
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct AppEnvironment {
    pub root: HtmlElement,
    pub window: Window,
    pub document: Document,
    pub local_storage: Option<Storage>,
    pub app_name: Rc<str>,
    pub version: Rc<str>,
    pub navigator: Navigator,
}

#[derive(Debug, Error)]
pub enum AppEnvironmentError {
    #[error("Window object not found")]
    NoWindow,
    #[error("Document object not found")]
    NoDocument,
    #[error("Application root element not found")]
    NoRoot,
}

impl AppEnvironment {
    pub fn new() -> Result<Self, AppEnvironmentError> {
        let window = window().ok_or(AppEnvironmentError::NoWindow)?;
        let document = window.document().ok_or(AppEnvironmentError::NoDocument)?;
        let root = document
            .get_element_by_id("root")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        let navigator = window.navigator();

        let local_storage = window.local_storage().ok().flatten();

        Ok(Self {
            root,
            window,
            document,
            local_storage,
            app_name: "Zipeek".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            navigator,
        })
    }

    pub fn try_get_log_level_from_local_storage(&self, key: &str) -> Option<Level> {
        let value = self.local_storage.as_ref()?.get_item(key).ok().flatten()?;
        Level::from_str(&value).ok()
    }
}
