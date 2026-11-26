use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Debug, Error)]
pub enum DuckDbError {
    #[error("DuckDB module loading failed: {0}")]
    Module(String),
    #[error("DuckDB bundle selection failed: {0}")]
    BundleSelection(String),
    #[error("DuckDB initialization failed: {0}")]
    Initialization(String),
    #[error("DuckDB connection failed: {0}")]
    Connection(String),
    #[error("DuckDB query failed: {0}")]
    Query(String),
    #[error("DuckDB statement cleanup failed: {0}")]
    StatementCleanup(String),
    #[error("DuckDB query and cleanup failed: query: {query}; cleanup: {cleanup}")]
    QueryAndCleanup { query: String, cleanup: String },
    #[error("DuckDB operation timed out: {0}")]
    Timeout(String),
    #[error("DuckDB shutdown failed: {0}")]
    Shutdown(String),
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error(transparent)]
    DuckDb(#[from] DuckDbError),
    #[error(transparent)]
    Evidence(#[from] crate::EvidenceError),
    #[error("{operation}: {message}")]
    Operation {
        operation: &'static str,
        message: String,
    },
    #[error("Cannot decode database response: {0}")]
    Decode(String),
    #[error(transparent)]
    Manifest(#[from] super::manifest::DictionaryManifestError),
    #[error(transparent)]
    Bundle(#[from] super::bundle::BaseBundleError),
}

pub(super) fn js_error(operation: &'static str, value: JsValue) -> DbError {
    let message = js_sys::Reflect::get(&value, &"message".into())
        .ok()
        .and_then(|message| message.as_string())
        .or_else(|| value.as_string())
        .unwrap_or_else(|| format!("{value:?}"));
    DbError::Operation { operation, message }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duckdb_error_preserves_both_failure_phases() {
        let error = DbError::from(DuckDbError::QueryAndCleanup {
            query: "syntax".into(),
            cleanup: "worker disconnected".into(),
        });
        assert!(error.to_string().contains("syntax"));
        assert!(error.to_string().contains("worker disconnected"));
    }
}
