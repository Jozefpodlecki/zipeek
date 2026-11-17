use crate::structured::ResponseFormat;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(super) struct ChatCompletionRequest<'a, M: Serialize> {
    pub model: &'a str,
    pub messages: &'a [M],
    pub temperature: f32,
    pub stream: bool,
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
}

#[derive(Deserialize)]
pub(super) struct AssistantMessage {
    pub content: Option<String>,
}
