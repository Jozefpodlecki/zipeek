use std::{fs, path::Path};

use duckdb_neo::{Parameters, connection::Connection};

use crate::{
    database::GeneratorDatabase,
    import_primary,
    llm::classification::{apply_cache, read_cache},
    prepare,
    schema::create_schema,
};

use super::{
    ArtifactError, ArtifactReport, LogicalSnapshot, files, read_snapshot, report::write_report,
};

pub fn generate(manifest: &Path, output: &Path) -> Result<ArtifactReport, ArtifactError> {
    generate_with_cache(manifest, output, None)
}

pub fn generate_with_cache(
    manifest: &Path,
    output: &Path,
    cache: Option<&Path>,
) -> Result<ArtifactReport, ArtifactError> {
    let (stage, output) = files::staging(output)?;
    let result = build(manifest, &stage, cache).and_then(|report| {
        if output.exists() {
            return Err(ArtifactError::ExistingOutput {
                path: output.clone(),
            });
        }
        fs::rename(&stage, &output)
            .map_err(|source| files::io("publish artifact directory", &output, source))?;
        Ok(report)
    });
    if result.is_err() {
        files::cleanup(&stage);
    }
    result
}

fn build(
    manifest: &Path,
    directory: &Path,
    cache: Option<&Path>,
) -> Result<ArtifactReport, ArtifactError> {
    tracing::info!("Verifying inputs and preparing in-memory dictionary");
    let prepared = prepare(manifest)?;
    let imported = import_primary(&prepared)?;
    tracing::info!(
        words = imported.words.len(),
        senses = imported.senses.len(),
        readings = imported.character_readings.len(),
        "In-memory dictionary import complete"
    );
    let cached = cache.map(read_cache).transpose()?;
    if let Some(ref records) = cached {
        apply_cache(prepared.database().connection(), &imported.senses, records)?;
    }
    tracing::info!("Capturing in-memory dictionary snapshot");
    let expected = read_snapshot(prepared.database().connection())?;
    let file = directory.join("hanzi.duckdb");
    let connection = prepared.database().connection();
    execute(
        connection,
        &format!(
            "ATTACH {} AS artifact (STORAGE_VERSION 'v2.0.0', BLOCK_SIZE 262144)",
            sql_path(&file)?
        ),
    )?;
    execute(connection, "USE artifact")?;
    create_schema(connection)?;
    tracing::info!("Bulk copying dictionary tables to artifact");
    copy_dictionary_tables(connection)?;
    tracing::info!("Comparing artifact snapshot with in-memory dictionary");
    if read_snapshot(connection)? != expected {
        return Err(ArtifactError::LogicalMismatch { stage: "writing" });
    }
    create_english_fts(connection)?;
    tracing::info!("Checkpointing artifact database");
    execute(connection, "CHECKPOINT")?;
    execute(connection, "USE memory")?;
    execute(connection, "DETACH artifact")?;
    tracing::info!("Verifying artifact after read-only reopening");
    if verify_database(&file)? != expected {
        return Err(ArtifactError::LogicalMismatch {
            stage: "read-only reopening",
        });
    }
    write_report(directory, &prepared, &imported, &expected)
}

pub fn verify_database(path: &Path) -> Result<LogicalSnapshot, ArtifactError> {
    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    execute(
        connection,
        &format!("ATTACH {} AS artifact (READ_ONLY)", sql_path(path)?),
    )?;
    execute(connection, "USE artifact")?;
    let snapshot = read_snapshot(connection)?;
    execute(connection, "LOAD fts")?;
    verify_english_fts(connection)?;
    execute(connection, "USE memory")?;
    execute(connection, "DETACH artifact")?;
    Ok(snapshot)
}

fn create_english_fts(connection: &Connection) -> Result<(), ArtifactError> {
    execute(connection, "INSTALL fts")?;
    execute(connection, "LOAD fts")?;
    execute(
        connection,
        "PRAGMA create_fts_index('words', 'id', 'english', stemmer = 'english', stopwords = 'none')",
    )?;
    verify_english_fts(connection)
}

