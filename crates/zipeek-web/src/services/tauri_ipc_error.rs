use js_sys::Reflect;
use serde::Deserialize;
use wasm_bindgen::JsValue;

#[derive(Debug, Deserialize)]
struct IpcError {
    kind: String,
    details: Option<String>,
}

pub fn message(value: JsValue) -> String {
    if let Ok(error) = serde_wasm_bindgen::from_value::<IpcError>(value.clone()) {
        let kind = error.kind.replace('_', " ");
        return match error.details {
            Some(details) if !details.is_empty() => details,
            _ => kind,
        };
    }
    value
        .as_string()
        .or_else(|| {
            Reflect::get(&value, &JsValue::from_str("message"))
                .ok()
                .and_then(|message| message.as_string())
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
