use std::sync::Arc;

use aimux_core::generate::{GenerateTextOptions, StreamTextResult};
use aimux_core::message::ModelMessage;
use aimux_core::prelude::*;
use aimux_core::result::StreamTextResultAggregated;
use aimux_providers::lmstudio::{LmStudioConfig, LmStudioProvider};
use tracing::debug;

use super::error::{LlmError, LlmResult};
use super::streaming::{consume_progress, consume_silent};

#[derive(Clone)]
pub struct LlmClient {
    inner: Arc<LmStudioProvider>,
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
        let base = if config.base_url.contains("/v1") {
            config.base_url.clone()
        } else {
            format!("{}/v1", config.base_url.trim_end_matches('/'))
        };
        
        let lm = LmStudioConfig::new("lm-studio").with_base_url(base);
        let provider = LmStudioProvider::new(lm);

        let timeout = Some(aimux_core::options::TimeoutConfiguration {
            total_ms: Some(config.request_timeout_ms),
            first_chunk_ms: Some(config.first_chunk_timeout_ms),
            step_ms: None,
            chunk_ms: None,
        });

        Self {
            inner: provider.into(),
            timeout,
            max_retries: config.max_retries,
        }
    }

    pub async fn get_models(&self) -> LlmResult<Vec<RuntimeModel>> {
        self.inner.list_models().await.map_err(LlmError::Provider)
    }

    async fn build_options(
        &self,
        tools: Option<Vec<aimux_core::tool::Tool>>,
        instructions: Option<String>,
        response_format: Option<aimux_core::options::ResponseFormat>,
    ) -> GenerateTextOptions {
        GenerateTextOptions {
            tools,
            timeout: self.timeout.clone(),
            instructions,
            response_format,
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
    ) -> LlmResult<StreamTextResultAggregated> {
        let model = self.inner.model(model_id);
        let options = self.build_options(tools, instructions, None).await;
        let stream = stream_text(&model, messages.to_vec(), options)
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
    ) -> LlmResult<StreamTextResultAggregated> {
        let model = self.inner.model(model_id);
        let options = self.build_options(tools, instructions, None).await;
        let stream = stream_text(&model, messages.to_vec(), options)
            .await
            .map_err(LlmError::Provider)?;
        consume_silent(stream).await
    }

    pub async fn complete(
        &self,
        model_id: &str,
        prompt: &str,
    ) -> LlmResult<StreamTextResultAggregated> {
        let model = self.inner.model(model_id);
        let options = self.build_options(None, None, None).await;
        debug!(model = model_id, prompt_chars = prompt.len(), "complete");
        let stream = stream_text(&model, prompt.to_string(), options)
            .await
            .map_err(LlmError::Provider)?;
        consume_silent(stream).await
    }
}