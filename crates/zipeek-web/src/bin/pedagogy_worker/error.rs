use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerErrorCode {
    InvalidRequest,
    InvalidEvidence,
    EvidenceLimitExceeded,
    UnverifiedSnapshot,
    UnexpectedEvidence,
    ProtocolError,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerRuntimeError {
    InvalidRequest(String),
    InvalidEvidence(String),
    EvidenceLimitExceeded(String),
    UnverifiedSnapshot(String),
    UnexpectedEvidence(String),
    Protocol(String),
    Internal(String),
}

impl WorkerRuntimeError {
    pub fn new(code: WorkerErrorCode, message: impl Into<String>) -> Self {
        let message = message.into();
        match code {
            WorkerErrorCode::InvalidRequest => Self::InvalidRequest(message),
            WorkerErrorCode::InvalidEvidence => Self::InvalidEvidence(message),
            WorkerErrorCode::EvidenceLimitExceeded => Self::EvidenceLimitExceeded(message),
            WorkerErrorCode::UnverifiedSnapshot => Self::UnverifiedSnapshot(message),
            WorkerErrorCode::UnexpectedEvidence => Self::UnexpectedEvidence(message),
            WorkerErrorCode::ProtocolError => Self::Protocol(message),
            WorkerErrorCode::InternalError => Self::Internal(message),
        }
    }

    pub const fn code(&self) -> WorkerErrorCode {
        match self {
            Self::InvalidRequest(_) => WorkerErrorCode::InvalidRequest,
            Self::InvalidEvidence(_) => WorkerErrorCode::InvalidEvidence,
            Self::EvidenceLimitExceeded(_) => WorkerErrorCode::EvidenceLimitExceeded,
            Self::UnverifiedSnapshot(_) => WorkerErrorCode::UnverifiedSnapshot,
            Self::UnexpectedEvidence(_) => WorkerErrorCode::UnexpectedEvidence,
            Self::Protocol(_) => WorkerErrorCode::ProtocolError,
            Self::Internal(_) => WorkerErrorCode::InternalError,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::InvalidRequest(message)
            | Self::InvalidEvidence(message)
            | Self::EvidenceLimitExceeded(message)
            | Self::UnverifiedSnapshot(message)
            | Self::UnexpectedEvidence(message)
            | Self::Protocol(message)
            | Self::Internal(message) => message,
        }
    }
}

impl std::fmt::Display for WorkerRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for WorkerRuntimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_error_variant_has_a_stable_wire_code() {
        for code in [
            WorkerErrorCode::InvalidRequest,
            WorkerErrorCode::InvalidEvidence,
            WorkerErrorCode::EvidenceLimitExceeded,
            WorkerErrorCode::UnverifiedSnapshot,
            WorkerErrorCode::UnexpectedEvidence,
            WorkerErrorCode::ProtocolError,
            WorkerErrorCode::InternalError,
        ] {
            let error = WorkerRuntimeError::new(code, "diagnostic");
            assert_eq!(error.code(), code);
            assert_eq!(error.to_string(), "diagnostic");
        }
    }
}
