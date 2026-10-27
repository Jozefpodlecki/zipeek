use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::{InputError, manifest::InputManifest, manifest::PinnedFile};

#[derive(Debug)]
pub struct VerifiedFile {
    path: PathBuf,
    bytes: Vec<u8>,
}

impl VerifiedFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Debug)]
pub struct VerifiedSource {
    name: String,
    file: VerifiedFile,
}

impl VerifiedSource {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn file(&self) -> &VerifiedFile {
        &self.file
    }
}

#[derive(Debug)]
pub struct VerifiedInputs {
    manifest: Vec<u8>,
    sources: Vec<VerifiedSource>,
    notices: Vec<VerifiedFile>,
}

impl VerifiedInputs {
    pub fn manifest_bytes(&self) -> &[u8] {
        &self.manifest
    }
    pub fn sources(&self) -> &[VerifiedSource] {
        &self.sources
    }

    pub fn notices(&self) -> &[VerifiedFile] {
        &self.notices
    }

    pub fn total_bytes(&self) -> usize {
        self.sources
            .iter()
            .map(|source| source.file.bytes.len())
            .chain(self.notices.iter().map(|notice| notice.bytes.len()))
            .sum()
    }
}

pub fn verify(manifest_path: &Path) -> Result<VerifiedInputs, InputError> {
    let manifest = InputManifest::load(manifest_path)?;
    let root = manifest_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(|source| InputError::Read {
            path: manifest_path.to_owned(),
            source,
        })?;
    let mut seen = BTreeSet::new();
    let mut sources = Vec::with_capacity(manifest.inputs.len());
    for source in manifest.inputs {
        let file = verify_file(&root, &source.file, Some(source.size_bytes), &mut seen)?;
        tracing::info!(source = %source.name, path = %source.file.path.display(), "Source checksum verified");
        sources.push(VerifiedSource {
            name: source.name,
            file,
        });
    }
    let mut notices = Vec::with_capacity(manifest.notices.len());
    for notice in manifest.notices {
        let file = verify_file(&root, &notice, None, &mut seen)?;
        tracing::info!(path = %notice.path.display(), "Source notice checksum verified");
        notices.push(file);
    }
    Ok(VerifiedInputs {
        manifest: manifest.raw,
        sources,
        notices,
    })
}

fn verify_file(
    root: &Path,
    pinned: &PinnedFile,
    expected_size: Option<u64>,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<VerifiedFile, InputError> {
    if pinned.path.as_os_str().is_empty()
        || !pinned
            .path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(InputError::InvalidPath {
            path: pinned.path.clone(),
            reason: "expected a nonempty relative path without parent traversal",
        });
    }
    if pinned.sha256.len() != 64 || !pinned.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(InputError::InvalidDigest {
            path: pinned.path.clone(),
            digest: pinned.sha256.clone(),
        });
    }
    let requested = root.join(&pinned.path);
    let path = requested
        .canonicalize()
        .map_err(|source| InputError::Read {
            path: requested,
            source,
        })?;
    if !path.starts_with(root) {
        return Err(InputError::InvalidPath {
            path: pinned.path.clone(),
            reason: "resolved file is outside the input directory",
        });
    }
    if !seen.insert(path.clone()) {
        return Err(InputError::DuplicateFile { path });
    }
    let bytes = fs::read(&path).map_err(|source| InputError::Read {
        path: path.clone(),
        source,
    })?;
    if let Some(expected) = expected_size {
        let actual = bytes.len() as u64;
        if actual != expected {
            return Err(InputError::SizeMismatch {
                path,
                expected,
                actual,
            });
        }
    }
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if !actual.eq_ignore_ascii_case(&pinned.sha256) {
        return Err(InputError::ChecksumMismatch {
            path,
            expected: pinned.sha256.clone(),
            actual,
        });
    }
    Ok(VerifiedFile { path, bytes })
}
