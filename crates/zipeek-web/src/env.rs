use std::{rc::Rc, str::FromStr};

use js_sys::{Function, Reflect};
use log::Level;
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement, Navigator, Storage, Window, window};
use zipeek_browser::document::{BrowserDocument, BrowserDocumentError};

use crate::{
    services::speech::{self, SpeechMode, SpeechService},
    theme::ThemeManager,
};
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct AppEnvironment {
    pub root: HtmlElement,
    pub window: Window,
    pub document: Document,
    pub browser_document: BrowserDocument,
    pub theme_manager: ThemeManager,
    pub local_storage: Storage,
    pub app_name: Rc<str>,
    pub version: Rc<str>,
    pub navigator: Navigator,
    pub(crate) speech: SpeechService,
    pub is_tauri: bool,
}

#[derive(Debug, Error)]
pub enum AppEnvironmentError {
    #[error("Database backend setup failed: {0}")]
    DatabaseBackend(String),

    #[error("Window object not found")]
    NoWindow,

    #[error("Document object not found")]
    NoDocument,

    #[error("Application root element not found")]
    NoRoot,

    #[error("Document root element unavailable: {0}")]
    NoDocumentRoot(#[from] BrowserDocumentError),

    #[error("Browser localStorage is unavailable: {0}")]
    NoStorage(String),

    #[error("JavaScript inspection failed for {field}: {details}")]
    Inspection {
        field: &'static str,
        details: String,
    },

    #[error("Tauri detection function is missing")]
    MissingCheck,

    #[error("Tauri detection failed: {0}")]
    CheckFailed(String),

    #[error("Tauri detection returned a non-boolean result")]
    InvalidCheckResult,

    #[error("Hostname access failed: {0}")]
    HostnameAccess(String),
}

impl AppEnvironment {
    pub fn new() -> Result<Self, AppEnvironmentError> {
        let window = window().ok_or(AppEnvironmentError::NoWindow)?;

        let document = window.document().ok_or(AppEnvironmentError::NoDocument)?;

        let root = document
            .get_element_by_id("root")
            .ok_or(AppEnvironmentError::NoRoot)?
            .unchecked_into::<HtmlElement>();

        let navigator = window.navigator();

        let browser_document = BrowserDocument::new(document.clone())?;

        let local_storage = window
            .local_storage()
            .map_err(|error| AppEnvironmentError::NoStorage(format!("{error:?}")))?
            .ok_or_else(|| {
                AppEnvironmentError::NoStorage("Storage API not available".to_owned())
            })?;

        let theme_manager = ThemeManager::new(browser_document.clone(), local_storage.clone());

        let speech_settings = window
            .session_storage()
            .ok()
            .flatten()
            .and_then(|storage| storage.get_item(speech::E2E_SETTINGS_KEY).ok().flatten());

        let mode = speech::speech_mode(
            speech_settings.as_deref(),
            option_env!("ZIPEEK_E2E_SPEECH") == Some("1"),
        );

        if mode == SpeechMode::Silent {
            let _ = root.set_attribute("data-e2e-speech", "silent");
        }

        let speech = SpeechService::new(window.clone(), mode);

        let is_tauri = Self::detect_tauri(&window)?;
        let desktop_assets = document
            .query_selector("meta[name=zipeek-platform][content=desktop]")
            .ok()
            .flatten()
            .is_some();
        if desktop_assets && !is_tauri {
            return Err(AppEnvironmentError::DatabaseBackend(
                "Desktop assets require native Tauri IPC".into(),
            ));
        }
        let backend = if is_tauri {
            crate::services::database::backend::DatabaseBackend::Native
        } else {
            crate::services::database::backend::DatabaseBackend::Browser
        };
        crate::services::database::backend::initialize(backend)
            .map_err(AppEnvironmentError::DatabaseBackend)?;

        Ok(Self {
            root,
            window,
            document,
            browser_document,
            theme_manager,
            local_storage,
            app_name: Rc::from("Zipeek"),
            version: Rc::from(env!("CARGO_PKG_VERSION")),
            navigator,
            speech,
            is_tauri,
        })
    }

    pub(crate) fn detect_tauri(window: &Window) -> Result<bool, AppEnvironmentError> {
        let tauri =
            Reflect::get(window.as_ref(), &JsValue::from_str("__TAURI__")).map_err(|error| {
                AppEnvironmentError::Inspection {
                    field: "__TAURI__",
                    details: format!("{error:?}"),
                }
            })?;

        if !tauri.is_object() {
            return Ok(false);
        }

        let core = Reflect::get(&tauri, &JsValue::from_str("core")).map_err(|error| {
            AppEnvironmentError::Inspection {
                field: "core",
                details: format!("{error:?}"),
            }
        })?;

        if !core.is_object() {
            return Ok(false);
        }

        let check = Reflect::get(&core, &JsValue::from_str("isTauri")).map_err(|error| {
            AppEnvironmentError::Inspection {
                field: "isTauri",
                details: format!("{error:?}"),
            }
        })?;

        let Ok(check) = check.dyn_into::<Function>() else {
            return Ok(false);
        };

        let result = check
            .call0(&core)
            .map_err(|error| AppEnvironmentError::CheckFailed(format!("{error:?}")))?;

        result
            .as_bool()
            .ok_or(AppEnvironmentError::InvalidCheckResult)
    }

    pub fn try_get_log_level_from_local_storage(&self, key: &str) -> Option<Level> {
        let value = self.local_storage.get_item(key).ok().flatten()?;

        Level::from_str(&value).ok()
    }
}
