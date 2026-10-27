use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use sha2::{Digest, Sha256};

use super::ArtifactError;

static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);

pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn write(path: &Path, bytes: &[u8]) -> Result<(), ArtifactError> {
    fs::write(path, bytes).map_err(|source| io("write", path, source))
}

pub(super) fn read(path: &Path) -> Result<Vec<u8>, ArtifactError> {
    fs::read(path).map_err(|source| io("read", path, source))
}

pub(super) fn io(operation: &'static str, path: &Path, source: std::io::Error) -> ArtifactError {
    ArtifactError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

pub(super) fn staging(output: &Path) -> Result<(PathBuf, PathBuf), ArtifactError> {
    if output.exists() {
        return Err(ArtifactError::ExistingOutput {
            path: output.to_owned(),
        });
    }
    let name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ArtifactError::InvalidPath {
            path: output.to_owned(),
        })?;
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| io("create parent directory", parent, source))?;
    let parent = parent
        .canonicalize()
        .map_err(|source| io("resolve parent", parent, source))?;
    let stage = parent.join(format!(
        ".{name}.building-{}-{}",
        std::process::id(),
        NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&stage).map_err(|source| io("create staging directory", &stage, source))?;
    Ok((stage, parent.join(name)))
}

pub(super) fn cleanup(stage: &Path) {
    remove_files(&stage.join("notices"));
    remove_directory(&stage.join("notices"));
    remove_files(stage);
    remove_directory(stage);
}

fn remove_files(directory: &Path) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            tracing::warn!(%error, path = %directory.display(), "Could not list staged files");
            return;
        }
    };
    for entry in entries {
        match entry {
            Ok(entry) => {
                if let Err(error) = fs::remove_file(entry.path()) {
                    tracing::warn!(%error, path = %entry.path().display(), "Could not remove staged file");
                }
            }
            Err(error) => {
                tracing::warn!(%error, path = %directory.display(), "Could not read staging entry")
            }
        }
    }
}

fn remove_directory(path: &Path) {
    if let Err(error) = fs::remove_dir(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(%error, path = %path.display(), "Could not remove staging directory");
    }
}
