use zipeek_db::native::Connection;

use crate::{database::DatabaseSession, search};

use super::ArtifactError;

pub(super) fn create(connection: &Connection) -> Result<(), ArtifactError> {
    let session = DatabaseSession::new(connection);
    session.execute("INSTALL fts")?;
    session.execute("LOAD fts")?;
    session.execute(
        "PRAGMA create_fts_index('word_search_documents', 'word_id', 'english_text', stemmer = 'english', stopwords = 'none')",
    )?;
    verify_loaded(connection)
}

pub(super) fn verify(connection: &Connection) -> Result<(), ArtifactError> {
    DatabaseSession::new(connection).execute("LOAD fts")?;
    verify_loaded(connection)
}

fn verify_loaded(connection: &Connection) -> Result<(), ArtifactError> {
    let schema_count = search::count(
        connection,
        "SELECT count(*) FROM duckdb_schemas() WHERE database_name = current_database() AND schema_name = 'fts_main_word_search_documents'",
    )?;
    if schema_count != 1 {
        return Err(ArtifactError::LogicalMismatch {
            stage: "FTS schema verification",
        });
    }

    let matches = search::count(
        connection,
        "SELECT count(*) FROM (SELECT fts_main_word_search_documents.match_bm25(word_id, 'language') AS score FROM word_search_documents) AS ranked WHERE score IS NOT NULL",
    )?;
    if matches == 0 {
        let expected_matches = search::count(
            connection,
            "SELECT count(*) FROM word_search_documents WHERE lower(english_text) LIKE '%language%'",
        )?;
        if expected_matches > 0 {
            return Err(ArtifactError::LogicalMismatch {
                stage: "FTS result verification",
            });
        }
    }
    Ok(())
}

