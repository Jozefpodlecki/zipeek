use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiClientError(pub String);

impl std::fmt::Display for AiClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for AiClientError {}

fn status_explanation(status: reqwest::StatusCode) -> &'static str {
    match status.as_u16() {
        401 | 403 => "AI authentication or access denied",
        404 => "AI endpoint not found; check the /v1/ base URL",
        400 | 422 => "AI request rejected; check the model and provider API compatibility",
        429 => "AI rate limit exceeded",
        500..=599 => "AI server unavailable",
        _ => "AI request failed",
    }
}

pub(super) fn http_failure(status: reqwest::StatusCode, body: &str, key: &str) -> AiClientError {
    let mut message = format!("{} (HTTP {})", status_explanation(status), status.as_u16());
    let excerpt: String = body.chars().take(8192).collect();
    if let Ok(payload) = serde_json::from_str::<Value>(&excerpt) {
        let error = &payload["error"];
        let provider_message = error.as_str().or_else(|| error["message"].as_str());
        if let Some(provider_message) = provider_message {
            let safe = provider_message
                .replace(&['\r', '\n', '\t'][..], " ")
                .chars()
                .filter(|character| !character.is_control())
                .collect::<String>();
            let safe = if !key.is_empty() {
                safe.replace(key, "[redacted]")
            } else {
                safe
            };
            let safe = redact_bearer(&safe);
            let safe: String = safe.chars().take(240).collect();
            if !safe.is_empty() {
                message.push_str(": ");
                message.push_str(&safe);
            }
        }
        if let Some(code) = error["code"].as_str() {
            let code: String = code
                .chars()
                .filter(|character| {
                    character.is_ascii_alphanumeric() || matches!(*character, '_' | '-')
                })
                .take(48)
                .collect();
            if !code.is_empty() && (key.is_empty() || !code.contains(key)) {
                message.push_str(&format!(" ({code})"));
            }
        }
    }
    AiClientError(message)
}

fn redact_bearer(input: &str) -> String {
    let mut words = input.split_whitespace().peekable();
    let mut safe = Vec::new();
    while let Some(word) = words.next() {
        if word.eq_ignore_ascii_case("Bearer") {
            safe.push("Bearer".to_owned());
            if words.next().is_some() {
                safe.push("[redacted]".to_owned());
            }
        } else if word.starts_with("sk-") {
            safe.push("[redacted]".to_owned());
        } else {
            safe.push(word.to_owned());
        }
    }
    safe.join(" ")
}

pub(super) fn request_error(error: reqwest::Error) -> AiClientError {
    if error.is_timeout() {
        AiClientError("AI request exceeded the five-minute timeout. Check whether the model is still generating.".to_owned())
    } else {
        AiClientError("Cannot reach the AI endpoint. Check the URL, server, CORS and HTTPS/mixed-content restrictions.".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_errors_are_bounded_and_redacted() {
        let error = http_failure(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"error":{"message":"Bearer sk-secret-12345678 exceeded context length","code":"context_length_exceeded"}}"#,
            "sk-secret-12345678",
        );
        assert!(error.0.contains("HTTP 400"));
        assert!(error.0.contains("context_length_exceeded"));
        assert!(!error.0.contains("sk-secret-12345678"));
        assert!(
            !http_failure(reqwest::StatusCode::BAD_REQUEST, "<html>secret</html>", "")
                .0
                .contains("secret")
        );
    }
}
