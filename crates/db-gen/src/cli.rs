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
        #[arg(long, value_name = "JSONL", help = "Apply validated sense classifications from a local JSONL cache")]
        classification_cache: Option<PathBuf>,
    },
    #[command(about = "Classify a bounded number of dictionary senses through local LM Studio")]
    Classify {
        #[arg(long, value_name = "MODEL")]
        model: String,
        #[arg(long, value_name = "JSONL")]
        cache: PathBuf,
        #[arg(long, default_value = "http://127.0.0.1:1234", value_name = "URL")]
        endpoint: String,
        #[arg(long, default_value_t = 10, value_name = "COUNT")]
        limit: usize,
        #[arg(long, default_value_t = 120000, value_name = "MS")]
        timeout_ms: u64,
        #[arg(long, default_value_t = 1)]
        retries: u32,
    },
    #[command(about = "Send one real LM Studio classification request; validate streamed JSON without importing or writing a database")]
    AiSmoke {
        #[arg(long, value_name = "MODEL")]
        model: String,
        #[arg(long, default_value = "http://127.0.0.1:1234", value_name = "URL")]
        endpoint: String,
        #[arg(long, default_value_t = 300000, value_name = "MS")]
        timeout_ms: u64,
        #[arg(long, default_value_t = 0)]
        retries: u32,
    },
    #[command(about = "Preview sense classification work and cache reuse without contacting AI")]
    ClassifyPlan {
        #[arg(long, value_name = "MODEL")]
        model: String,
        #[arg(long, value_name = "JSONL")]
        cache: PathBuf,
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
        .try_init()
    {
        eprintln!("Could not initialize generator logging: {error}");
        return ExitCode::FAILURE;
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/MANIFEST.toml");
    match cli.command {
        Command::Generate { output, classification_cache } => {
            return generate_artifact(&manifest, &output, classification_cache.as_deref());
        }
        Command::Classify { model, cache, endpoint, limit, timeout_ms, retries } => {
            return classify_live(&manifest, &model, &cache, &endpoint, limit, timeout_ms, retries);
        }
        Command::AiSmoke { model, endpoint, timeout_ms, retries } => {
            return ai_smoke(&model, &endpoint, timeout_ms, retries);
        }
        Command::ClassifyPlan { model, cache } => {
            return classification_plan(&manifest, &model, &cache);
        }
        Command::Check => {}
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

fn classify_live(
    manifest: &Path,
    model: &str,
    cache: &Path,
    endpoint: &str,
    limit: usize,
    timeout_ms: u64,
    retries: u32,
) -> ExitCode {
    if model.trim().is_empty() || endpoint.trim().is_empty() || timeout_ms == 0 || limit == 0 {
        eprintln!("Model and endpoint must be nonempty; timeout and limit must be positive");
        return ExitCode::FAILURE;
    }
    let result = (|| -> Result<_, Box<dyn std::error::Error>> {
        let prepared = prepare(manifest)?;
        let imported = import_primary(&prepared)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let client = crate::llm::LlmClient::new(&crate::llm::client::Config {
            base_url: endpoint.to_owned(),
            request_timeout_ms: timeout_ms,
            first_chunk_timeout_ms: timeout_ms,
            max_retries: retries,
        });
        Ok(runtime.block_on(crate::llm::batch::run_with_client(
            &client, model, &imported.words, &imported.senses, cache, limit,
        ))?)
    })();
    match result {
        Ok(summary) => {
            println!(
                "Model: {model}\nReused: {}\nGenerated: {}\nRemaining: {}\nDeferred: {}\nFailure report: {}\nCache: {}",
                summary.reused, summary.generated, summary.remaining, summary.deferred,
                crate::llm::batch::failure_report_path(cache).display(), cache.display()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Classification failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn ai_smoke(model: &str, endpoint: &str, timeout_ms: u64, retries: u32) -> ExitCode {
    if model.trim().is_empty() || endpoint.trim().is_empty() || timeout_ms == 0 {
        eprintln!("Model and endpoint must be nonempty; timeout must be positive");
        return ExitCode::FAILURE;
    }
    let result = (|| -> Result<_, Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let client = crate::llm::LlmClient::new(&crate::llm::client::Config {
            base_url: endpoint.to_owned(),
            request_timeout_ms: timeout_ms,
            first_chunk_timeout_ms: timeout_ms,
            max_retries: retries,
        });
        let word_id = zipeek_core::WordId::try_from(1)?;
        let word = zipeek_core::Word {
            id: word_id,
            simplified: "苹果".into(),
            traditional: Some("蘋果".into()),
            pinyin: "píng guǒ".into(),
            pinyin_numeric: Some("ping2 guo3".into()),
            english: Some("apple (fruit)".into()),
            part_of_speech: vec![],
            hsk2_level: None,
            hsk3_level: None,
            frequency: None,
        };
        let sense = crate::import::SourceSense {
            word_id,
            sense_order: 1,
            gloss: "apple (fruit)".into(),
            source: "ai-smoke",
            source_ref: 1,
            source_gloss_order: 1,
        };
        Ok(runtime.block_on(crate::llm::classification::classify(&client, model, &word, &sense))?)
    })();
    match result {
        Ok(record) => {
            println!("AI smoke passed. Model: {model}\nPoS: {:?}\nCategories: {:?}\nNo database or classification cache was written.", record.labels.pos, record.labels.categories);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("AI smoke failed: {error}. No database or classification cache was written.");
            ExitCode::FAILURE
        }
    }
}

fn classification_plan(manifest: &Path, model: &str, cache: &Path) -> ExitCode {
    let result = (|| -> Result<_, Box<dyn std::error::Error>> {
        let prepared = prepare(manifest)?;
        let imported = import_primary(&prepared)?;
        let cached = crate::llm::classification::read_cache(cache)?;
        Ok(crate::llm::batch::plan(&imported.words, &imported.senses, cached, model)?)
    })();
    match result {
        Ok(plan) => {
            println!("Model: {model}\nCached valid senses: {}\nPending senses: {}\nCache: {}\nNo AI requests or file writes were performed.",
                plan.cached.len(), plan.pending.len(), cache.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Classification planning failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn generate_artifact(manifest: &Path, directory: &Path, cache: Option<&Path>) -> ExitCode {
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
    match crate::artifact::generate_with_cache(manifest, &directory, cache) {
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
        let Command::Generate { output, .. } = explicit.command else {
            panic!("Expected generate")
        };
        assert_eq!(output, PathBuf::from("target/review bundle"));
        let default = Cli::try_parse_from(["db-gen", "generate"]).unwrap();
        let Command::Generate { output, .. } = default.command else {
            panic!("Expected generate")
        };
        assert_eq!(output, default_output());
        assert!(Cli::try_parse_from(["db-gen", "generate", "--output"]).is_err());
        assert!(Cli::try_parse_from(["db-gen", "--unknown"]).is_err());
    }
}
