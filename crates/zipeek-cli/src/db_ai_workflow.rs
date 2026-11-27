use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use sha2::{Digest, Sha256};

const DEFAULT_MODEL: &str = "qwen3-14b-claude-4.5-opus-high-reasoning-distill";
const CACHE_VERSION: &str = "v6";

#[derive(Debug, PartialEq, Eq)]
struct Config {
    model: String,
    endpoint: String,
    timeout_ms: u64,
    retries: u32,
    batch_size: usize,
    cache: PathBuf,
}

fn numeric(name: &str, default: &str, zero_allowed: bool) -> Result<u64, Box<dyn Error>> {
    let value = env::var(name).unwrap_or_else(|_| default.to_owned());
    if value.is_empty()
        || !value.bytes().all(|c| c.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(format!("{name} must be a decimal integer").into());
    }
    let parsed: u64 = value.parse()?;
    if parsed == 0 && !zero_allowed {
        return Err(format!("{name} must be positive").into());
    }
    Ok(parsed)
}

impl Config {
    fn from_env(root: &Path) -> Result<Self, Box<dyn Error>> {
        let model = env::var("ZIPEEK_AI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_owned());
        if model.trim().is_empty() {
            return Err("ZIPEEK_AI_MODEL must not be empty".into());
        }
        let endpoint =
            env::var("ZIPEEK_AI_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:1234".into());
        if !(endpoint.starts_with("http://") || endpoint.starts_with("https://"))
            || endpoint
                .split_once("://")
                .is_none_or(|(_, host)| host.is_empty())
        {
            return Err("ZIPEEK_AI_ENDPOINT must be an HTTP(S) URL".into());
        }
        let timeout_ms = numeric("ZIPEEK_AI_TIMEOUT_MS", "300000", false)?;
        let retries = u32::try_from(numeric("ZIPEEK_AI_RETRIES", "1", true)?)?;
        let batch_size = usize::try_from(numeric("ZIPEEK_AI_BATCH_SIZE", "2", false)?)?;
        if batch_size > 100 {
            return Err("ZIPEEK_AI_BATCH_SIZE must be at most 100".into());
        }
        let digest = Sha256::digest(model.as_bytes());
        let hash = digest[..8]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let cache = root
            .join("target/db-workflow/classification-cache")
            .join(format!("{hash}-{CACHE_VERSION}.jsonl"));
        Ok(Self {
            model,
            endpoint: endpoint.trim_end_matches('/').to_owned(),
            timeout_ms,
            retries,
            batch_size,
            cache,
        })
    }
}

fn pending_from_plan(stdout: &str) -> Result<usize, Box<dyn Error>> {
    let values: Vec<_> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("Pending senses: "))
        .collect();
    if values.len() != 1 {
        return Err("Could not determine pending classifications from db-gen plan output".into());
    }
    Ok(values[0].trim().parse::<usize>()?)
}

fn run_db_gen(args: &[String], capture: bool) -> Result<String, Box<dyn Error>> {
    let exe = env::current_exe()?;
    let mut cmd = Command::new(exe);
    cmd.arg("db-gen").args(args);
    if capture {
        cmd.stdout(Stdio::piped());
    } else {
        cmd.stdout(Stdio::inherit());
    }
    cmd.stderr(Stdio::inherit());
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(format!("db-gen {} failed; installed bundle unchanged", args[0]).into());
    }
    Ok(if capture {
        String::from_utf8(output.stdout)?
    } else {
        String::new()
    })
}

fn base_args(action: &str, config: &Config) -> Vec<String> {
    vec![action.to_owned(), "--model".into(), config.model.clone()]
}

fn smoke_args(config: &Config, batch_size: usize) -> Vec<String> {
    let mut args = base_args("ai-smoke", config);
    args.extend([
        "--endpoint".into(),
        config.endpoint.clone(),
        "--timeout-ms".into(),
        config.timeout_ms.to_string(),
        "--retries".into(),
        config.retries.to_string(),
        "--batch-size".into(),
        batch_size.to_string(),
    ]);
    args
}

