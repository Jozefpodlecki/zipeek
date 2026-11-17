use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct DictionaryManifest {
    pub engine: String,
    pub storage_compatibility: String,
    pub files: Vec<DictionaryAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DictionaryAsset {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct DictionarySelection<'a> {
    pub filename: &'a str,
    pub original: &'a DictionaryAsset,
    pub download: &'a DictionaryAsset,
    pub compressed: bool,
}

#[derive(Debug, Error)]
pub enum DictionaryManifestError {
    #[error("Dictionary manifest must identify exactly one database file")]
    AmbiguousDatabase,
    #[error("Dictionary manifest has duplicate asset entries: {0}")]
    DuplicateAsset(String),
    #[error("Dictionary asset has an invalid SHA-256 checksum: {0}")]
    InvalidDigest(String),
    #[error("Dictionary asset has an invalid size: {0}")]
    InvalidSize(String),
    #[error("Dictionary manifest has invalid version metadata: {0}")]
    InvalidVersion(&'static str),
}

impl DictionaryManifest {
    pub fn select(&self) -> Result<DictionarySelection<'_>, DictionaryManifestError> {
        for (name, version) in [
            ("engine", &self.engine),
            ("storage_compatibility", &self.storage_compatibility),
        ] {
            let valid = version.strip_prefix('v').is_some_and(|version| {
                version.split('.').count() >= 2
                    && version.split('.').all(|part| {
                        !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
                    })
            });
            if !valid {
                return Err(DictionaryManifestError::InvalidVersion(name));
            }
        }
        let candidates: Vec<&DictionaryAsset> = self
            .files
            .iter()
            .filter(|entry| valid_database_name(&entry.path))
            .collect();
        let [original] = candidates.as_slice() else {
            return Err(DictionaryManifestError::AmbiguousDatabase);
        };
        let original = *original;
        let compressed_name = format!("{}.gz", original.path);
        for file in &self.files {
            if file.size_bytes == 0
                || (file.path == original.path && file.size_bytes > 256 * 1024 * 1024)
                || (file.path == compressed_name && file.size_bytes > 128 * 1024 * 1024)
            {
                return Err(DictionaryManifestError::InvalidSize(file.path.clone()));
            }
            if file.sha256.len() != 64 || !file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(DictionaryManifestError::InvalidDigest(file.path.clone()));
            }
            if self
                .files
                .iter()
                .filter(|other| other.path == file.path)
                .count()
                != 1
            {
                return Err(DictionaryManifestError::DuplicateAsset(file.path.clone()));
            }
        }
        let compressed = self
            .files
            .iter()
            .find(|entry| entry.path == compressed_name);
        Ok(DictionarySelection {
            filename: &original.path,
            original,
            download: compressed.unwrap_or(original),
            compressed: compressed.is_some(),
        })
    }
}

fn valid_database_name(name: &str) -> bool {
    if name == "hanzi.duckdb" {
        return true;
    }
    let Some(timestamp) = name
        .strip_prefix("hanzi-")
        .and_then(|name| name.strip_suffix(".duckdb"))
    else {
        return false;
    };
    timestamp.len() == 15
        && timestamp.as_bytes()[8] == b'-'
        && timestamp
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 8 || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(path: &str) -> DictionaryAsset {
        DictionaryAsset {
            path: path.to_owned(),
            size_bytes: 12,
            sha256: "a".repeat(64),
        }
    }

    #[test]
    fn reports_generator_versions_when_manifest_provides_them() {
        let manifest: DictionaryManifest = serde_json::from_str(
            r#"{"engine":"v2.0.0","storage_compatibility":"v2.0.0","files":[]}"#,
        )
        .unwrap();
        assert_eq!(manifest.engine, "v2.0.0");
        assert_eq!(manifest.storage_compatibility, "v2.0.0");
        for invalid in [
            r#"{"files":[]}"#,
            r#"{"engine":"v2.0.0","files":[]}"#,
            r#"{"storage_compatibility":"v2.0.0","files":[]}"#,
        ] {
            assert!(serde_json::from_str::<DictionaryManifest>(invalid).is_err());
        }
    }

    #[test]
    fn refuses_invalid_generator_version_metadata() {
        let mut manifest = DictionaryManifest {
            engine: String::new(),
            storage_compatibility: "v2.0.0".into(),
            files: vec![asset("hanzi.duckdb")],
        };
        assert!(matches!(
            manifest.select(),
            Err(DictionaryManifestError::InvalidVersion("engine"))
        ));
        manifest.engine = "v2.0.0".into();
        manifest.storage_compatibility = "latest".into();
        assert!(matches!(
            manifest.select(),
            Err(DictionaryManifestError::InvalidVersion(
                "storage_compatibility"
            ))
        ));
    }

    #[test]
    fn oversized_advertised_database_is_rejected_before_download() {
        let mut original = asset("hanzi.duckdb");
        original.size_bytes = 257 * 1024 * 1024;
        let manifest = DictionaryManifest {
            engine: "v2.0.0".into(),
            storage_compatibility: "v2.0.0".into(),
            files: vec![original],
        };
        assert!(matches!(
            manifest.select(),
            Err(DictionaryManifestError::InvalidSize(_))
        ));
    }

    #[test]
    fn accepts_only_expected_database_filenames() {
        assert!(valid_database_name("hanzi.duckdb"));
        assert!(valid_database_name("hanzi-20261107-210000.duckdb"));
        assert!(!valid_database_name("../../hanzi.duckdb"));
        assert!(!valid_database_name("hanzi-20261107-21000X.duckdb"));
        assert!(!valid_database_name("hanzi-20261107-210000.duckdb.gz"));
    }

    #[test]
    fn prefers_compressed_asset_but_requires_original_metadata() {
        let manifest = DictionaryManifest {
            engine: "v2.0.0".into(),
            storage_compatibility: "v2.0.0".into(),
            files: vec![asset("hanzi.duckdb"), asset("hanzi.duckdb.gz")],
        };
        let selected = manifest.select().unwrap();
        assert!(selected.compressed);
        assert_eq!(selected.filename, "hanzi.duckdb");
        assert_eq!(selected.download.path, "hanzi.duckdb.gz");
    }

    #[test]
    fn refuses_ambiguous_and_duplicate_manifest_entries() {
        let manifest = DictionaryManifest {
            engine: "v2.0.0".into(),
            storage_compatibility: "v2.0.0".into(),
            files: vec![],
        };
        assert!(matches!(
            manifest.select(),
            Err(DictionaryManifestError::AmbiguousDatabase)
        ));
        let manifest = DictionaryManifest {
            engine: "v2.0.0".into(),
            storage_compatibility: "v2.0.0".into(),
            files: vec![asset("hanzi.duckdb"), asset("hanzi.duckdb")],
        };
        assert!(matches!(
            manifest.select(),
            Err(DictionaryManifestError::AmbiguousDatabase)
        ));
        let manifest = DictionaryManifest {
            engine: "v2.0.0".into(),
            storage_compatibility: "v2.0.0".into(),
            files: vec![
                asset("hanzi.duckdb"),
                asset("hanzi.duckdb.gz"),
                asset("hanzi.duckdb.gz"),
            ],
        };
        assert!(matches!(
            manifest.select(),
            Err(DictionaryManifestError::DuplicateAsset(_))
        ));
    }
}
