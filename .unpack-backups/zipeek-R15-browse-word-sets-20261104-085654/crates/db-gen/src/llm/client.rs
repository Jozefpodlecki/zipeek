use aimux_core::generate::{GenerateTextOptions, GenerateTextResult, stream_text};
use aimux_core::language_model::LanguageModel;
use aimux_core::message::ModelMessage;
use aimux_providers::{PresetSettings, create_provider};
use std::sync::Arc;
use tracing::debug;

use super::error::{LlmError, LlmResult};
use super::streaming::{consume_progress, consume_silent};

#[derive(Clone)]
pub struct LlmClient {
    base_url: String,
    timeout: Option<aimux_core::options::TimeoutConfiguration>,
    max_retries: u32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub request_timeout_ms: u64,
    pub first_chunk_timeout_ms: u64,
    pub max_retries: u32,
}

impl LlmClient {
    pub fn new(config: &Config) -> Self {
        let base = config.base_url.trim_end_matches('/');
        let base_url = if base.ends_with("/v1") {
            base.to_owned()
        } else {
            format!("{base}/v1")
        };
        Self {
            base_url,
            timeout: Some(aimux_core::options::TimeoutConfiguration {
                total_ms: Some(config.request_timeout_ms),
                first_chunk_ms: Some(config.first_chunk_timeout_ms),
                step_ms: None,
                chunk_ms: None,
            }),
            max_retries: config.max_retries,
        }
    }

    fn model(&self, model_id: &str) -> LlmResult<Arc<dyn LanguageModel>> {
        let provider = create_provider(
            "openai",
            PresetSettings {
                api_key: Some("lmstudio".to_owned().into()),
                base_url: Some(self.base_url.clone()),
                ..Default::default()
            },
        )?;
        provider.language_model(model_id).map_err(LlmError::Provider)
    }

    fn options(
        &self,
        tools: Option<Vec<aimux_core::tool::Tool>>,
        instructions: Option<String>,
    ) -> GenerateTextOptions {
        GenerateTextOptions {
            tools,
            timeout: self.timeout.clone(),
            instructions,
            max_retries: Some(self.max_retries),
            ..Default::default()
        }
    }

    pub async fn chat(
        &self,
        model_id: &str,
        messages: &[ModelMessage],
        tools: Option<Vec<aimux_core::tool::Tool>>,
        instructions: Option<String>,
    ) -> LlmResult<GenerateTextResult> {
        let model = self.model(model_id)?;
        let stream = stream_text(model.as_ref(), messages.to_vec(), self.options(tools, instructions))
            .await
            .map_err(LlmError::Provider)?;
        consume_progress(stream).await
    }

    pub async fn chat_silent(
        &self,
        model_id: &str,
        messages: &[ModelMessage],
        tools: Option<Vec<aimux_core::tool::Tool>>,
        instructions: Option<String>,
    ) -> LlmResult<GenerateTextResult> {
        let model = self.model(model_id)?;
        let stream = stream_text(model.as_ref(), messages.to_vec(), self.options(tools, instructions))
            .await
            .map_err(LlmError::Provider)?;
        consume_silent(stream).await
    }

    pub async fn complete(&self, model_id: &str, prompt: &str) -> LlmResult<GenerateTextResult> {
        let model = self.model(model_id)?;
        debug!(model = model_id, prompt_chars = prompt.len(), "complete");
        let stream = stream_text(model.as_ref(), prompt.to_owned(), self.options(None, None))
            .await
            .map_err(LlmError::Provider)?;
        consume_silent(stream).await
    }
}