fn plan_args(config: &Config) -> Vec<String> {
    let mut args = base_args("classify-plan", config);
    args.extend(["--cache".into(), config.cache.display().to_string()]);
    args
}

fn classify_args(config: &Config, pending: usize) -> Vec<String> {
    let mut args = base_args("classify", config);
    args.extend([
        "--cache".into(),
        config.cache.display().to_string(),
        "--endpoint".into(),
        config.endpoint.clone(),
        "--timeout-ms".into(),
        config.timeout_ms.to_string(),
        "--retries".into(),
        config.retries.to_string(),
        "--limit".into(),
        pending.to_string(),
        "--batch-size".into(),
        config.batch_size.to_string(),
    ]);
    args
}

fn execute_workflow(
    config: &Config,
    smoke_only: bool,
    mut execute_db_gen: impl FnMut(&[String], bool) -> Result<String, Box<dyn Error>>,
) -> Result<Option<PathBuf>, Box<dyn Error>> {
    if smoke_only {
        execute_db_gen(&smoke_args(config, 100), false)?;
        println!(
            "LM Studio smoke passed for {}; no database changes",
            config.model
        );
        return Ok(None);
    }
    let pending = pending_from_plan(&execute_db_gen(&plan_args(config), true)?)?;
    println!(
        "Model: {}\nCache: {}\nPending senses: {pending}",
        config.model,
        config.cache.display()
    );
    if pending > 0 {
        execute_db_gen(&smoke_args(config, config.batch_size), false)?;
        execute_db_gen(&classify_args(config, pending), false)?;
    }
    let remaining = pending_from_plan(&execute_db_gen(&plan_args(config), true)?)?;
    if remaining != 0 {
        return Err(
            format!("{remaining} senses remain unclassified; installed bundle unchanged").into(),
        );
    }
    Ok(Some(config.cache.clone()))
}

fn prepare_cedict_cache(
    root: &Path,
    mut execute_db_gen: impl FnMut(&[String], bool) -> Result<String, Box<dyn Error>>,
) -> Result<String, Box<dyn Error>> {
    let cache = root.join("target/db-workflow/cedict-gloss-rules-v1.jsonl");
    let cache_str = cache
        .to_str()
        .ok_or("CC-CEDICT cache path is not UTF-8")?
        .to_owned();
    if !cache.exists() {
        execute_db_gen(
            &[
                "import-cedict-classifications".into(),
                "--output".into(),
                cache_str.clone(),
            ],
            false,
        )?;
    }
    let plan = execute_db_gen(
        &[
            "classify-plan".into(),
            "--model".into(),
            zipeek_db_gen::cedict_classifications::MODEL.into(),
            "--cache".into(),
            cache_str.clone(),
        ],
        true,
    )?;
    let remaining = pending_from_plan(&plan)?;
    if remaining != 0 {
        return Err(format!("{remaining} senses not covered by the CC-CEDICT cache; installed bundle unchanged. Remove the stale cache and retry").into());
    }
    Ok(cache_str)
}

fn execute_cedict_workflow(
    root: &Path,
    execute_db_gen: impl FnMut(&[String], bool) -> Result<String, Box<dyn Error>>,
) -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(prepare_cedict_cache(root, execute_db_gen)?))
}

const SELECTION_FILE: &str = "selected-classification.json";

fn selection_path(root: &Path) -> PathBuf {
    root.join("target/db-workflow").join(SELECTION_FILE)
}

fn write_selection(root: &Path, cache: &Path) -> Result<(), Box<dyn Error>> {
    let directory = root.join("target/db-workflow").canonicalize()?;
    let cache = cache.canonicalize()?;
    if !cache.starts_with(&directory) || !cache.is_file() {
        return Err("Classification cache must be a regular file inside target/db-workflow".into());
    }
    let path = selection_path(root);
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let relative = cache.strip_prefix(&directory)?;
    let relative = relative.to_string_lossy().replace('\\', "/");
    let payload = serde_json::json!({"version": 1, "cache": relative});
    std::fs::write(&temporary, format!("{payload}\n"))?;
    if let Err(error) = std::fs::rename(&temporary, &path) {
        if path.exists() {
            std::fs::remove_file(&path)?;
            std::fs::rename(&temporary, &path)?;
        } else {
            let _ = std::fs::remove_file(&temporary);
            return Err(error.into());
        }
    }
    println!("Classification cache ready: {}", cache.display());
    println!("Generate an enriched candidate with: cargo make db-generate-enriched");
    Ok(())
}

