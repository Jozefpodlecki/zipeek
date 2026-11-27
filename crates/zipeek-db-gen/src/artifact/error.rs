use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error(transparent)]
    Classification(#[from] crate::llm::classification::ClassificationError),
    #[error(transparent)]
    Preparation(#[from] crate::GeneratorError),
    #[error(transparent)]
    Database(#[from] crate::database::DbError),
    #[error(transparent)]
    Import(#[from] crate::import::ImportError),
    #[error("artifact {operation} failed at {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("artifact output {path} already exists; use a fresh output directory")]
    ExistingOutput { path: PathBuf },
    #[error("invalid artifact path: {path}")]
    InvalidPath { path: PathBuf },
    #[error("artifact SQL operation {query} failed: {source}")]
    Query {
        query: String,
        #[source]
        source: zipeek_db::native::Error,
    },
    #[error("artifact write failed ({original}); rollback also failed: {rollback}")]
    Rollback {
        original: Box<ArtifactError>,
        rollback: zipeek_db::native::Error,
    },
    #[error("artifact query {query} returned a NULL or missing result")]
    MissingValue { query: &'static str },
    #[error("artifact JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("artifact logical content differs after {stage}")]
    LogicalMismatch { stage: &'static str },
    #[error("artifact notice filename is invalid or collides: {name}")]
    InvalidNotice { name: String },
}

impl From<crate::database::StatementError> for ArtifactError {
    fn from(error: crate::database::StatementError) -> Self {
        Self::Query {
            query: error.query,
            source: error.source,
        }
    }
}
