use std::{
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use super::ArtifactError;
use sha2::{Digest, Sha256};

static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);

pub(super) fn hash(bytes: &[u8]) -> String {
    crate::sha256_hex(bytes)
}

pub(super) fn write(path: &Path, bytes: &[u8]) -> Result<(), ArtifactError> {
    fs::write(path, bytes).map_err(|source| io("write", path, source))
}

pub(super) fn hash_file(path: &Path) -> Result<(usize, String), ArtifactError> {
    let file = fs::File::open(path).map_err(|source| io("open for checksum", path, source))?;
    let mut reader = BufReader::with_capacity(256 * 1024, file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut length = 0_usize;
    loop {
        let read = reader.read(&mut buffer)
            .map_err(|source| io("read for checksum", path, source))?;
        if read == 0 {
            break;
        }
        length = length.checked_add(read).ok_or_else(|| ArtifactError::InvalidPath {
            path: path.to_owned(),
        })?;
        digest.update(&buffer[..read]);
    }
    Ok((length, hex::encode(digest.finalize())))
}

pub(super) fn io(operation: &'static str, path: &Path, source: std::io::Error) -> ArtifactError {
    ArtifactError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

pub(super) struct StagedArtifact {
    stage: PathBuf,
    output: PathBuf,
    published: bool,
}

impl StagedArtifact {
    pub(super) fn create(output: &Path) -> Result<Self, ArtifactError> {
        let (stage, output) = staging(output)?;
        Ok(Self { stage, output, published: false })
    }

    pub(super) fn path(&self) -> &Path {
        &self.stage
    }

    pub(super) fn publish(&mut self) -> Result<(), ArtifactError> {
        if self.output.exists() {
            return Err(ArtifactError::ExistingOutput {
                path: self.output.clone(),
            });
        }
        fs::rename(&self.stage, &self.output)
            .map_err(|source| io("publish artifact directory", &self.output, source))?;
        self.published = true;
        Ok(())
    }
}

impl Drop for StagedArtifact {
    fn drop(&mut self) {
        if !self.published {
            cleanup(&self.stage);
        }
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
    if let Err(error) = fs::remove_dir_all(stage)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(%error, path = %stage.display(), "Could not remove artifact staging directory");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn output_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        Ok(std::env::temp_dir().join(format!("zipeek-stage-{}-{nonce}", std::process::id())))
    }

    #[test]
    fn staging_is_removed_when_unpublished() -> Result<(), Box<dyn std::error::Error>> {
        let output = output_path()?;
        let path = {
            let stage = StagedArtifact::create(&output)?;
            let path = stage.path().to_path_buf();
            write(&path.join("incomplete.duckdb"), b"test")?;
            path
        };
        assert!(!path.exists());
        assert!(!output.exists());
        Ok(())
    }

    #[test]
    fn streamed_checksum_matches_in_memory_digest() -> Result<(), Box<dyn std::error::Error>> {
        let output = output_path()?;
        let contents = vec![b'a'; 170_000];
        write(&output, &contents)?;
        let result = hash_file(&output)?;
        fs::remove_file(&output)?;
        assert_eq!(result, (contents.len(), hash(&contents)));
        Ok(())
    }

    #[test]
    fn publishing_moves_complete_stage() -> Result<(), Box<dyn std::error::Error>> {
        let output = output_path()?;
        let mut stage = StagedArtifact::create(&output)?;
        write(&stage.path().join("hanzi.duckdb"), b"fixture")?;
        let previous = stage.path().to_owned();
        stage.publish()?;
        drop(stage);
        assert!(!previous.exists());
        assert_eq!(fs::read(output.join("hanzi.duckdb"))?, b"fixture");
        fs::remove_dir_all(&output)?;
        Ok(())
    }
}
