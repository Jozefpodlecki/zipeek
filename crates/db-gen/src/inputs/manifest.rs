use std::{collections::BTreeSet, fs, path::Path};

use serde::Deserialize;

use super::InputError;
use crate::policy::{ENGINE_VERSION, REPRODUCIBILITY, STORAGE_VERSION, WASM_PACKAGE};

#[derive(Debug, Deserialize)]
pub(super) struct InputManifest {
    #[serde(skip)]
    pub raw: Vec<u8>,
    duckdb_engine: String,
    storage_compatibility_version: String,
    duckdb_wasm_package: String,
    reproducibility: String,
    pub inputs: Vec<Source>,
    pub notices: Vec<PinnedFile>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Source {
    pub name: String,
    #[serde(flatten)]
    pub file: PinnedFile,
    pub size_bytes: u64,
    pub notices: Vec<std::path::PathBuf>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PinnedFile {
    pub path: std::path::PathBuf,
    pub sha256: String,
}

impl InputManifest {
    pub fn load(path: &Path) -> Result<Self, InputError> {
        let text = fs::read_to_string(path).map_err(|source| InputError::Read {
            path: path.to_owned(),
            source,
        })?;
        let mut manifest: Self = toml::from_str(&text).map_err(|source| InputError::Manifest {
            path: path.to_owned(),
            source,
        })?;
        manifest.raw = text.into_bytes();
        manifest.validate(path)?;
        Ok(manifest)
    }

    fn validate(&self, path: &Path) -> Result<(), InputError> {
        if self.inputs.is_empty() {
            return Err(InputError::EmptyManifest {
                path: path.to_owned(),
            });
        }
        for (field, expected, actual) in [
            ("duckdb_engine", ENGINE_VERSION, &self.duckdb_engine),
            (
                "storage_compatibility_version",
                STORAGE_VERSION,
                &self.storage_compatibility_version,
            ),
            (
                "duckdb_wasm_package",
                WASM_PACKAGE,
                &self.duckdb_wasm_package,
            ),
            ("reproducibility", REPRODUCIBILITY, &self.reproducibility),
        ] {
            if actual != expected {
                return Err(InputError::PolicyMismatch {
                    field,
                    expected,
                    actual: actual.clone(),
                });
            }
        }
        let notices: BTreeSet<_> = self.notices.iter().map(|notice| &notice.path).collect();
        let mut names = BTreeSet::new();
        for input in &self.inputs {
            if !names.insert(&input.name) {
                return Err(InputError::DuplicateSource {
                    name: input.name.clone(),
                });
            }
            for notice in &input.notices {
                if !notices.contains(notice) {
                    return Err(InputError::UndeclaredNotice {
                        name: input.name.clone(),
                        path: notice.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}
