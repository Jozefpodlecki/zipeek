use std::{net::SocketAddr, time::Duration};

use serde_json::{Value, json};
use tauri::command;

use super::{error::CommandError, native_http};

const MAX_AI_BODY: usize = 4 * 1024 * 1024;

fn validated_endpoint(endpoint: &str) -> Result<SocketAddr, String> {
    let authority = endpoint
        .trim()
        .strip_prefix("http://")
        .ok_or("Native AI supports HTTP private-network endpoints only")?;
    let (host, path) = authority
        .split_once('/')
        .ok_or("AI endpoint must end with /v1")?;
    if !matches!(path, "v1" | "v1/") || host.contains('@') || host.contains('?') {
        return Err("Invalid native AI endpoint".into());
    }
    let address: SocketAddr = host
        .parse()
        .map_err(|_| "Native AI requires an IPv4 address and port")?;
    match address {
        SocketAddr::V4(value)
            if (value.ip().is_loopback() && value.port() == 1430)
                || ((value.ip().is_private() || value.ip().is_loopback())
                    && value.port() == 1234) =>
        {
            Ok(address)
        }
        _ => Err("Native AI permits local service :1430 or private-network LM Studio :1234".into()),
    }
}

#[command]
pub async fn ai_models(endpoint: String, key: String) -> Result<Vec<String>, CommandError> {
    let address = validated_endpoint(&endpoint).map_err(CommandError::Ai)?;
    if !key.is_empty() && !address.ip().is_loopback() {
        return Err(CommandError::Ai(
            "API keys cannot be sent over an unencrypted LAN connection".into(),
        ));
    }
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let authorization = if key.is_empty() {
            None
        } else {
            Some(format!("Bearer {key}"))
        };
        let headers = authorization
            .as_ref()
            .map_or_else(Vec::new, |value| vec![("Authorization", value.as_str())]);
        let (status, bytes) = native_http::request(
            address,
            "/v1/models",
            "GET",
            "application/json",
            &[],
            &headers,
            MAX_AI_BODY,
            Duration::from_secs(20),
        )?;
        if status != 200 {
            return Err(format!("AI model discovery returned HTTP {status}"));
        }
        Ok::<_, String>(bytes)
    })
    .await
    .map_err(|_| CommandError::Ai("AI discovery task stopped".into()))?
    .map_err(CommandError::Ai)?;
    let response: Value = serde_json::from_slice(&bytes)
        .map_err(|_| CommandError::Ai("Invalid AI model list".into()))?;
    let models = response
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| CommandError::Ai("AI model list is missing data".into()))?
        .iter()
        .filter_map(|model| model.get("id").and_then(Value::as_str))
        .filter(|id| !id.is_empty() && id.len() <= 256)
        .map(str::to_owned)
        .take(100)
        .collect();
    Ok(models)
}

#[command]
pub async fn ai_complete(
    endpoint: String,
    model: String,
    messages: Vec<Value>,
    key: String,
    schema: Option<Value>,
    max_tokens: Option<u32>,
) -> Result<String, CommandError> {
    let address = validated_endpoint(&endpoint).map_err(CommandError::Ai)?;
    if model.trim().is_empty()
        || model.len() > 256
        || messages.is_empty()
        || messages.len() > 40
        || key.len() > 512
        || key
            .chars()
            .any(|character| character == '\r' || character == '\n')
        || max_tokens.is_some_and(|tokens| !(64..=16384).contains(&tokens))
    {
        return Err(CommandError::Ai("Invalid AI request".into()));
    }
    if !key.is_empty() && !address.ip().is_loopback() {
        return Err(CommandError::Ai(
            "API keys cannot be sent over an unencrypted LAN connection".into(),
        ));
    }
    let mut payload =
        json!({"model": model, "messages": messages, "temperature": 0.2, "stream": false});
    if let Some(schema) = schema {
        payload["response_format"] = schema;
    }
    if let Some(tokens) = max_tokens {
        payload["max_completion_tokens"] = json!(tokens);
    }
    let bytes =
        serde_json::to_vec(&payload).map_err(|_| CommandError::Ai("Invalid AI request".into()))?;
    if bytes.len() > 128 * 1024 {
        return Err(CommandError::Ai("AI request too large".into()));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let authorization = if key.is_empty() {
            None
        } else {
            Some(format!("Bearer {key}"))
        };
        let headers = authorization
            .as_ref()
            .map_or_else(Vec::new, |value| vec![("Authorization", value.as_str())]);
        let (status, response) = native_http::request(
            address,
            "/v1/chat/completions",
            "POST",
            "application/json",
            &bytes,
            &headers,
            MAX_AI_BODY,
            Duration::from_secs(180),
        )?;
        if status != 200 {
            return Err(format!("AI request rejected (HTTP {status})"));
        }
        let value: Value =
            serde_json::from_slice(&response).map_err(|_| "Invalid AI completion response")?;
        if value
            .pointer("/choices/0/finish_reason")
            .and_then(Value::as_str)
            == Some("length")
        {
            return Err("AI output limit reached".into());
        }
        value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(str::to_owned)
            .ok_or_else(|| "AI returned no text".into())
    })
    .await
    .map_err(|_| CommandError::Ai("AI inference task stopped".into()))?
    .map_err(CommandError::Ai)
}

#[cfg(test)]
mod tests {
    use super::validated_endpoint;

    #[test]
    fn blocks_untrusted_destination_and_credentials() {
        assert!(validated_endpoint("http://127.0.0.1:1430/v1").is_ok());
        assert!(validated_endpoint("http://192.168.1.148:1234/v1/").is_ok());
        for url in [
            "https://example.com/v1",
            "http://8.8.8.8:1234/v1",
            "http://192.168.1.148:8888/v1",
            "http://user@192.168.1.148:1234/v1",
            "http://192.168.1.148:1234/v1/../../secret",
        ] {
            assert!(validated_endpoint(url).is_err(), "{url}");
        }
    }
}
