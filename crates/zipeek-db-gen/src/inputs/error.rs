use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Could not read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Invalid input manifest {path}: {source}")]
    Manifest {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("Input manifest {path} has no sources")]
    EmptyManifest { path: PathBuf },
    #[error("Manifest policy {field} mismatch: expected {expected}, got {actual}")]
    PolicyMismatch {
        field: &'static str,
        expected: &'static str,
        actual: String,
    },
    #[error("Invalid relative input path {path}: {reason}")]
    InvalidPath { path: PathBuf, reason: &'static str },
    #[error("Input manifest repeats file {path}")]
    DuplicateFile { path: PathBuf },
    #[error("Input manifest repeats source name {name}")]
    DuplicateSource { name: String },
    #[error("Source {name} references undeclared notice {path}")]
    UndeclaredNotice { name: String, path: PathBuf },
    #[error("Invalid SHA-256 for {path}: expected 64 hexadecimal characters, got {digest}")]
    InvalidDigest { path: PathBuf, digest: String },
    #[error("Input size mismatch for {path}: expected {expected} bytes, got {actual}")]
    SizeMismatch {
        path: PathBuf,
        expected: u64,
        actual: u64,
    },
    #[error("SHA-256 mismatch for {path}: expected {expected}, got {actual}; refusing generation")]
    ChecksumMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
}