fn run_inner(smoke_only: bool) -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;

    if !smoke_only {
        let selection = selection_path(&root);
        if selection.exists() {
            std::fs::remove_file(&selection)?;
        }
    }
    if !smoke_only && env::var("ZIPEEK_DB_CLASSIFIER").as_deref() != Ok("lmstudio") {
        let cache = execute_cedict_workflow(&root, run_db_gen)?;
        return write_selection(&root, &cache);
    }
    let config = Config::from_env(&root)?;
    let result = execute_workflow(&config, smoke_only, run_db_gen)?;
    if let Some(cache) = result {
        write_selection(&root, &cache)?;
    }
    Ok(())
}

pub fn run(smoke_only: bool) -> std::process::ExitCode {
    match run_inner(smoke_only) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("AI database workflow failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            model: "test".into(),
            endpoint: "http://127.0.0.1:1234".into(),
            timeout_ms: 300000,
            retries: 1,
            batch_size: 2,
            cache: "cache-v6.jsonl".into(),
        }
    }

    #[test]
    fn local_workflow_imports_and_validates_without_running_generator() {
        let mut actions = Vec::new();
        let cache = execute_cedict_workflow(Path::new("/empty/test/repository"), |args, _| {
            actions.push(args[0].clone());
            if args[0] == "classify-plan" {
                Ok("Pending senses: 0\n".into())
            } else {
                Ok(String::new())
            }
        })
        .unwrap();
        assert!(cache.ends_with("cedict-gloss-rules-v1.jsonl"));
        assert_eq!(actions, ["import-cedict-classifications", "classify-plan"]);
    }

    #[test]
    fn incomplete_cache_cannot_be_selected() {
        let err = execute_cedict_workflow(Path::new("/empty/test/repository"), |args, _| {
            if args[0] == "classify-plan" {
                Ok("Pending senses: 4\n".into())
            } else {
                Ok(String::new())
            }
        })
        .unwrap_err();
        assert!(err.to_string().contains("4 senses"));
    }

    #[test]
    fn live_workflow_prepares_cache_without_nested_generation() {
        let mut actions = Vec::new();
        let mut plans = 0;
        let result = execute_workflow(&config(), false, |args, _| {
            actions.push(args[0].clone());
            if args[0] == "classify-plan" {
                plans += 1;
                Ok(format!(
                    "Pending senses: {}\n",
                    if plans == 1 { 3 } else { 0 }
                ))
            } else {
                Ok(String::new())
            }
        })
        .unwrap();
        assert_eq!(result, Some(PathBuf::from("cache-v6.jsonl")));
        assert_eq!(
            actions,
            ["classify-plan", "ai-smoke", "classify", "classify-plan"]
        );
    }

    #[test]
    fn failed_smoke_never_returns_cache() {
        let err = execute_workflow(&config(), false, |args, _| {
            if args[0] == "classify-plan" {
                Ok("Pending senses: 1\n".into())
            } else {
                Err("smoke failed".into())
            }
        })
        .unwrap_err();
        assert!(err.to_string().contains("smoke failed"));
    }

    #[test]
    fn smoke_cannot_select_a_classification_cache() {
        let result = execute_workflow(&config(), true, |args, _| {
            assert_eq!(args[0], "ai-smoke");
            Ok(String::new())
        })
        .unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn plan_requires_one_pending_count() {
        assert_eq!(pending_from_plan("Pending senses: 0\n").unwrap(), 0);
        assert!(pending_from_plan("Pending senses: 1\nPending senses: 2\n").is_err());
    }

    #[test]
    fn selection_path_is_in_workflow_directory() {
        assert!(
            selection_path(Path::new("/repo"))
                .ends_with("target/db-workflow/selected-classification.json")
        );
    }
}
