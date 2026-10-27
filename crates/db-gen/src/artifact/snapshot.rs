use duckdb_neo::{Parameters, connection::Connection};
use serde::{Deserialize, Serialize};

use super::ArtifactError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalSnapshot {
    pub tables: Vec<String>,
    pub rows: Vec<String>,
    pub columns: Vec<String>,
    pub constraints: Vec<String>,
    pub indexes: Vec<String>,
}

pub fn read_snapshot(connection: &Connection) -> Result<LogicalSnapshot, ArtifactError> {
    Ok(LogicalSnapshot {
        tables: query_strings(
            connection,
            "SELECT table_name FROM duckdb_tables() WHERE database_name = current_database() AND NOT internal ORDER BY table_name",
            false,
        )?,
        rows: query_strings(
            connection,
            "SELECT to_json(words)::VARCHAR FROM words ORDER BY id",
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
