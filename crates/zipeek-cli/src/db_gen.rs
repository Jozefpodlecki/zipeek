use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Subcommand;
#[cfg(test)]
use clap::Parser;

use zipeek_db_gen::{import_primary, prepare};

#[cfg(test)]
#[derive(Parser)]
#[command(
    name = "zipeek-cli db-gen",
    version,
    about = "Generate the Zipeek dictionary from verified local inputs",
    after_help = "Examples (repository root):\n  cargo run -p zipeek-cli --locked -- db-gen check\n  cargo run -p zipeek-cli --locked -- db-gen generate --output target/reviewed-dictionary\n\nNo database is written by check or by displaying this help."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Verify and import inputs in memory without writing a database")]
    Check,
    #[command(about = "Write a complete verified database bundle to a new directory")]
    Generate {
        #[arg(short, long, value_name = "DIRECTORY", default_value_os_t = default_output(), help = "Output bundle directory; must not already exist")]
        output: PathBuf,
        #[arg(
            long,
            value_name = "JSONL",
            help = "Apply validated sense classifications from a local JSONL cache"
        )]
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
        #[arg(
            long,
            default_value_t = 20,
            value_name = "SENSES",
            help = "Senses per model request (1..=100); default 20"
        )]
        batch_size: usize,
        #[arg(long, default_value_t = 120000, value_name = "MS")]
        timeout_ms: u64,
        #[arg(long, default_value_t = 1)]
        retries: u32,
    },
    #[command(
        about = "Verify a single-sense and configurable large-batch LM Studio JSON request without writing a database"
    )]
    AiSmoke {
        #[arg(long, value_name = "MODEL")]
        model: String,
        #[arg(long, default_value = "http://127.0.0.1:1234", value_name = "URL")]
        endpoint: String,
        #[arg(long, default_value_t = 300000, value_name = "MS")]
        timeout_ms: u64,
        #[arg(long, default_value_t = 0)]
        retries: u32,
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u8).range(1..=100), help = "Real imported senses in the batch smoke (1..=100); default 100")]
        batch_size: u8,
    },
    #[command(
        about = "Convert the supplied CC-CEDICT gloss classifications into the validated Zipeek sense cache (no AI requests)"
    )]
    ImportCedictClassifications {
        #[arg(long, value_name = "JSONL")]
        output: PathBuf,
    },
    #[command(
        about = "Prepare a validated classification cache without generating or installing DuckDB"
    )]
    AiWorkflow {
        #[arg(long, help = "Test live classification without changing the database")]
        smoke: bool,
        #[arg(
            long,
            conflicts_with = "smoke",
            help = "Prepare classifications without publishing a database (same as default)"
        )]
        classify_only: bool,
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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../zipeek-web/static")
}

