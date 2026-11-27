use std::{num::NonZeroU32, path::PathBuf, sync::Arc};

use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{LlamaChatMessage, LlamaModel, params::LlamaModelParams},
    sampling::LlamaSampler,
};
use zipeek_ai_core::{
    ChatRole, GenerationBackend, GenerationCancellation, GenerationError, GenerationEvent,
    GenerationFuture, GenerationRequest,
};

pub struct LocalModelConfig {
    pub path: PathBuf,
    pub gpu_layers: u32,
    pub context_tokens: u32,
}

impl LocalModelConfig {
    pub fn from_environment() -> Result<Self, String> {
        let path = match std::env::var_os("ZIPEEK_AI_MODEL_PATH") {
            Some(path) => PathBuf::from(path),
            None => find_model_in_directory()?,
        };
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("gguf") {
            return Err(format!("GGUF model not found at {}", path.display()));
        }
        let gpu_layers = number_from_env("ZIPEEK_AI_GPU_LAYERS", 0)?;
        let context_tokens = number_from_env("ZIPEEK_AI_CONTEXT", 4096)?;
        if !(512..=131_072).contains(&context_tokens) {
            return Err("ZIPEEK_AI_CONTEXT must be between 512 and 131072".into());
        }
        Ok(Self {
            path,
            gpu_layers,
            context_tokens,
        })
    }
}

fn number_from_env(name: &str, default: u32) -> Result<u32, String> {
    std::env::var(name)
        .map(|text| text.parse::<u32>().map_err(|_| format!("Invalid {name}")))
        .unwrap_or(Ok(default))
}

fn find_model_in_directory() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    let executable_directory = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from));
    let path = [
        Some(cwd.join("models/ai")),
        executable_directory.map(|directory| directory.join("models/ai")),
    ]
    .into_iter()
    .flatten()
    .find(|candidate| candidate.is_dir())
    .ok_or("Configure ZIPEEK_AI_MODEL_PATH or install one GGUF in models/ai")?;
    let mut models = std::fs::read_dir(&path)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|candidate| {
            candidate.is_file()
                && candidate
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("gguf")
        });
    let model = models.next().ok_or("No GGUF model found in models/ai")?;
    if models.next().is_some() {
        return Err("Multiple GGUF files found; set ZIPEEK_AI_MODEL_PATH explicitly".into());
    }
    Ok(model)
}

#[derive(Clone)]
pub struct LocalLlama {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    context_tokens: u32,
}

impl LocalLlama {
    pub async fn load(config: LocalModelConfig) -> Result<Self, String> {
        tokio::task::spawn_blocking(move || {
            let backend = LlamaBackend::init().map_err(|error| error.to_string())?;
            let params = LlamaModelParams::default().with_n_gpu_layers(config.gpu_layers);
            let model = LlamaModel::load_from_file(&backend, &config.path, &params)
                .map_err(|error| error.to_string())?;
            Ok(Self {
                backend: Arc::new(backend),
                model: Arc::new(model),
                context_tokens: config.context_tokens,
            })
        })
        .await
        .map_err(|error| error.to_string())?
    }
}

