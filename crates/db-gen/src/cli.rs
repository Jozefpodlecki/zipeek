use std::{path::Path, process::ExitCode};

use tracing_subscriber::EnvFilter;

use crate::{import_primary, prepare};

pub fn run_cli() -> ExitCode {
    let filter = if std::env::var_os("RUST_LOG").is_some() {
        match EnvFilter::try_from_default_env() {
            Ok(filter) => filter,
            Err(error) => {
                eprintln!("Invalid RUST_LOG filter: {error}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        EnvFilter::new("info")
    };
    if let Err(error) = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .try_init()
    {
        eprintln!("Could not initialize generator logging: {error}");
        return ExitCode::FAILURE;
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/MANIFEST.toml");
    match prepare(&manifest) {
        Ok(prepared) => {
            tracing::info!(
                sources = prepared.inputs().sources().len(),
                notices = prepared.inputs().notices().len(),
                verified_bytes = prepared.inputs().total_bytes(),
                "Generator inputs verified and empty words schema prepared"
            );
            match import_primary(&prepared) {
                Ok(imported) => {
                    tracing::info!(
                        source_entries = imported.source_entries,
                        rejected_entries = imported.rejected_entries,
                        words = imported.words.len(),
                        diagnostics = imported.diagnostics.len(),
                        "Primary HSK import complete in memory; no artifact written"
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("Generation failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(error) => {
            eprintln!("Generation failed: {error}");
            ExitCode::FAILURE
        }
    }
}
