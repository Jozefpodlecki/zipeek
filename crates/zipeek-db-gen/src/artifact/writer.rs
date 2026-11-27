use std::path::Path;

use zipeek_db::native::Connection;

use crate::{
    database::{DatabaseSession, GeneratorDatabase},
    import_primary,
    llm::classification::{apply_cache, read_cache},
    prepare,
    schema::create_schema,
    search,
};

use super::{
    ArtifactError, ArtifactReport, LogicalSnapshot, files, read_snapshot,
    report::ArtifactReportOptions,
    sql::sql_path,
};

pub fn generate(manifest: &Path, output: &Path) -> Result<ArtifactReport, ArtifactError> {
    generate_with_cache(manifest, output, None)
}

pub fn generate_with_cache(
    manifest: &Path,
    output: &Path,
    cache: Option<&Path>,
) -> Result<ArtifactReport, ArtifactError> {
    let mut stage = files::StagedArtifact::create(output)?;
    let report = build(manifest, stage.path(), cache)?;
    stage.publish()?;
    Ok(report)
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
    tracing::info!("Building dictionary search documents and normalized keys");
    search::populate(prepared.database().connection())?;
    tracing::info!("Capturing in-memory dictionary snapshot");
    let expected = read_snapshot(prepared.database().connection())?;
    let file = directory.join("hanzi.duckdb");
    let connection = prepared.database().connection();
    let session = DatabaseSession::new(connection);
    session.execute(
        &format!(
            "ATTACH {} AS artifact (STORAGE_VERSION 'v2.0.0', BLOCK_SIZE 262144)",
            sql_path(&file)?
        ),
    )?;
    session.execute("USE artifact")?;
    create_schema(connection)?;
    tracing::info!("Bulk copying dictionary tables to artifact");
    copy_dictionary_tables(connection)?;
    tracing::info!("Comparing artifact snapshot with in-memory dictionary");
    if read_snapshot(connection)? != expected {
        return Err(ArtifactError::LogicalMismatch { stage: "writing" });
    }
    super::fts::create(connection)?;
    tracing::info!("Checkpointing artifact database");
    session.execute("CHECKPOINT")?;
    session.execute("USE memory")?;
    session.execute("DETACH artifact")?;
    tracing::info!("Verifying artifact after read-only reopening");
    if verify_database(&file)? != expected {
        return Err(ArtifactError::LogicalMismatch {
            stage: "read-only reopening",
        });
    }
    ArtifactReport::new(ArtifactReportOptions {
        directory,
        prepared: &prepared,
        imported: &imported,
        snapshot: &expected,
    })
}

pub fn verify_database(path: &Path) -> Result<LogicalSnapshot, ArtifactError> {
    let database = GeneratorDatabase::open()?;
    let session = DatabaseSession::from_database(&database);
    let connection = session.connection();
    session.execute(&format!(
        "ATTACH {} AS artifact (READ_ONLY)",
        sql_path(path)?
    ))?;
    session.execute("USE artifact")?;
    let snapshot = read_snapshot(connection)?;
    search::verify(connection)?;
    super::fts::verify(connection)?;
    session.execute("USE memory")?;
    session.execute("DETACH artifact")?;
    Ok(snapshot)
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
    "word_search_documents",
    "word_search_keys",
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
    let mut transaction = DatabaseSession::new(connection)
        .transaction()
        .map_err(|source| ArtifactError::Query {
            query: "BEGIN TRANSACTION".to_owned(),
            source,
        })?;
    let result = BULK_COPY_TABLES.iter().try_for_each(|table| {
        tracing::info!(table = *table, "Copying dictionary table");
        transaction
            .execute(&format!(
                "INSERT INTO artifact.{table} SELECT * FROM memory.{table}"
            ))
            .map_err(ArtifactError::from)
    });
    match result {
        Ok(()) => match transaction.commit() {
            Ok(()) => Ok(()),
            Err(source) => rollback_copy(
                &mut transaction,
                ArtifactError::Query {
                    query: "COMMIT".to_owned(),
                    source,
                },
            ),
        },
        Err(original) => rollback_copy(&mut transaction, original),
    }
}

fn rollback_copy(
    transaction: &mut crate::database::Transaction<'_>,
    original: ArtifactError,
) -> Result<(), ArtifactError> {
    match transaction.rollback() {
        Ok(()) => Err(original),
        Err(rollback) => Err(ArtifactError::Rollback {
            original: Box::new(original),
            rollback,
        }),
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
