use js_sys::Reflect;
use serde::Deserialize;
use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Debug, Error)]
pub(crate) enum TauriError {
    #[error("Tauri bridge unavailable")]
    BridgeUnavailable,
    #[error("Tauri API unavailable: {0}")]
    ApiUnavailable(&'static str),
    #[error("Tauri JavaScript interop failed: {0}")]
    JavaScript(String),
    #[error("Tauri request encoding failed: {0}")]
    Encode(String),
    #[error("Tauri command {command} failed: {details}")]
    Invocation {
        command: &'static str,
        details: String,
    },
    #[error("Tauri IPC returned an unexpected response for {0}")]
    InvalidResponse(&'static str),
    #[error("Tauri response decoding failed: {0}")]
    Decode(String),
}

#[derive(Debug, Deserialize)]
struct IpcError {
    kind: String,
    details: Option<String>,
}

pub(crate) fn message(value: JsValue) -> String {
    if let Ok(error) = serde_wasm_bindgen::from_value::<IpcError>(value.clone()) {
        return error
            .details
            .filter(|details| !details.is_empty())
            .unwrap_or_else(|| error.kind.replace('_', " "));
    }
    value
        .as_string()
        .or_else(|| {
            Reflect::get(&value, &JsValue::from_str("message"))
                .ok()
                .and_then(|value| value.as_string())
        })
        .unwrap_or_else(|| format!("{value:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_structured_service_failure() {
        let error: IpcError = serde_json::from_str(
            r#"{"kind":"executable_missing","details":"Missing service binary"}"#,
        )
        .expect("structured error");
        assert_eq!(error.kind, "executable_missing");
        assert_eq!(error.details.as_deref(), Some("Missing service binary"));
    }

    #[test]
    fn decodes_unit_database_failure() {
        let error: IpcError =
            serde_json::from_str(r#"{"kind":"worker_unavailable"}"#).expect("unit error");
        assert_eq!(error.kind, "worker_unavailable");
        assert!(error.details.is_none());
    }
}
