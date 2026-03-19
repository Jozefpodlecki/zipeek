use std::{error::Error, fmt::{self, Debug, Display, Formatter}};

use serde::Deserialize;
use wasm_bindgen::JsValue;
use serde_wasm_bindgen::Error as SerdeError;

#[derive(Debug, Clone, PartialEq)]
pub struct FetchError {
    pub err: JsValue,
}

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl Error for FetchError {}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}

impl From<SerdeError> for FetchError {
    fn from(error: SerdeError) -> Self {
        Self { 
            err: JsValue::from_str(&error.to_string())
        }
    }
}