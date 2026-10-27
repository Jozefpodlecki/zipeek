use std::{collections::BTreeSet, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{PreparedGeneration, import::HskImport};

use super::{ArtifactError, LogicalSnapshot, files};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChecksum {
    pub path: String,
    pub size_bytes: usize,
    pub sha256: String,
}

impl FileChecksum {
    fn new(path: String, bytes: &[u8]) -> Self {
        Self {
            path,
            size_bytes: bytes.len(),
            sha256: files::hash(bytes),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactReport {
    pub generator_version: String,
    pub generator_manifest_sha256: String,
    pub engine: String,
    pub storage_compatibility: String,
    pub wasm_package: String,
    pub reproducibility: String,
    pub source_entries: usize,
    pub rejected_entries: usize,
    pub rows: usize,
    pub diagnostics: usize,
    pub logical_sha256: String,
    pub diagnostics_sha256: String,
    pub tables: Vec<String>,
    pub columns: Vec<String>,
    pub constraints: Vec<String>,
    pub indexes: Vec<String>,
    pub inputs: Vec<FileChecksum>,
    pub files: Vec<FileChecksum>,
}

pub(super) fn write_report(
    directory: &Path,
    prepared: &PreparedGeneration,
    imported: &HskImport,
    snapshot: &LogicalSnapshot,
) -> Result<ArtifactReport, ArtifactError> {
    let mut checksums = Vec::new();
    let database_bytes = files::read(&directory.join("hanzi.duckdb"))?;
    checksums.push(FileChecksum::new(
        "hanzi.duckdb".to_owned(),
        &database_bytes,
    ));
    let diagnostics = serde_json::to_vec_pretty(&imported.diagnostics)?;
    for (name, bytes) in [
        ("hanzi.diagnostics.json", diagnostics.as_slice()),
        ("INPUTS.toml", prepared.inputs().manifest_bytes()),
        (
            "ATTRIBUTION.md",
            include_bytes!("ATTRIBUTION.md").as_slice(),
        ),
    ] {
        files::write(&directory.join(name), bytes)?;
        checksums.push(FileChecksum::new(name.to_owned(), bytes));
    }
    let notices = directory.join("notices");
    fs::create_dir(&notices)
        .map_err(|source| files::io("create notices directory", &notices, source))?;
    let mut seen = BTreeSet::new();
    for notice in prepared.inputs().notices() {
        let name = notice
            .path()
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| ArtifactError::InvalidNotice {
                name: notice.path().display().to_string(),
            })?;
        if !seen.insert(name.to_owned()) {
            return Err(ArtifactError::InvalidNotice {
                name: name.to_owned(),
            });
        }
        files::write(&notices.join(name), notice.bytes())?;
        checksums.push(FileChecksum::new(format!("notices/{name}"), notice.bytes()));
    }
    let report = ArtifactReport {
        generator_version: env!("CARGO_PKG_VERSION").to_owned(),
        generator_manifest_sha256: files::hash(include_bytes!("../../Cargo.toml")),
        engine: prepared.database().engine_version().to_owned(),
        storage_compatibility: prepared.database().storage_version().to_owned(),
        wasm_package: crate::policy::WASM_PACKAGE.to_owned(),
        reproducibility: crate::policy::REPRODUCIBILITY.to_owned(),
        source_entries: imported.source_entries,
        rejected_entries: imported.rejected_entries,
        rows: imported.words.len(),
        diagnostics: imported.diagnostics.len(),
        logical_sha256: files::hash(&serde_json::to_vec(snapshot)?),
        diagnostics_sha256: files::hash(&diagnostics),
        tables: snapshot.tables.clone(),
        columns: snapshot.columns.clone(),
        constraints: snapshot.constraints.clone(),
        indexes: snapshot.indexes.clone(),
        inputs: prepared
            .inputs()
            .sources()
            .iter()
            .map(|source| FileChecksum::new(source.name().to_owned(), source.file().bytes()))
            .collect(),
        files: checksums,
    };
    files::write(
        &directory.join("hanzi.manifest.json"),
        &serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}
