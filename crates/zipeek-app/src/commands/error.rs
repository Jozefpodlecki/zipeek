use serde::Serialize;
use thiserror::Error;

use crate::database::DatabaseError;
use crate::services::ServiceError;

#[derive(Debug, Error, Serialize)]
#[serde(untagged)]
pub enum CommandError {
    #[error(transparent)]
    Service(#[from] ServiceError),

    #[error("{0}")]
    Image(String),
    #[error("{0}")]
    Speech(String),
    #[error("{0}")]
    Ai(String),
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_error_converts_and_retains_ipc_shape() {
        let error: CommandError = ServiceError::ExecutableMissing("missing".into()).into();
        assert_eq!(error.to_string(), "missing");
        assert_eq!(
            serde_json::to_value(error).expect("service error serialization"),
            serde_json::json!({"kind":"executable_missing","details":"missing"})
        );
    }

    #[test]
    fn database_error_converts_and_retains_ipc_shape() {
        let error: CommandError = DatabaseError::WorkerUnavailable.into();
        assert_eq!(error.to_string(), "Native database worker exited");
        assert_eq!(
            serde_json::to_value(error).expect("database error serialization"),
            serde_json::json!({"kind":"worker_unavailable"})
        );
    }
}
