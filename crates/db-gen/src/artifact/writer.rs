use std::{fs, path::Path};

use duckdb_neo::{Parameters, connection::Connection};

use crate::{
    database::GeneratorDatabase, import::insert_words, import_primary, prepare,
    schema::create_schema,
};

use super::{
    ArtifactError, ArtifactReport, LogicalSnapshot, files, read_snapshot, report::write_report,
};

pub fn generate(manifest: &Path, output: &Path) -> Result<ArtifactReport, ArtifactError> {
    let (stage, output) = files::staging(output)?;
    let result = build(manifest, &stage).and_then(|report| {
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

fn build(manifest: &Path, directory: &Path) -> Result<ArtifactReport, ArtifactError> {
    let prepared = prepare(manifest)?;
    let imported = import_primary(&prepared)?;
    let expected = read_snapshot(prepared.database().connection())?;
    let file = directory.join("hanzi.duckdb");
    let connection = prepared.database().connection();
    execute(
        connection,
        &format!(
            "ATTACH {} AS artifact (STORAGE_VERSION 'v1.4.0')",
            sql_path(&file)?
        ),
    )?;
    execute(connection, "USE artifact")?;
    create_schema(connection)?;
    insert_words(connection, &imported.words)?;
    if read_snapshot(connection)? != expected {
        return Err(ArtifactError::LogicalMismatch { stage: "writing" });
    }
    execute(connection, "CHECKPOINT")?;
    execute(connection, "USE memory")?;
    execute(connection, "DETACH artifact")?;
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
    execute(connection, "USE memory")?;
    execute(connection, "DETACH artifact")?;
    Ok(snapshot)
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
