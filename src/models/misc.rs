use std::{error::Error, fmt::{self, Debug, Display, Formatter}};

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::models::AppError;

#[derive(Clone, Default, Debug, PartialEq)]
pub enum AppState {
    #[default]
    Loading,
    Error(AppError),
    Loaded(Social)
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