fn verify_english_fts(connection: &Connection) -> Result<(), ArtifactError> {
    {
        let mut result = connection
            .query(
                "SELECT count(*) FROM duckdb_schemas() WHERE database_name = current_database() AND schema_name = 'fts_main_words'",
                Parameters::None,
            )
            .map_err(|source| ArtifactError::Query {
                query: "verify English FTS schema".to_owned(),
                source,
            })?;
        let Some(chunk) = result.next_chunk().map_err(|source| ArtifactError::Query {
            query: "read English FTS schema count".to_owned(),
            source,
        })?
        else {
            return Err(ArtifactError::LogicalMismatch {
                stage: "FTS schema verification",
            });
        };
        let vector = chunk
            .get_vector_at::<i64>(0)
            .map_err(|source| ArtifactError::Query {
                query: "read English FTS schema vector".to_owned(),
                source,
            })?;
        let reader = vector.reader().map_err(|source| ArtifactError::Query {
            query: "read English FTS schema count value".to_owned(),
            source,
        })?;
        if reader.iter().next().flatten().copied() != Some(1) {
            return Err(ArtifactError::LogicalMismatch {
                stage: "FTS schema verification",
            });
        }
    }
    let mut matches = connection
        .query(
            "SELECT count(*) FROM (SELECT fts_main_words.match_bm25(id, 'language') AS score FROM words) AS ranked WHERE score IS NOT NULL",
            Parameters::None,
        )
        .map_err(|source| ArtifactError::Query {
            query: "verify nonempty FTS BM25 results".to_owned(),
            source,
        })?;
    let Some(chunk) = matches
        .next_chunk()
        .map_err(|source| ArtifactError::Query {
            query: "execute FTS BM25 result check".to_owned(),
            source,
        })?
    else {
        return Err(ArtifactError::LogicalMismatch {
            stage: "FTS result verification",
        });
    };
    let scores = chunk
        .get_vector_at::<i64>(0)
        .map_err(|source| ArtifactError::Query {
            query: "read FTS result count".to_owned(),
            source,
        })?;
    let values = scores.reader().map_err(|source| ArtifactError::Query {
        query: "read FTS result count value".to_owned(),
        source,
    })?;
    if !values
        .iter()
        .next()
        .flatten()
        .is_some_and(|count| *count > 0)
    {
        return Err(ArtifactError::LogicalMismatch {
            stage: "FTS result verification",
        });
    }
    Ok(())
}

const BULK_COPY_TABLES: &[&str] = &[
    "words",
    "dictionary_versions",
    "characters",
    "character_readings",
    "pinyin_syllable_candidates",
    "vocabulary_identities",
    "word_hsk_membership",
    "word_pos_membership",
    "word_senses",
    "character_gloss_candidates",
    "word_characters",
    "word_relations",
    "sense_display",
    "sense_qualifiers",
    "sense_relations",
    "classification_runs",
    "sense_pos",
    "sense_categories",
];

fn copy_dictionary_tables(connection: &Connection) -> Result<(), ArtifactError> {
    execute(connection, "BEGIN TRANSACTION")?;
    let result = BULK_COPY_TABLES.iter().try_for_each(|table| {
        tracing::info!(table = *table, "Copying dictionary table");
        execute(
            connection,
            &format!("INSERT INTO artifact.{table} SELECT * FROM memory.{table}"),
        )
    });
    match result {
        Ok(()) => {
            if let Err(error) = execute(connection, "COMMIT") {
                let _ = execute(connection, "ROLLBACK");
                return Err(error);
            }
            Ok(())
        }
        Err(error) => {
            let _ = execute(connection, "ROLLBACK");
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_copy_includes_all_non_seed_tables_in_dependency_order() {
        assert_eq!(BULK_COPY_TABLES[0], "words");
        assert!(
            BULK_COPY_TABLES
                .iter()
                .position(|name| *name == "word_senses")
                < BULK_COPY_TABLES
                    .iter()
                    .position(|name| *name == "sense_display")
        );
        assert!(
            BULK_COPY_TABLES
                .iter()
                .position(|name| *name == "characters")
                < BULK_COPY_TABLES
                    .iter()
                    .position(|name| *name == "word_characters")
        );
        let mut seen = std::collections::HashSet::new();
        for table in BULK_COPY_TABLES {
            assert!(seen.insert(table), "table copied twice: {table}");
        }
        let schema = include_str!("../schema.sql");
        for definition in schema.split("CREATE TABLE ").skip(1) {
            let (table, rest) = definition.split_once('(').expect("table definition");
            let table = table.trim();
            let body = rest.split_once(';').expect("table terminator").0;
            if let Some(child_position) = BULK_COPY_TABLES.iter().position(|name| *name == table) {
                for foreign_key in body.split("REFERENCES ").skip(1) {
                    let parent = foreign_key
                        .split(|character: char| {
                            !character.is_ascii_alphanumeric() && character != '_'
                        })
                        .next()
                        .expect("foreign key parent");
                    if let Some(parent_position) =
                        BULK_COPY_TABLES.iter().position(|name| *name == parent)
                    {
                        assert!(
                            parent_position < child_position,
                            "{table} copied before its referenced parent {parent}"
                        );
                    }
                }
            }
        }
        assert!(!BULK_COPY_TABLES.contains(&"parts_of_speech"));
        assert!(!BULK_COPY_TABLES.contains(&"categories"));
    }
}

fn execute(connection: &Connection, query: &str) -> Result<(), ArtifactError> {
    connection
        .execute(query, Parameters::None)
        .map_err(|source| ArtifactError::Query {
            query: query.to_owned(),
            source,
        })?;
    Ok(())
}

fn sql_path(path: &Path) -> Result<String, ArtifactError> {
    let path = path.to_str().ok_or_else(|| ArtifactError::InvalidPath {
        path: path.to_owned(),
    })?;
    let normalized = if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!("//{}", unc.replace('\\', "/"))
    } else {
        path.strip_prefix(r"\\?\")
            .unwrap_or(path)
            .replace('\\', "/")
    };
    Ok(format!("'{}'", normalized.replace('\'', "''")))
}
