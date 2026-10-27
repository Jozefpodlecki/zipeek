use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use db_gen::artifact::{ArtifactError, ArtifactReport, generate, verify_database};
use duckdb_neo::Parameters;
use sha2::{Digest, Sha256};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Result<Self, Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!(
            "zipeek-artifact-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
    fn directory(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for name in ["first", "second", "existing", "quoted'output", "empty"] {
            let directory = self.0.join(name);
            for file in [
                "hanzi.duckdb",
                "hanzi.diagnostics.json",
                "hanzi.manifest.json",
                "INPUTS.toml",
                "ATTRIBUTION.md",
                "keep.txt",
            ] {
                let _ = fs::remove_file(directory.join(file));
            }
            let _ = fs::remove_file(directory.join("notices/NOTICE.txt"));
            let _ = fs::remove_dir(directory.join("notices"));
            let _ = fs::remove_dir(directory);
        }
        if let Err(error) = fs::remove_dir(&self.0) {
            eprintln!(
                "Could not remove test directory {}: {error}",
                self.0.display()
            );
        }
    }
}

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fallback/MANIFEST.toml")
}

#[test]
fn independent_exports_have_identical_logical_content_schema_ids_and_diagnostics()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new()?;
    let first = generate(&manifest(), &fixture.directory("first"))?;
    let second = generate(&manifest(), &fixture.directory("second"))?;
    assert_eq!(first.rows, 9);
    assert_eq!(first.logical_sha256, second.logical_sha256);
    assert_eq!(first.diagnostics_sha256, second.diagnostics_sha256);
    assert_eq!(first.columns, second.columns);
    assert_eq!(first.constraints, second.constraints);
    assert_eq!(first.indexes, second.indexes);
    assert_eq!(
        verify_database(&fixture.directory("first/hanzi.duckdb"))?,
        verify_database(&fixture.directory("second/hanzi.duckdb"))?
    );
    assert_eq!(
        fs::read(fixture.directory("first/hanzi.diagnostics.json"))?,
        fs::read(fixture.directory("second/hanzi.diagnostics.json"))?
    );
    assert_eq!(first.tables, ["words"]);
    assert_eq!(first.columns.len(), 10);
    assert_eq!(first.indexes.len(), 2);
    assert_eq!(first.constraints.len(), 5);
    assert_eq!(first.engine, "v2.0.0");
    assert_eq!(first.storage_compatibility, "v1.4.0");
    for file in &first.files {
        let bytes = fs::read(fixture.directory("first").join(&file.path))?;
        assert_eq!(bytes.len(), file.size_bytes);
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), file.sha256);
    }
    let saved: ArtifactReport =
        serde_json::from_slice(&fs::read(fixture.directory("first/hanzi.manifest.json"))?)?;
    assert_eq!(saved, first);
    assert_eq!(
        fs::read(fixture.directory("first/INPUTS.toml"))?,
        fs::read(manifest())?
    );
    Ok(())
}

#[test]
fn existing_output_is_preserved_and_failed_preparation_leaves_no_published_directory()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new()?;
    let output = fixture.directory("existing");
    fs::create_dir(&output)?;
    fs::write(output.join("keep.txt"), "preserve me")?;
    assert!(matches!(
        generate(&manifest(), &output),
        Err(ArtifactError::ExistingOutput { .. })
    ));
    assert_eq!(fs::read_to_string(output.join("keep.txt"))?, "preserve me");
    let fresh = fixture.directory("first");
    assert!(generate(&fixture.directory("missing.toml"), &fresh).is_err());
    assert!(!fresh.exists());
    assert_eq!(fs::read_dir(&fixture.0)?.count(), 1);
    Ok(())
}

#[test]
fn quoted_output_paths_are_sql_literals_and_missing_databases_are_not_created()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new()?;
    let output = fixture.directory("quoted'output");
    generate(&manifest(), &output)?;
    assert_eq!(verify_database(&output.join("hanzi.duckdb"))?.rows.len(), 9);
    let missing = fixture.directory("absent.duckdb");
    assert!(verify_database(&missing).is_err());
    assert!(!missing.exists());
    Ok(())
}

#[test]
fn read_only_attachment_rejects_mutation_without_changing_rows() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new()?;
    let output = fixture.directory("first");
    generate(&manifest(), &output)?;
    let file = output.join("hanzi.duckdb");
    let before = verify_database(&file)?;
    let database = db_gen::database::GeneratorDatabase::open()?;
    database.connection().execute(
        format!(
            "ATTACH '{}' AS saved (READ_ONLY)",
            file.to_str()
                .ok_or("non-UTF-8 fixture path")?
                .replace('\\', "/")
                .replace('\'', "''")
        ),
        Parameters::None,
    )?;
    let error = database
        .connection()
        .execute("DELETE FROM saved.words", Parameters::None)
        .unwrap_err();
    assert!(error.to_string().to_lowercase().contains("read-only"));
    database
        .connection()
        .execute("DETACH saved", Parameters::None)?;
    assert_eq!(before, verify_database(&file)?);
    Ok(())
}

#[test]
fn empty_dictionary_keeps_complete_schema_and_produces_valid_checksums()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new()?;
    let manifest =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/artifact-empty/MANIFEST.toml");
    let report = generate(&manifest, &fixture.directory("empty"))?;
    assert_eq!(report.rows, 0);
    assert_eq!(report.diagnostics, 0);
    let snapshot = verify_database(&fixture.directory("empty/hanzi.duckdb"))?;
    assert!(snapshot.rows.is_empty());
    assert_eq!(snapshot.columns.len(), 10);
    assert_eq!(snapshot.constraints.len(), 5);
    assert_eq!(snapshot.indexes.len(), 2);
    Ok(())
}

#[test]
fn command_refuses_existing_output_and_unknown_options() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new()?;
    let output = fixture.directory("existing");
    fs::create_dir(&output)?;
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_db-gen"))
        .arg("generate")
        .arg(&output)
        .output()?;
    assert!(!result.status.success());
    assert!(String::from_utf8(result.stderr)?.contains("already exists"));
    let invalid = std::process::Command::new(env!("CARGO_BIN_EXE_db-gen"))
        .arg("--unknown")
        .output()?;
    assert!(!invalid.status.success());
    assert!(String::from_utf8(invalid.stderr)?.contains("Usage:"));
    Ok(())
}
