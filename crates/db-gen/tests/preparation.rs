use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use db_gen::{GeneratorError, inputs::InputError, prepare};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn copy() -> Result<Self, Box<dyn Error>> {
        let directory = std::env::temp_dir().join(format!(
            "zipeek-input-test-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let fixture = Self { directory };
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/inputs");
        for file in ["MANIFEST.toml", "sample.json", "NOTICE.txt"] {
            fs::copy(source.join(file), fixture.directory.join(file))?;
        }
        Ok(fixture)
    }

    fn manifest(&self) -> PathBuf {
        self.directory.join("MANIFEST.toml")
    }

    fn replace(&self, from: &str, to: &str) -> Result<(), Box<dyn Error>> {
        let text = fs::read_to_string(self.manifest())?;
        assert!(text.contains(from), "fixture replacement must match");
        fs::write(self.manifest(), text.replace(from, to))?;
        Ok(())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for file in ["MANIFEST.toml", "sample.json", "NOTICE.txt"] {
            if let Err(error) = fs::remove_file(self.directory.join(file))
                && error.kind() != std::io::ErrorKind::NotFound
            {
                eprintln!("Could not clean fixture file {file}: {error}");
            }
        }
        if let Err(error) = fs::remove_dir(&self.directory) {
            eprintln!(
                "Could not clean fixture {}: {error}",
                self.directory.display()
            );
        }
    }
}

#[test]
fn verified_sources_prepare_the_required_database_connection() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    let prepared = prepare(&fixture.manifest())?;
    assert_eq!(prepared.database().engine_version(), "v2.0.0");
    assert_eq!(prepared.database().storage_version(), "v1.4.0");
    assert_eq!(prepared.inputs().sources().len(), 1);
    assert_eq!(prepared.inputs().sources()[0].name(), "fixture-source");
    assert_eq!(
        prepared.inputs().sources()[0].file().bytes(),
        fs::read(fixture.directory.join("sample.json"))?
    );
    assert_eq!(prepared.inputs().notices().len(), 1);
    assert_eq!(
        prepared.inputs().total_bytes(),
        prepared.inputs().sources()[0].file().bytes().len()
            + prepared.inputs().notices()[0].bytes().len()
    );
    Ok(())
}

#[test]
fn modified_source_bytes_are_rejected_with_the_file_and_hashes() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    let path = fixture.directory.join("sample.json");
    let mut bytes = fs::read(&path)?;
    bytes[0] = b'[';
    fs::write(&path, bytes)?;
    let error = prepare(&fixture.manifest()).expect_err("tampered input must fail");
    let GeneratorError::Inputs(InputError::ChecksumMismatch {
        path: failed,
        expected,
        actual,
    }) = error
    else {
        panic!("expected a checksum mismatch, got {error}");
    };
    assert!(failed.ends_with("sample.json"));
    assert_eq!(expected.len(), 64);
    assert_eq!(actual.len(), 64);
    assert_ne!(expected, actual);
    Ok(())
}

#[test]
fn parsers_receive_verified_bytes_even_if_the_source_file_changes() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    let prepared = prepare(&fixture.manifest())?;
    let original = prepared.inputs().sources()[0].file().bytes().to_vec();
    fs::write(
        fixture.directory.join("sample.json"),
        "unverified replacement",
    )?;
    assert_eq!(prepared.inputs().sources()[0].file().bytes(), original);
    Ok(())
}

#[test]
fn modified_notices_are_rejected() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    fs::write(fixture.directory.join("NOTICE.txt"), "Changed notice\n")?;
    let error = prepare(&fixture.manifest()).expect_err("notice must be verified");
    assert!(matches!(
        error,
        GeneratorError::Inputs(InputError::ChecksumMismatch { path, .. })
            if path.ends_with("NOTICE.txt")
    ));
    Ok(())
}

#[test]
fn missing_sources_report_the_requested_path() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    fs::remove_file(fixture.directory.join("sample.json"))?;
    let error = prepare(&fixture.manifest()).expect_err("missing source must fail");
    assert!(matches!(
        error,
        GeneratorError::Inputs(InputError::Read { path, .. }) if path.ends_with("sample.json")
    ));
    Ok(())
}

