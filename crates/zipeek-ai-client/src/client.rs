#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use futures_util::StreamExt;

use serde::Serialize;
use serde_json::json;

use crate::StreamUpdate;
use crate::config::chat_url;
use crate::error::{http_failure, request_error};
use crate::prompts::{
    TranslationDirection, connectivity_messages, translation_messages, validate_source,
};
use crate::protocol::{ChatCompletionRequest, ChatCompletionResponse};
use crate::stream::SseDecoder;
use crate::{AiClientError, ResponseFormat};

#[derive(Clone, Debug)]
pub struct AiClient(ClientConfig);

#[derive(Clone, Debug)]
struct ClientConfig {
    endpoint: String,
    model: String,
    http: reqwest::Client,
}

impl AiClient {
    pub fn new(endpoint: &str, model: &str) -> Result<Self, AiClientError> {
        let model = model.trim();
        if model.is_empty() {
            return Err(AiClientError(
                "Configure your AI model in Settings first".to_owned(),
            ));
        }
        chat_url(endpoint, "")?;

        #[cfg(not(target_arch = "wasm32"))]
        let builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .redirect(reqwest::redirect::Policy::none());

        #[cfg(target_arch = "wasm32")]
        let builder = reqwest::Client::builder();

        let http = builder
            .build()
            .map_err(|_| AiClientError("Could not initialize AI HTTP client".to_owned()))?;
        Ok(Self(ClientConfig {
            endpoint: endpoint.trim().to_owned(),
            model: model.to_owned(),
            http,
        }))
    }

    pub async fn complete_with_updates<F, M>(
        &self,
        messages: Vec<M>,
        api_key: &str,
        on_update: F,
    ) -> Result<String, AiClientError>
    where
        F: FnMut(StreamUpdate),
        M: Serialize,
    {
        self.complete_with_format(messages, api_key, None, on_update)
            .await
    }

    pub async fn complete_with_format<F, M>(
        &self,
        messages: Vec<M>,
        api_key: &str,
        response_format: Option<&ResponseFormat>,
        on_update: F,
    ) -> Result<String, AiClientError>
    where
        F: FnMut(StreamUpdate),
        M: Serialize,
    {
        self.complete_with_budget(messages, api_key, response_format, None, on_update)
            .await
    }

    pub async fn complete_with_budget<F, M>(
        &self,
        messages: Vec<M>,
        api_key: &str,
        response_format: Option<&ResponseFormat>,
        max_completion_tokens: Option<u32>,
        mut on_update: F,
    ) -> Result<String, AiClientError>
    where
        F: FnMut(StreamUpdate),
        M: Serialize,
    {
        if max_completion_tokens.is_some_and(|limit| !(64..=16_384).contains(&limit)) {
            return Err(AiClientError("AI output token budget must be between 64 and 16384".to_owned()));
        }
        if messages.is_empty() {
            return Err(AiClientError("AI request requires messages".to_owned()));
        }
        let endpoint = chat_url(&self.0.endpoint, api_key)?;
        let body = ChatCompletionRequest {
            model: &self.0.model,
            messages: &messages,
            temperature: 0.2,
            stream: true,
            max_completion_tokens,
            response_format,
        };
        let mut request = self
            .0
            .http
            .post(endpoint.clone())
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .json(&body);
        if !api_key.is_empty() {
            request = request.bearer_auth(api_key);
        }
        let response = request.send().await.map_err(request_error)?;
        if response.url() != &endpoint {
            return Err(AiClientError(
                "AI endpoint redirected to a different URL; use the final /v1/ base URL directly"
                    .to_owned(),
            ));
        }
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.map_err(request_error)?;
            return Err(http_failure(status, &body, api_key));
        }

        let is_event_stream = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.to_ascii_lowercase().starts_with("text/event-stream"));
        if !is_event_stream {
            let payload: ChatCompletionResponse = response.json().await.map_err(|_| {
                AiClientError(
                    "AI returned invalid JSON instead of a chat completion or SSE stream"
                        .to_owned(),
                )
            })?;
            let choice = payload.choices.into_iter().next()
                .ok_or_else(|| AiClientError("AI returned no completion choices".to_owned()))?;
            if choice.finish_reason.as_deref() == Some("length") {
                return Err(AiClientError("AI stopped because its output token limit was reached".to_owned()));
            }
            let content = choice.message.content
                .filter(|content| !content.trim().is_empty())
                .ok_or_else(|| AiClientError("AI returned no text response. Check whether the model supports chat completions.".to_owned()))?;
            on_update(StreamUpdate::Content(content.clone()));
            return Ok(content.trim().to_owned());
        }

        let mut decoder = SseDecoder::default();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(request_error)?;
            decoder.push(&chunk, &mut on_update)?;
            if decoder.is_done() {
                break;
            }
        }
        decoder.finish(&mut on_update)
    }

    pub async fn complete_streaming<F, M>(
        &self,
        messages: Vec<M>,
        api_key: &str,
        mut on_delta: F,
    ) -> Result<String, AiClientError>
    where
        F: FnMut(&str),
        M: Serialize,
    {
        self.complete_with_updates(messages, api_key, |update| {
            if let StreamUpdate::Content(content) = update {
                on_delta(&content);
            }
        })
        .await
    }

    pub async fn complete<M: Serialize>(
        &self,
        messages: Vec<M>,
        api_key: &str,
    ) -> Result<String, AiClientError> {
        self.complete_streaming(messages, api_key, |_| {}).await
    }

    pub async fn translate(
        &self,
        source: &str,
        direction: TranslationDirection,
        api_key: &str,
    ) -> Result<String, AiClientError> {
        let source = validate_source(source)?;
        self.complete(translation_messages(source, direction), api_key)
            .await
    }

    pub async fn test_connection(&self, api_key: &str) -> Result<String, AiClientError> {
        let reply = self.complete(connectivity_messages(), api_key).await?;
        Ok(format!(
            "Connection succeeded. Model responded ({} characters).",
            reply.chars().count()
        ))
    }

    pub async fn connection_details(&self, api_key: &str) -> Result<String, AiClientError> {
        let messages = connectivity_messages();
        let reply = self.complete(messages.clone(), api_key).await?;
        let clipped: String = reply.chars().take(600).collect();
        Ok(json!({
            "model": self.0.model,
            "messages": messages,
            "reply": clipped,
            "truncated": reply.chars().count() > 600,
        })
        .to_string())
    }
}
