use std::sync::Arc;

use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::{LlamaModel, params::LlamaModelParams};

use crate::model_runtime::{ModelRuntimeConfig, ModelRuntimeError};

pub struct LoadedModel {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
}

impl LoadedModel {
    pub async fn load(config: ModelRuntimeConfig) -> Result<Self, ModelRuntimeError> {
        config.validate()?;
        super::initialize_logging();
        tokio::task::spawn_blocking(move || {
            let backend = LlamaBackend::init()
                .map_err(|error| ModelRuntimeError::Backend(error.to_string()))?;
            let params = LlamaModelParams::default().with_n_gpu_layers(config.gpu_layers);
            let model = LlamaModel::load_from_file(&backend, &config.model_path, &params)
                .map_err(|error| ModelRuntimeError::Backend(error.to_string()))?;
            Ok(Self {
                backend: Arc::new(backend),
                model: Arc::new(model),
            })
        })
        .await
        .map_err(|error| ModelRuntimeError::Task(error.to_string()))?
    }

    pub fn model(&self) -> &LlamaModel {
        &self.model
    }

    pub fn backend(&self) -> &LlamaBackend {
        &self.backend
    }
}
