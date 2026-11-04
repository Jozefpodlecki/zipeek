use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Dictionary is not ready; wait for readiness or retry initialization")]
    NotReady,
    #[error("{operation}: {message}")]
    Operation {
        operation: &'static str,
        message: String,
    },
    #[error("Invalid dictionary row {id}: {message}")]
    InvalidRow { id: i32, message: String },
    #[error("Cannot decode dictionary response: {0}")]
    Decode(String),
}
