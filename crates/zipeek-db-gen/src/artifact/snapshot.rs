use zipeek_db::native::{Parameters, Connection};
use serde::{Deserialize, Serialize};

use super::{ArtifactError, queries};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalSnapshot {
    pub tables: Vec<String>,
    pub rows: Vec<String>,
    pub search_documents: Vec<String>,
    pub search_keys: Vec<String>,
    pub dictionary_versions: Vec<String>,
    pub vocabulary_identities: Vec<String>,
    pub characters: Vec<String>,
    pub character_readings: Vec<String>,
    pub pinyin_syllable_candidates: Vec<String>,
    pub character_gloss_candidates: Vec<String>,
    pub word_characters: Vec<String>,
    pub word_relations: Vec<String>,
    pub hsk_memberships: Vec<String>,
    pub pos_memberships: Vec<String>,
    pub senses: Vec<String>,
    pub sense_display: Vec<String>,
    pub sense_qualifiers: Vec<String>,
    pub sense_relations: Vec<String>,
    pub categories: Vec<String>,
    pub classification_runs: Vec<String>,
    pub sense_pos: Vec<String>,
    pub sense_categories: Vec<String>,
    pub columns: Vec<String>,
    pub constraints: Vec<String>,
    pub indexes: Vec<String>,
}

pub fn read_snapshot(connection: &Connection) -> Result<LogicalSnapshot, ArtifactError> {
    Ok(LogicalSnapshot {
        tables: query_strings(connection, queries::TABLES, false)?,
        rows: query_strings(connection, queries::ROWS, true)?,
        search_documents: query_strings(connection, queries::SEARCH_DOCUMENTS, true)?,
        search_keys: query_strings(connection, queries::SEARCH_KEYS, true)?,
        dictionary_versions: query_strings(connection, queries::DICTIONARY_VERSIONS, true)?,
        vocabulary_identities: query_strings(connection, queries::VOCABULARY_IDENTITIES, true)?,
        characters: query_strings(connection, queries::CHARACTERS, true)?,
        pinyin_syllable_candidates: query_strings(connection, queries::PINYIN_SYLLABLE_CANDIDATES, true)?,
        character_gloss_candidates: query_strings(connection, queries::CHARACTER_GLOSS_CANDIDATES, true)?,
        character_readings: query_strings(connection, queries::CHARACTER_READINGS, true)?,
        word_relations: query_strings(connection, queries::WORD_RELATIONS, true)?,
        word_characters: query_strings(connection, queries::WORD_CHARACTERS, true)?,
        hsk_memberships: query_strings(connection, queries::HSK_MEMBERSHIPS, true)?,
        pos_memberships: query_strings(connection, queries::POS_MEMBERSHIPS, true)?,
        senses: query_strings(connection, queries::SENSES, true)?,
        sense_display: query_strings(connection, queries::SENSE_DISPLAY, true)?,
        sense_qualifiers: query_strings(connection, queries::SENSE_QUALIFIERS, true)?,
        sense_relations: query_strings(connection, queries::SENSE_RELATIONS, true)?,
        categories: query_strings(connection, queries::CATEGORIES, true)?,
        classification_runs: query_strings(connection, queries::CLASSIFICATION_RUNS, true)?,
        sense_pos: query_strings(connection, queries::SENSE_POS, true)?,
        sense_categories: query_strings(connection, queries::SENSE_CATEGORIES, true)?,
        columns: query_strings(connection, queries::COLUMNS, false)?,
        constraints: query_strings(connection, queries::CONSTRAINTS, false)?,
        indexes: query_strings(connection, queries::INDEXES, false)?,
    })
}

fn query_strings(
    connection: &Connection,
    query: &'static str,
    allow_empty: bool,
) -> Result<Vec<String>, ArtifactError> {
    let failed = |source| ArtifactError::Query {
        query: query.to_owned(),
        source,
    };
    let mut result = connection.query(query, Parameters::None).map_err(failed)?;
    let mut values = Vec::new();
    while let Some(chunk) = result.next_chunk().map_err(failed)? {
        let reader = chunk
            .get_vector_at::<String>(0)
            .map_err(failed)?
            .reader()
            .map_err(failed)?;
        for value in reader.iter() {
            values.push(
                value
                    .ok_or(ArtifactError::MissingValue { query })?
                    .to_owned(),
            );
        }
    }
    if values.is_empty() && !allow_empty {
        return Err(ArtifactError::MissingValue { query });
    }
    Ok(values)
}
