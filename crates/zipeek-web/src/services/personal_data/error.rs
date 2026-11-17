use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersonalDataError {
    #[error("Invalid backup: {0}")]
    InvalidBackup(String),
    #[error("Unsupported personal-data category: {0}")]
    UnknownCategory(String),
    #[error("Browser storage operation failed: {0}")]
    Storage(String),
    #[error("Storage mutation failed: {operation}; rollback {rollback}")]
    PartialRollback { operation: String, rollback: String },
    #[error("Private file operation failed: {0}")]
    Opfs(String),
    #[error("File selection or download failed: {0}")]
    File(String),
    #[error("Browser API operation failed: {0}")]
    Browser(String),
}

impl From<wasm_bindgen::JsValue> for PersonalDataError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::Browser(format!("{value:?}"))
    }
}