#[test]
fn wrong_declared_sizes_are_rejected() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    let size = fs::metadata(fixture.directory.join("sample.json"))?.len();
    fixture.replace(&format!("size_bytes = {size}"), "size_bytes = 0")?;
    let error = prepare(&fixture.manifest()).expect_err("wrong size must fail");
    assert!(matches!(
        error,
        GeneratorError::Inputs(InputError::SizeMismatch { expected: 0, actual, .. }) if actual == size
    ));
    Ok(())
}

#[test]
fn malformed_manifests_are_rejected_before_input_loading() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    fs::write(fixture.manifest(), "not valid TOML = [")?;
    let error = prepare(&fixture.manifest()).expect_err("invalid TOML must fail");
    assert!(matches!(
        error,
        GeneratorError::Inputs(InputError::Manifest { .. })
    ));
    Ok(())
}

#[test]
fn manifests_cannot_override_the_engine_or_storage_policy() -> Result<(), Box<dyn Error>> {
    for (current, replacement, field) in [
        ("v2.0.0", "v1.4.3", "duckdb_engine"),
        ("v1.4.0", "latest", "storage_compatibility_version"),
        (
            "@duckdb/duckdb-wasm@1.32.0",
            "@duckdb/duckdb-wasm@latest",
            "duckdb_wasm_package",
        ),
    ] {
        let fixture = Fixture::copy()?;
        fixture.replace(current, replacement)?;
        let error = prepare(&fixture.manifest()).expect_err("policy override must fail");
        assert!(matches!(
            error,
            GeneratorError::Inputs(InputError::PolicyMismatch { field: failed, .. }) if failed == field
        ));
    }
    Ok(())
}

#[test]
fn parent_paths_are_rejected() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    fixture.replace("path = \"sample.json\"", "path = \"../sample.json\"")?;
    let error = prepare(&fixture.manifest()).expect_err("source must remain local");
    assert!(matches!(
        error,
        GeneratorError::Inputs(InputError::InvalidPath { .. })
    ));
    Ok(())
}

#[test]
fn undeclared_source_notices_are_rejected() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    fixture.replace("notices = [\"NOTICE.txt\"]", "notices = [\"unknown.txt\"]")?;
    let error = prepare(&fixture.manifest()).expect_err("notice must be declared");
    assert!(matches!(
        error,
        GeneratorError::Inputs(InputError::UndeclaredNotice { .. })
    ));
    Ok(())
}

#[test]
fn invalid_hashes_are_rejected_before_reading_the_source() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    let text = fs::read_to_string(fixture.manifest())?;
    let digest = text
        .lines()
        .find(|line| line.starts_with("sha256 = "))
        .expect("fixture contains a source digest");
    fixture.replace(digest, "sha256 = \"not-a-sha256\"")?;
    let error = prepare(&fixture.manifest()).expect_err("invalid hash must fail");
    assert!(matches!(
        error,
        GeneratorError::Inputs(InputError::InvalidDigest { .. })
    ));
    Ok(())
}

#[test]
fn repeated_notice_files_are_rejected() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::copy()?;
    let manifest = fs::read_to_string(fixture.manifest())?;
    let (_, notice) = manifest
        .split_once("[[notices]]")
        .expect("fixture contains a notice");
    fs::write(
        fixture.manifest(),
        format!("{manifest}\n[[notices]]{notice}"),
    )?;
    let error = prepare(&fixture.manifest()).expect_err("a file must not be counted twice");
    assert!(matches!(
        error,
        GeneratorError::Inputs(InputError::DuplicateFile { path }) if path.ends_with("NOTICE.txt")
    ));
    Ok(())
}

#[test]
fn command_verifies_the_preserved_inputs() -> Result<(), Box<dyn Error>> {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_db-gen"))
        .env_remove("RUST_LOG")
        .output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = String::from_utf8(result.stdout)?;
    assert!(output.contains("sources=2"), "{output}");
    assert!(output.contains("notices=3"), "{output}");
    assert!(output.contains("v2.0.0"), "{output}");
    assert!(output.contains("v1.4.0"), "{output}");
    Ok(())
}

#[test]
fn invalid_logging_configuration_fails_clearly() -> Result<(), Box<dyn Error>> {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_db-gen"))
        .env("RUST_LOG", "[")
        .output()?;
    assert!(!result.status.success());
    assert!(String::from_utf8(result.stderr)?.contains("Invalid RUST_LOG filter"));
    Ok(())
}
