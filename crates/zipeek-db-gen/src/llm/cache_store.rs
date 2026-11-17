use std::{
    collections::BTreeSet,
    fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use super::classification::{CachedClassification, ClassificationError};

pub fn records_directory(cache: &Path) -> PathBuf {
    cache.with_extension("jsonl.records")
}

pub fn read_checkpoints(cache: &Path) -> Result<Vec<CachedClassification>, ClassificationError> {
    let directory = records_directory(cache);
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            return Err(ClassificationError::Invalid(format!(
                "unexpected classification checkpoint file: {}",
                path.display()
            )));
        }
        if !entry.file_type()?.is_file() {
            return Err(ClassificationError::Invalid(format!(
                "classification checkpoint is not a regular file: {}",
                path.display()
            )));
        }
        let record: CachedClassification = serde_json::from_slice(&fs::read(&path)?)?;
        let expected = format!("{}-{}.json", record.word_id, record.sense_order);
        if path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
            return Err(ClassificationError::Invalid(format!(
                "classification checkpoint filename does not match record: {}",
                path.display()
            )));
        }
        if !seen.insert((record.word_id, record.sense_order)) {
            return Err(ClassificationError::Invalid(format!(
                "duplicate classification checkpoint {}:{}",
                record.word_id, record.sense_order
            )));
        }
        records.push(record);
    }
    records.sort_by_key(|record| (record.word_id, record.sense_order));
    Ok(records)
}

pub fn checkpoint(cache: &Path, record: &CachedClassification) -> Result<(), ClassificationError> {
    if !cache.exists() {
        if let Some(parent) = cache
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        match OpenOptions::new().create_new(true).write(true).open(cache) {
            Ok(mut file) => file.flush()?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    let directory = records_directory(cache);
    fs::create_dir_all(&directory)?;
    let name = format!("{}-{}.json", record.word_id, record.sense_order);
    let destination = directory.join(&name);
    let temporary = directory.join(format!("{name}.tmp"));
    if destination.exists() || temporary.exists() {
        return Err(ClassificationError::Invalid(format!(
            "classification checkpoint already exists or has an unresolved temporary file: {}",
            name
        )));
    }
    let payload = serde_json::to_vec(record)?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    if let Err(error) = output.write_all(&payload).and_then(|()| output.sync_all()) {
        drop(output);
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    drop(output);
    fs::rename(&temporary, &destination)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::classification::{ClassificationLabels, PROMPT_VERSION, TAXONOMY_VERSION};
    use super::*;

    fn fixture() -> CachedClassification {
        CachedClassification {
            word_id: 5,
            sense_order: 1,
            input_digest: "digest".into(),
            model: "local".into(),
            prompt_version: PROMPT_VERSION.into(),
            taxonomy_version: TAXONOMY_VERSION,
            labels: ClassificationLabels {
                pos: vec!["noun".into()],
                categories: vec![],
            },
        }
    }

    #[test]
    fn atomic_record_checkpoint_is_readable_and_never_overwritten() {
        let root = std::env::temp_dir().join(format!(
            "zipeek-record-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cache = root.join("cache.jsonl");
        let record = fixture();
        checkpoint(&cache, &record).unwrap();
        assert!(cache.exists());
        assert_eq!(read_checkpoints(&cache).unwrap(), vec![record.clone()]);
        assert!(checkpoint(&cache, &record).is_err());
        assert_eq!(read_checkpoints(&cache).unwrap(), vec![record]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_jsonl_and_new_checkpoints_are_loaded_together() {
        let root = std::env::temp_dir().join(format!(
            "zipeek-legacy-record-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cache = root.join("cache.jsonl");
        fs::create_dir_all(&root).unwrap();
        let legacy = fixture();
        super::super::classification::write_cache(&cache, &[legacy.clone()]).unwrap();
        let mut additional = fixture();
        additional.sense_order = 2;
        checkpoint(&cache, &additional).unwrap();
        let records = super::super::classification::read_cache(&cache).unwrap();
        assert_eq!(records.len(), 2);
        assert!(records.contains(&legacy));
        assert!(records.contains(&additional));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_checkpoint_name_and_temporary_files_fail_closed() {
        let root = std::env::temp_dir().join(format!(
            "zipeek-invalid-record-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cache = root.join("cache.jsonl");
        let directory = records_directory(&cache);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("bad.json"),
            serde_json::to_vec(&fixture()).unwrap(),
        )
        .unwrap();
        assert!(read_checkpoints(&cache).is_err());
        fs::remove_file(directory.join("bad.json")).unwrap();
        fs::write(directory.join("5-1.json.tmp"), b"partial").unwrap();
        assert!(read_checkpoints(&cache).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