pub fn run_cli(command: Command) -> ExitCode {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../zipeek-db-gen/data/MANIFEST.toml");
    match command {
        Command::Generate {
            output,
            classification_cache,
        } => {
            return generate_artifact(&manifest, &output, classification_cache.as_deref());
        }
        Command::Classify {
            model,
            cache,
            endpoint,
            limit,
            batch_size,
            timeout_ms,
            retries,
        } => {
            return classify_live(
                &manifest, &model, &cache, &endpoint, limit, batch_size, timeout_ms, retries,
            );
        }
        Command::AiSmoke {
            model,
            endpoint,
            timeout_ms,
            retries,
            batch_size,
        } => {
            return ai_smoke(
                &manifest,
                &model,
                &endpoint,
                timeout_ms,
                retries,
                usize::from(batch_size),
            );
        }
        Command::ImportCedictClassifications { output } => {
            return import_cedict_classifications(&manifest, &output);
        }
        Command::ClassifyPlan { model, cache } => {
            return classification_plan(&manifest, &model, &cache);
        }
        Command::AiWorkflow {
            smoke,
            classify_only: _,
        } => {
            return crate::db_ai_workflow::run(smoke);
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
                        "Check complete: {} readings imported in memory. No database file was written.\nTo write a bundle: cargo run -p zipeek-cli --locked -- db-gen generate --output <NEW_DIRECTORY>",
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
    batch_size: usize,
    timeout_ms: u64,
    retries: u32,
) -> ExitCode {
    if model.trim().is_empty()
        || endpoint.trim().is_empty()
        || timeout_ms == 0
        || limit == 0
        || !(1..=zipeek_db_gen::llm::batch::MAX_REQUEST_SIZE).contains(&batch_size)
    {
        eprintln!("Model and endpoint must be nonempty; timeout and limit must be positive");
        return ExitCode::FAILURE;
    }
    let result = (|| -> Result<_, Box<dyn std::error::Error>> {
        let prepared = prepare(manifest)?;
        let imported = import_primary(&prepared)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let client = zipeek_db_gen::llm::LlmClient::new(&zipeek_db_gen::llm::LlmClientConfig {
            base_url: endpoint.to_owned(),
            request_timeout_ms: timeout_ms,
            first_chunk_timeout_ms: timeout_ms,
            max_retries: retries,
        });
        Ok(runtime.block_on(zipeek_db_gen::llm::batch::run_with_client(
            &client,
            model,
            &imported.words,
            &imported.senses,
            cache,
            limit,
            batch_size,
        ))?)
    })();
    match result {
        Ok(summary) => {
            println!(
                "Model: {model}\nReused: {}\nGenerated: {}\nRemaining: {}\nDeferred: {}\nFailure report: {}\nCache: {}",
                summary.reused,
                summary.generated,
                summary.remaining,
                summary.deferred,
                zipeek_db_gen::llm::batch::failure_report_path(cache).display(),
                cache.display()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Classification failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn smoke_sample(
    words: &[zipeek_core::Word],
    senses: &[zipeek_db_gen::import::SourceSense],
    size: usize,
) -> Result<Vec<(zipeek_core::Word, zipeek_db_gen::import::SourceSense)>, Box<dyn std::error::Error>> {
    use std::collections::BTreeMap;
    if size == 0 || size > 100 || senses.len() < size {
        return Err("AI smoke batch size must be 1..=100 and fit the imported senses".into());
    }
    let by_id: BTreeMap<_, _> = words.iter().map(|word| (word.id, word)).collect();
    let mut sample = Vec::with_capacity(size);
    for index in 0..size {
        let sense = &senses[index * senses.len() / size];
        let word = by_id
            .get(&sense.word_id)
            .ok_or("imported smoke sense has no matching word")?;
        sample.push(((**word).clone(), sense.clone()));
    }
    Ok(sample)
}

fn ai_smoke(
    manifest: &Path,
    model: &str,
    endpoint: &str,
    timeout_ms: u64,
    retries: u32,
    batch_size: usize,
) -> ExitCode {
    if model.trim().is_empty()
        || endpoint.trim().is_empty()
        || timeout_ms == 0
        || !(1..=100).contains(&batch_size)
    {
        eprintln!("Model and endpoint must be nonempty; timeout positive; batch size 1..=100");
        return ExitCode::FAILURE;
    }
    let result = (|| -> Result<_, Box<dyn std::error::Error>> {
        let prepared = prepare(manifest)?;
        let imported = import_primary(&prepared)?;
        let sample = smoke_sample(&imported.words, &imported.senses, batch_size)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let client = zipeek_db_gen::llm::LlmClient::new(&zipeek_db_gen::llm::LlmClientConfig {
            base_url: endpoint.to_owned(),
            request_timeout_ms: timeout_ms,
            first_chunk_timeout_ms: timeout_ms,
            max_retries: retries,
        });
        let (first_word, first_sense) = &sample[0];
        let single = runtime.block_on(zipeek_db_gen::llm::classification::classify(
            &client,
            model,
            first_word,
            first_sense,
        ))?;
        tracing::info!(batch_size, "AI smoke submitting real dictionary senses");
        let batch = runtime.block_on(zipeek_db_gen::llm::classification::classify_many(
            &client, model, &sample,
        ))?;
        Ok((single, batch.len()))
    })();
    match result {
        Ok((record, count)) => {
            println!(
                "AI smoke passed (single + {count}-sense batch). Model: {model}\nPoS: {:?}\nCategories: {:?}\nNo database or classification cache was written.",
                record.labels.pos, record.labels.categories
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "AI smoke failed: {error}. No database or classification cache was written. Check target/db-workflow/classification-rejections.jsonl for rejected responses."
            );
            ExitCode::FAILURE
        }
    }
}

fn import_cedict_classifications(manifest: &Path, output: &Path) -> ExitCode {
    let result = (|| -> Result<_, Box<dyn std::error::Error>> {
        let prepared = prepare(manifest)?;
        let imported = import_primary(&prepared)?;
        let data = manifest.parent().ok_or("missing data directory")?;
        zipeek_db_gen::cedict_classifications::import(
            &data.join("classifications/cc-cedict-2026-10-26-classified.json"),
            &data.join("classifications/manifest.json"),
            &data.join("cc-cedict-2026-10-26.u8"),
            &imported.words,
            &imported.senses,
            output,
        )
    })();
    match result {
        Ok(summary) => {
            println!(
                "CC-CEDICT rule-based classifications imported: {} senses\nExact source references: {}\nExact lexical matches: {}\nUnmatched (unclassified): {}\nMatched but flagged for review: {}\nAmbiguous lexical matches: {}\nCache: {}\nNo model requests were made.",
                summary.total,
                summary.exact_source,
                summary.exact_gloss,
                summary.unresolved,
                summary.review,
                summary.ambiguous,
                output.display()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("CC-CEDICT classification import failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn classification_plan(manifest: &Path, model: &str, cache: &Path) -> ExitCode {
    let result = (|| -> Result<_, Box<dyn std::error::Error>> {
        let prepared = prepare(manifest)?;
        let imported = import_primary(&prepared)?;
        let cached = zipeek_db_gen::llm::classification::read_cache(cache)?;
        Ok(zipeek_db_gen::llm::batch::plan(
            &imported.words,
            &imported.senses,
            cached,
            model,
        )?)
    })();
    match result {
        Ok(plan) => {
            println!(
                "Model: {model}\nCached valid senses: {}\nPending senses: {}\nCache: {}\nNo AI requests or file writes were performed.",
                plan.cached.len(),
                plan.pending.len(),
                cache.display()
            );
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
    match zipeek_db_gen::artifact::generate_with_cache(manifest, &directory, cache) {
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
        let error = Cli::try_parse_from(["zipeek-cli db-gen"]).err().unwrap();
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
            Cli::try_parse_from(["zipeek-cli db-gen", "generate", "--output", "target/review bundle"])
                .unwrap();
        let Command::Generate { output, .. } = explicit.command else {
            panic!("Expected generate")
        };
        assert_eq!(output, PathBuf::from("target/review bundle"));
        let default = Cli::try_parse_from(["zipeek-cli db-gen", "generate"]).unwrap();
        let Command::Generate { output, .. } = default.command else {
            panic!("Expected generate")
        };
        assert_eq!(output, default_output());
        assert!(Cli::try_parse_from(["zipeek-cli db-gen", "generate", "--output"]).is_err());
        assert!(Cli::try_parse_from(["zipeek-cli db-gen", "--unknown"]).is_err());
    }
}
