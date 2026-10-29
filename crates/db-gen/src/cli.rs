use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use crate::{import_primary, prepare};

#[derive(Parser)]
#[command(
    name = "db-gen",
    version,
    about = "Generate the Zipeek dictionary from verified local inputs",
    after_help = "Examples (repository root):\n  cargo run -p db-gen --locked -- check\n  cargo run -p db-gen --locked -- generate --output target/reviewed-dictionary\n\nNo database is written by check or by displaying this help."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Verify and import inputs in memory without writing a database")]
    Check,
    #[command(about = "Write a complete verified database bundle to a new directory")]
    Generate {
        #[arg(short, long, value_name = "DIRECTORY", default_value_os_t = default_output(), help = "Output bundle directory; must not already exist")]
        output: PathBuf,
    },
}

fn default_output() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/static")
}

pub fn run_cli() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let code = error.exit_code() as u8;
            if let Err(error) = error.print() {
                eprintln!("Could not print CLI help/error: {error}");
                return ExitCode::FAILURE;
            }
            return ExitCode::from(code);
        }
    };
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
    if let Command::Generate { output } = cli.command {
        return generate_artifact(&manifest, &output);
    }
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
                    println!(
                        "Check complete: {} readings imported in memory. No database file was written.\nTo write a bundle: cargo run -p db-gen --locked -- generate --output <NEW_DIRECTORY>",
                        imported.words.len()
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

fn generate_artifact(manifest: &Path, directory: &Path) -> ExitCode {
    let directory = match std::path::absolute(directory) {
        Ok(directory) => directory,
        Err(error) => {
            eprintln!(
                "Could not resolve output directory {}: {error}",
                directory.display()
            );
            return ExitCode::FAILURE;
        }
    };
    println!("Output directory: {}", directory.display());
    match crate::artifact::generate(manifest, &directory) {
        Ok(report) => {
            tracing::info!(path = %directory.display(), rows = report.rows,
                size_bytes = report.files[0].size_bytes, sha256 = %report.files[0].sha256,
                "Verified database artifact generated");
            println!(
                "Database written: {}\nReadings: {}\nManifest: {}",
                directory.join("hanzi.duckdb").display(),
                report.rows,
                directory.join("hanzi.manifest.json").display()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Generation failed for {}: {error}", directory.display());
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, error::ErrorKind};

    #[test]
    fn missing_command_shows_help_instead_of_silently_importing() {
        Cli::command().debug_assert();
        let error = Cli::try_parse_from(["db-gen"]).err().unwrap();
        assert_eq!(
            error.kind(),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
        let help = error.to_string();
        assert!(help.contains("check") && help.contains("generate --output"));
    }

    #[test]
    fn explicit_output_and_default_directory_are_unambiguous() {
        let explicit =
            Cli::try_parse_from(["db-gen", "generate", "--output", "target/review bundle"])
                .unwrap();
        let Command::Generate { output } = explicit.command else {
            panic!("Expected generate")
        };
        assert_eq!(output, PathBuf::from("target/review bundle"));
        let default = Cli::try_parse_from(["db-gen", "generate"]).unwrap();
        let Command::Generate { output } = default.command else {
            panic!("Expected generate")
        };
        assert_eq!(output, default_output());
        assert!(Cli::try_parse_from(["db-gen", "generate", "--output"]).is_err());
        assert!(Cli::try_parse_from(["db-gen", "--unknown"]).is_err());
    }
}
