use std::num::NonZeroU32;
use std::sync::Arc;

use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::sampling::LlamaSampler;

use crate::model_runtime::stream::{run_generation, spawn_generation_worker};
use crate::model_runtime::{
    GenerationCancellation, GenerationRequest, GenerationStream, ModelRuntimeError, TokenSource,
};

use super::LoadedModel;
use super::error::LlamaInferenceError;
use super::text_decoder::TokenTextDecoder;

struct LlamaTokenSource<'a> {
    model: &'a LlamaModel,
    context: LlamaContext<'a>,
    sampler: LlamaSampler,
    decoder: TokenTextDecoder,
    context_limit: i32,
    position: i32,
    batch_size: usize,
    prompt_limit: usize,
    ready: bool,
    parse_special: bool,
    cancellation: GenerationCancellation,
}

impl<'a> LlamaTokenSource<'a> {
    fn new(
        model: &'a LlamaModel,
        backend: &llama_cpp_2::llama_backend::LlamaBackend,
        request: &GenerationRequest,
        cancellation: GenerationCancellation,
        parse_special: bool,
    ) -> Result<Self, ModelRuntimeError> {
        let params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(request.config.context_size))
            .with_n_batch(request.config.batch_size);
        let context = model
            .new_context(backend, params)
            .map_err(|error| LlamaInferenceError::Context(error.to_string()))?;
        Ok(Self {
            model,
            context,
            sampler: LlamaSampler::greedy(),
            decoder: TokenTextDecoder::new(),
            context_limit: i32::try_from(request.config.context_size).map_err(backend_error)?,
            position: 0,
            batch_size: request.config.batch_size as usize,
            prompt_limit: request.config.prompt_token_limit() as usize,
            ready: false,
            parse_special,
            cancellation,
        })
    }
}

impl TokenSource for LlamaTokenSource<'_> {
    fn prompt_tokens(&mut self, prompt: &str) -> Result<usize, ModelRuntimeError> {
        let tokens = self
            .model
            .vocab()
            .tokenize(prompt.as_bytes(), true, self.parse_special);
        if tokens.is_empty() {
            return Err(ModelRuntimeError::InvalidGenerationConfig(
                "tokenized prompt cannot be empty",
            ));
        }
        let token_count = tokens.len();
        if token_count > self.prompt_limit {
            return Err(ModelRuntimeError::InvalidGenerationConfig(
                "tokenized prompt exceeds reserved context budget",
            ));
        }
        let last = token_count - 1;
        for (offset, chunk) in tokens.chunks(self.batch_size).enumerate() {
            if self.cancellation.is_cancelled() {
                return Ok(token_count);
            }
            let mut batch = LlamaBatch::new(self.batch_size, 1);
            for (index, &token) in chunk.iter().enumerate() {
                let absolute = offset * self.batch_size + index;
                let position = i32::try_from(absolute).map_err(backend_error)?;
                batch
                    .add(token, position, &[0], absolute == last)
                    .map_err(|error| LlamaInferenceError::Batch(error.to_string()))?;
            }
            self.context
                .decode(&mut batch)
                .map_err(|error| LlamaInferenceError::PromptDecode(error.to_string()))?;
        }
        self.position = i32::try_from(token_count).map_err(backend_error)?;
        self.ready = true;
        Ok(token_count)
    }

    fn next_token(&mut self) -> Result<Option<String>, ModelRuntimeError> {
        if self.cancellation.is_cancelled() {
            return Ok(None);
        }
        if !self.ready {
            return Err(ModelRuntimeError::Backend(
                "prompt has not been decoded".into(),
            ));
        }
        if self.position >= self.context_limit {
            return Err(ModelRuntimeError::InvalidGenerationConfig(
                "generation exceeds context capacity",
            ));
        }
        let token = self.sampler.sample(&self.context, -1);
        self.sampler.accept(token);
        if self.model.vocab().is_eog(token) {
            return Ok(None);
        }
        let piece = self.model.vocab().token_to_piece(token, false, None);
        let mut batch = LlamaBatch::new(1, 1);
        batch
            .add(token, self.position, &[0], true)
            .map_err(|error| LlamaInferenceError::Batch(error.to_string()))?;
        self.context
            .decode(&mut batch)
            .map_err(|error| LlamaInferenceError::TokenDecode(error.to_string()))?;
        self.position =
            self.position
                .checked_add(1)
                .ok_or(ModelRuntimeError::InvalidGenerationConfig(
                    "generation position overflow",
                ))?;
        Ok(Some(self.decoder.decode(&piece, false)))
    }

    fn finish_text(&mut self) -> Result<String, ModelRuntimeError> {
        Ok(self.decoder.decode(&[], true))
    }
}

fn backend_error(error: impl std::fmt::Display) -> ModelRuntimeError {
    ModelRuntimeError::Backend(error.to_string())
}

pub fn generate_chat(
    model: Arc<LoadedModel>,
    request: GenerationRequest,
    cancellation: GenerationCancellation,
) -> Result<GenerationStream, ModelRuntimeError> {
    generate_with_format(model, request, cancellation, true)
}

pub fn generate(
    model: Arc<LoadedModel>,
    request: GenerationRequest,
    cancellation: GenerationCancellation,
) -> Result<GenerationStream, ModelRuntimeError> {
    generate_with_format(model, request, cancellation, false)
}

fn generate_with_format(
    model: Arc<LoadedModel>,
    request: GenerationRequest,
    cancellation: GenerationCancellation,
    parse_special: bool,
) -> Result<GenerationStream, ModelRuntimeError> {
    spawn_generation_worker(
        request,
        cancellation,
        move |request, cancellation, sender| {
            if cancellation.is_cancelled() {
                return Ok(());
            }
            let mut source = LlamaTokenSource::new(
                model.model(),
                model.backend(),
                &request,
                cancellation.clone(),
                parse_special,
            )?;
            run_generation(&mut source, request, cancellation, sender)
        },
    )
}
