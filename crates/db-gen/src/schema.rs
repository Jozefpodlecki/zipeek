use crate::database::DbError;
use duckdb_neo::{Parameters, connection::Connection};

pub const WORDS_DDL: &str = include_str!("schema/words.sql");
pub const HSK_INDEX_DDL: [&str; 2] = [
    "CREATE INDEX idx_words_hsk3 ON words (hsk3_level)",
    "CREATE INDEX idx_words_hsk2 ON words (hsk2_level)",
];
pub fn create_schema(connection: &Connection) -> Result<(), DbError> {
    for query in std::iter::once(WORDS_DDL).chain(HSK_INDEX_DDL) {
        connection
            .execute(query, Parameters::None)
            .map_err(|source| DbError::Query { query, source })?;
    }
    Ok(())
}
