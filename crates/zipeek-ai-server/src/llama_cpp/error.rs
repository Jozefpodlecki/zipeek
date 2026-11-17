#[derive(Debug, thiserror::Error)]
pub enum LlamaInferenceError {
    #[error("failed to create inference context: {0}")]
    Context(String),
    #[error("failed to construct token batch: {0}")]
    Batch(String),
    #[error("failed to decode prompt: {0}")]
    PromptDecode(String),
    #[error("failed to decode generated token: {0}")]
    TokenDecode(String),
}
