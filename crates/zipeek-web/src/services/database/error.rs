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
    #[error("Dictionary manifest validation failed: {0}")]
    Manifest(String),
    #[error("Cannot decode dictionary response: {0}")]
    Decode(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_manifest_error_is_distinct_from_query_decoding() {
        let error = DbError::Manifest("ambiguous database entries".to_owned());
        assert_eq!(
            error.to_string(),
            "Dictionary manifest validation failed: ambiguous database entries"
        );
    }
}
