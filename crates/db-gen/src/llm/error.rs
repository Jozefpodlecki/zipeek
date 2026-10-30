use thiserror::Error;

use aimux_core::error::AiMuxError;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("model call failed: {0}")]
    Provider(#[from] AiMuxError),

    #[error("model call timed out after {millis}ms")]
    Timeout { millis: u64 },

    #[error("model call aborted")]
    Aborted,

    #[error("model returned empty response")]
    Empty,

    #[error("invalid model response: {0}")]
    Invalid(String),
}

pub type LlmResult<T> = Result<T, LlmError>;