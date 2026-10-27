use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("CC-CEDICT is not UTF-8: {0}")]
    CedictUtf8(#[source] std::str::Utf8Error),
    #[error("invalid CC-CEDICT line {line}: {reason}; input: {original:?}")]
    CedictLine {
        line: usize,
        original: String,
        reason: &'static str,
    },
    #[error("invalid HSK JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HSK source provenance did not match the parsed entry count")]
    Provenance,
    #[error(
        "display key conflict for {simplified:?}, {pinyin:?}; first source row: {first}; conflicting source row: {second}"
    )]
    DisplayKeyConflict {
        simplified: String,
        pinyin: String,
        first: String,
        second: String,
    },
    #[error("cannot assign SQL INTEGER word id for row {row}")]
    IdOverflow { row: usize },
    #[error("required verified source {name:?} is missing from the manifest")]
    MissingSource { name: &'static str },
    #[error("HSK insertion failed at word {word_id:?}: {source}")]
    Insert {
        word_id: Option<i32>,
        #[source]
        source: duckdb_neo::Error,
    },
    #[error("HSK insertion failed ({original}); rollback also failed: {rollback}")]
    Rollback {
        original: Box<ImportError>,
        rollback: duckdb_neo::Error,
    },
}
