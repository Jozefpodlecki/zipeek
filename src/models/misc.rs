use std::{error::Error, fmt::{self, Debug, Display, Formatter}};

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::models::FetchError;

#[derive(Clone, Debug, PartialEq)]
pub enum AppState {
    Loading,
    Error(FetchError),
    Loaded(Social)
}

impl Default for AppState {
    fn default() -> Self {
        Self::Loading
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct Social {
    pub linkedin: String,
    pub github: String,
    pub portfolio: String
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct LexemeBreakdown {
    pub grapheme: String,
    pub pinyin: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct Lexeme {
    pub value: String,
    pub pinyin: String,
    pub breakdown: Vec<LexemeBreakdown>,
    pub part_of_speech: String,
    pub meanings: Vec<String>
}
