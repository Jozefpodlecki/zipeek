use duckdb_neo::{Parameters, connection::Connection};
use serde::{Deserialize, Serialize};

use super::ArtifactError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalSnapshot {
    pub tables: Vec<String>,
    pub rows: Vec<String>,
    pub dictionary_versions: Vec<String>,
    pub vocabulary_identities: Vec<String>,
    pub characters: Vec<String>,
    pub character_readings: Vec<String>,
    pub word_characters: Vec<String>,
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
        tables: query_strings(
            connection,
            "SELECT table_name FROM duckdb_tables() WHERE database_name = current_database() AND schema_name = 'main' AND NOT internal ORDER BY table_name",
            false,
        )?,
        rows: query_strings(
            connection,
            "SELECT to_json(words)::VARCHAR FROM words ORDER BY id",
            true,
        )?,
        dictionary_versions: query_strings(
            connection,
            "SELECT to_json(dictionary_versions)::VARCHAR FROM dictionary_versions ORDER BY snapshot_fingerprint",
            true,
        )?,
        vocabulary_identities: query_strings(
            connection,
            "SELECT to_json(vocabulary_identities)::VARCHAR FROM vocabulary_identities ORDER BY word_id",
            true,
        )?,
        characters: query_strings(
            connection,
            "SELECT to_json(characters)::VARCHAR FROM characters ORDER BY unicode_scalar",
            true,
        )?,
        character_readings: query_strings(
            connection,
            "SELECT to_json(character_readings)::VARCHAR FROM character_readings ORDER BY character, pinyin_numeric",
            true,
        )?,
        word_characters: query_strings(
            connection,
            "SELECT to_json(word_characters)::VARCHAR FROM word_characters ORDER BY word_id, character_position",
            true,
        )?,
        hsk_memberships: query_strings(
            connection,
            "SELECT to_json(struct_pack(word_id := word_id, system := system, level := level))::VARCHAR FROM word_hsk_membership ORDER BY word_id, system",
            true,
        )?,
        pos_memberships: query_strings(
            connection,
            "SELECT to_json(struct_pack(word_id := word_id, pos := pos))::VARCHAR FROM word_pos_membership ORDER BY word_id, pos",
            true,
        )?,
        senses: query_strings(
            connection,
            "SELECT to_json(word_senses)::VARCHAR FROM word_senses ORDER BY word_id, sense_order",
            true,
        )?,
        sense_display: query_strings(
            connection,
            "SELECT to_json(sense_display)::VARCHAR FROM sense_display ORDER BY word_id, sense_order",
            true,
        )?,
        sense_qualifiers: query_strings(
            connection,
            "SELECT to_json(sense_qualifiers)::VARCHAR FROM sense_qualifiers ORDER BY word_id, sense_order, qualifier_kind, qualifier_code",
            true,
        )?,
        sense_relations: query_strings(
            connection,
            "SELECT to_json(sense_relations)::VARCHAR FROM sense_relations ORDER BY word_id, sense_order, relation_order",
            true,
        )?,
        categories: query_strings(
            connection,
            "SELECT to_json(categories)::VARCHAR FROM categories ORDER BY slug",
            true,
        )?,
        classification_runs: query_strings(
            connection,
            "SELECT to_json(classification_runs)::VARCHAR FROM classification_runs ORDER BY id",
            true,
        )?,
        sense_pos: query_strings(
            connection,
            "SELECT to_json(sense_pos)::VARCHAR FROM sense_pos ORDER BY word_id, sense_order, pos_code",
            true,
        )?,
        sense_categories: query_strings(
            connection,
            "SELECT to_json(sense_categories)::VARCHAR FROM sense_categories ORDER BY word_id, sense_order, category_slug",
            true,
        )?,
        columns: query_strings(
            connection,
            "SELECT to_json(struct_pack(name := column_name, type := data_type, nullable := is_nullable, position := ordinal_position))::VARCHAR FROM information_schema.columns WHERE table_catalog = current_database() AND table_name = 'words' ORDER BY ordinal_position",
            false,
        )?,
        constraints: query_strings(
            connection,
            "SELECT to_json(struct_pack(kind := constraint_type, definition := constraint_text, columns := constraint_column_names))::VARCHAR FROM duckdb_constraints() WHERE database_name = current_database() AND table_name = 'words' ORDER BY constraint_type, constraint_text",
            false,
        )?,
        indexes: query_strings(
            connection,
            "SELECT to_json(struct_pack(name := index_name, unique_index := is_unique, primary_index := is_primary, expressions := expressions))::VARCHAR FROM duckdb_indexes() WHERE database_name = current_database() AND table_name = 'words' ORDER BY index_name",
            false,
        )?,
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
