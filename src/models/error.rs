use std::{error::Error, fmt::{self, Debug, Display, Formatter}};

use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use serde_wasm_bindgen::Error as SerdeError;

#[derive(Debug, Clone, PartialEq)]
pub enum AppErrorKind {
    Network,
    Serde,
    Js,
    Unexpected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppError {
    pub kind: AppErrorKind,
    pub message: String,
    pub source: Option<JsValue>,
}

impl AppError {
    pub fn new(kind: AppErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(kind: AppErrorKind, message: impl Into<String>, source: JsValue) -> Self {
        Self {
            kind,
            message: message.into(),
            source: Some(source),
        }
    }

    pub fn network(message: impl Into<String>, source: JsValue) -> Self {
        Self {
            kind: AppErrorKind::Network,
            message: message.into(),
            source: Some(source),
        }
    }

    pub fn js(message: impl Into<String>, source: JsValue) -> Self {
        Self {
            kind: AppErrorKind::Js,
            message: message.into(),
            source: Some(source),
        }
    }

    pub fn clipboard_not_available(source: JsValue) -> Self {
        Self::js("Clipboard not available", source)
    }

    pub fn failed_to_build_request(source: JsValue) -> Self {
        Self::network("Failed to build request", source)
    }

    pub fn network_request_failed(source: JsValue) -> Self {
        Self::network("Network request failed", source)
    }

    pub fn invalid_response(source: JsValue) -> Self {
        Self::network("Invalid response object", source)
    }

    pub fn failed_to_read_body(source: JsValue) -> Self {
        Self::network("Failed to read response body", source)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AppError {}

impl From<JsValue> for AppError {
    fn from(value: JsValue) -> Self {
        if let Some(s) = value.as_string() {
            return Self::with_source(AppErrorKind::Js, s, value);
        }

        if let Some(err) = value.dyn_ref::<js_sys::Error>() {
            return Self::with_source(
                AppErrorKind::Js,
                err.message(),
                value,
            );
        }

        if let Some(obj) = value.dyn_ref::<js_sys::Object>() {
            if let Ok(val) = js_sys::Reflect::get(obj, &JsValue::from_str("message")) {
                if let Some(msg) = val.as_string() {
                    return Self::with_source(AppErrorKind::Js, msg, value);
                }
            }
        }

        Self::with_source(
            AppErrorKind::Unexpected,
            "Unexpected JavaScript error",
            value,
        )
    }
}

impl From<SerdeError> for AppError {
    fn from(error: SerdeError) -> Self {
        Self {
            kind: AppErrorKind::Serde,
            message: "Failed to parse response data".into(),
            source: Some(JsValue::from_str(&error.to_string())),
        }
    }
}