use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use super::classification::{self, ClassificationError};
use crate::import::SourceSense;

#[derive(Debug, Clone, Serialize)]
pub struct DeferredSense {
    pub word_id: i32,
    pub sense_order: i32,
    pub source: String,
    pub gloss: String,
    pub model: String,
    pub prompt_version: String,
    pub attempts: usize,
    pub error: String,
    pub timestamp_unix_seconds: u64,
}

impl DeferredSense {
    pub fn new(sense: &SourceSense, model: &str, error: &ClassificationError) -> Self {
        Self {
            word_id: sense.word_id.as_i32(),
            sense_order: sense.sense_order,
            source: sense.source.to_owned(),
            gloss: sense.gloss.clone(),
            model: model.to_owned(),
            prompt_version: classification::PROMPT_VERSION.to_owned(),
            attempts: super::recovery::MAX_CLASSIFICATION_ATTEMPTS,
            error: error.to_string(),
            timestamp_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

pub fn path_for(cache: &Path) -> PathBuf {
    cache.with_extension("failures.jsonl")
}

pub fn write(cache: &Path, failures: &[DeferredSense]) -> Result<(), ClassificationError> {
    let report_path = path_for(cache);
    if failures.is_empty() {
        match fs::remove_file(&report_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        return Ok(());
    }
    if let Some(parent) = report_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let mut output = String::new();
    for failure in failures {
        output.push_str(&serde_json::to_string(failure)?);
        output.push('\n');
    }
    let temporary = report_path.with_extension("failures.jsonl.tmp");
    fs::write(&temporary, output)?;
    let backup = report_path.with_extension("failures.jsonl.bak");
    if backup.exists() {
        return Err(ClassificationError::Invalid(format!(
            "failure report backup already exists: {}",
            backup.display()
        )));
    }
    if !report_path.exists() {
        fs::rename(temporary, report_path)?;
        return Ok(());
    }
    if let Err(error) = fs::rename(&report_path, &backup) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    if let Err(error) = fs::rename(&temporary, &report_path) {
        let _ = fs::rename(&backup, &report_path);
        return Err(error.into());
    }
    fs::remove_file(backup)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_path_is_adjacent_to_model_cache() {
        assert_eq!(
            path_for(Path::new("target/a.jsonl")),
            PathBuf::from("target/a.failures.jsonl")
        );
    }
}
