use crate::structured::ResponseFormat;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(super) struct ChatCompletionRequest<'a, M: Serialize> {
    pub model: &'a str,
    pub messages: &'a [M],
    pub temperature: f32,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<&'a ResponseFormat>,
}

#[derive(Deserialize)]
pub(super) struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub(super) struct Choice {
    pub message: AssistantMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct AssistantMessage {
    pub content: Option<String>,
}

#[cfg(test)]
mod request_tests {
    use super::ChatCompletionRequest;

    #[test]
    fn output_budget_is_only_serialized_when_requested() {
        let messages = [serde_json::json!({"role":"user", "content":"test"})];
        let limited = ChatCompletionRequest {
            model: "test",
            messages: &messages,
            temperature: 0.2,
            stream: true,
            max_completion_tokens: Some(1024),
            response_format: None,
        };
        let value = serde_json::to_value(limited).unwrap();
        assert_eq!(value["max_completion_tokens"], 1024);
        let unlimited = ChatCompletionRequest {
            model: "test",
            messages: &messages,
            temperature: 0.2,
            stream: true,
            max_completion_tokens: None,
            response_format: None,
        };
        let value = serde_json::to_value(unlimited).unwrap();
        assert!(value.get("max_completion_tokens").is_none());
    }
}