fn prompt_for_model(
    model: &LlamaModel,
    request: &GenerationRequest,
) -> Result<String, GenerationError> {
    let template = model.chat_template(None).map_err(|error| {
        GenerationError::Backend(format!("GGUF chat template unavailable: {error}"))
    })?;
    let messages = request
        .messages
        .iter()
        .map(|message| {
            let role = match message.role {
                ChatRole::System => "system",
                ChatRole::User => "user",
                ChatRole::Assistant => "assistant",
            };
            LlamaChatMessage::new(role.to_owned(), message.content.clone())
                .map_err(|error| GenerationError::Backend(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    model
        .apply_chat_template(&template, &messages, true)
        .map_err(|error| {
            GenerationError::Backend(format!("Cannot render model chat template: {error}"))
        })
}

fn produce(
    engine: &LocalLlama,
    request: GenerationRequest,
    cancellation: GenerationCancellation,
    sender: tokio::sync::mpsc::Sender<GenerationEvent>,
) -> Result<(), GenerationError> {
    request.validate()?;
    let context_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(engine.context_tokens))
        .with_n_batch(512);
    let mut context = engine
        .model
        .new_context(&engine.backend, context_params)
        .map_err(|error| GenerationError::Backend(error.to_string()))?;
    let prompt = prompt_for_model(&engine.model, &request)?;
    let tokens = engine.model.vocab().tokenize(prompt.as_bytes(), true, true);
    let output_limit = request.options.max_output_tokens;
    let prompt_limit = engine.context_tokens.saturating_sub(output_limit);
    if tokens.is_empty() || tokens.len() as u32 > prompt_limit {
        return Err(GenerationError::InvalidRequest(
            "prompt exceeds configured context budget",
        ));
    }
    let mut position = 0_i32;
    for chunk in tokens.chunks(512) {
        if cancellation.is_cancelled() {
            return Err(GenerationError::Cancelled);
        }
        let mut batch = LlamaBatch::new(512, 1);
        for (index, token) in chunk.iter().enumerate() {
            let last = position as usize + index + 1 == tokens.len();
            batch
                .add(*token, position + index as i32, &[0], last)
                .map_err(|error| GenerationError::Backend(error.to_string()))?;
        }
        context
            .decode(&mut batch)
            .map_err(|error| GenerationError::Backend(error.to_string()))?;
        position += chunk.len() as i32;
    }
    let mut sampler = if request.options.temperature <= 0.0 {
        LlamaSampler::greedy()
    } else {
        LlamaSampler::chain(
            [
                LlamaSampler::temp(request.options.temperature),
                LlamaSampler::dist(42),
            ],
            false,
        )
    };
    let mut pending = Vec::<u8>::new();
    for _ in 0..output_limit {
        if cancellation.is_cancelled() {
            return Err(GenerationError::Cancelled);
        }
        let token = sampler.sample(&context, -1);
        sampler.accept(token);
        if engine.model.vocab().is_eog(token) {
            break;
        }
        pending.extend(engine.model.vocab().token_to_piece(token, false, None));
        if let Ok(text) = std::str::from_utf8(&pending) {
            if !text.is_empty()
                && sender
                    .blocking_send(GenerationEvent::Content(text.to_owned()))
                    .is_err()
            {
                return Err(GenerationError::Cancelled);
            }
            pending.clear();
        } else if pending.len() > 8_192 {
            return Err(GenerationError::Backend(
                "Invalid model UTF-8 output".into(),
            ));
        }
        let mut batch = LlamaBatch::new(1, 1);
        batch
            .add(token, position, &[0], true)
            .map_err(|error| GenerationError::Backend(error.to_string()))?;
        context
            .decode(&mut batch)
            .map_err(|error| GenerationError::Backend(error.to_string()))?;
        position += 1;
    }
    if !pending.is_empty() {
        let text = String::from_utf8_lossy(&pending).into_owned();
        let _ = sender.blocking_send(GenerationEvent::Content(text));
    }
    let _ = sender.blocking_send(GenerationEvent::Complete);
    Ok(())
}

impl GenerationBackend for LocalLlama {
    fn generate<'a>(
        &'a self,
        request: GenerationRequest,
        cancellation: GenerationCancellation,
        on_event: &'a mut (dyn FnMut(GenerationEvent) + Send),
    ) -> GenerationFuture<'a> {
        Box::pin(async move {
            let (sender, mut receiver) = tokio::sync::mpsc::channel(16);
            let task_cancellation = cancellation.clone();
            let engine = LocalLlama::clone(self);
            let task = tokio::task::spawn_blocking(move || {
                produce(&engine, request, task_cancellation, sender)
            });
            while let Some(event) = receiver.recv().await {
                if cancellation.is_cancelled() {
                    break;
                }
                on_event(event);
            }
            if cancellation.is_cancelled() {
                return Err(GenerationError::Cancelled);
            }
            task.await
                .map_err(|error| GenerationError::Backend(error.to_string()))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_model_path() {
        let config = LocalModelConfig {
            path: PathBuf::from("nonexistent-test-model.gguf"),
            gpu_layers: 0,
            context_tokens: 4096,
        };
        assert!(!config.path.is_file());
    }
}
