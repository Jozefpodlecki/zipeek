use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{Harness, HarnessError, HarnessResult, ScenarioContext};
use serde_json::json;

pub fn valid_artifact_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

pub fn setup_failure(directory: &std::path::Path, name: &str, error: &HarnessError) {
    eprintln!("Harness setup failed: {error}");
    if let Err(write_error) = std::fs::write(directory.join(name), error.to_string()) {
        eprintln!("Setup diagnostic write failed: {write_error}");
    }
}

pub fn suite_start_failure(directory: &std::path::Path, error: &HarnessError) -> HarnessResult<()> {
    let result = json!({
        "schema": "zipeek-e2e-suite-v1",
        "passed": 0,
        "failed": 0,
        "infrastructure_failures": [error.to_string()],
        "scenarios": [],
    });
    std::fs::write(
        directory.join("suite.json"),
        serde_json::to_vec_pretty(&result).map_err(|error| error.to_string())?,
    )?;
    std::fs::write(
        directory.join("summary.txt"),
        format!("Suite startup failed: {error}"),
    )?;
    Ok(())
}

pub fn run_directory(suite: &str) -> HarnessResult<PathBuf> {
    if !valid_artifact_name(suite) {
        return Err(format!("Invalid E2E suite artifact name: {suite}").into());
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = Harness::repository_root()
        .join("target/e2e")
        .join(format!("{suite}-{}-{stamp}", std::process::id()));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub async fn diagnostics(context: &ScenarioContext) {
    if std::env::var_os("ZIPEEK_E2E_CAPTURE_AI_REQUESTS").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        capture_ai_requests(context).await;
    }
    capture_browser_diagnostics(context).await;
}

async fn capture_ai_requests(context: &ScenarioContext) {
    let result = super::FixtureClient::from_ai_url(&context.ai_url);
    match result {
        Ok(client) => match client.requests().await {
            Ok(records) => {
                if let Err(error) = std::fs::write(
                    context.artifacts.join("ai-fixture-requests.json"),
                    records.to_string(),
                ) {
                    eprintln!("AI fixture diagnostic write failed: {error}");
                }
            }
            Err(error) => eprintln!("AI fixture diagnostic capture failed: {error}"),
        },
        Err(error) => eprintln!("AI fixture configuration failed: {error}"),
    }
}

async fn capture_browser_diagnostics(context: &ScenarioContext) {
    match context
        .driver
        .execute(
            crate::js_scripts::harness::ARTIFACTS_CAPTURE_BROWSER_DIAGNOSTICS_01,
            vec![],
        )
        .await
    {
        Ok(result) => match result.convert::<String>() {
            Ok(text) => {
                if let Err(error) = std::fs::write(context.artifacts.join("browser.json"), text) {
                    eprintln!("Diagnostic write failed: {error}");
                }
            }
            Err(error) => eprintln!("Diagnostic conversion failed: {error}"),
        },
        Err(error) => eprintln!("Diagnostic capture failed: {error}"),
    }
}

pub async fn failure(context: &ScenarioContext, message: &str) {
    eprintln!("FAIL {}: {message}", context.artifacts.display());
    use std::io::Write;
    let write = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(context.artifacts.join("failure.txt"))
        .and_then(|mut file| writeln!(file, "{message}"));
    if let Err(error) = write {
        eprintln!("Failure log write failed: {error}");
    }
    match context
        .driver
        .execute(
            crate::js_scripts::harness::ARTIFACTS_CAPTURE_FAILURE_STATE,
            vec![],
        )
        .await
    {
        Ok(result) => {
            if let Ok(snapshot) = result.convert::<String>() {
                if let Err(error) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(context.artifacts.join("failure.txt"))
                    .and_then(|mut file| writeln!(file, "Browser state: {snapshot}"))
                {
                    eprintln!("Failure state write failed: {error}");
                }
            }
        }
        Err(error) => eprintln!("Failure state capture failed: {error}"),
    }
    if let Err(error) = context
        .driver
        .screenshot(&context.artifacts.join("failure.png"))
        .await
    {
        eprintln!("Failure screenshot unavailable: {error}");
    }
}

pub fn case_result(
    directory: &std::path::Path,
    status: &str,
    phase: &str,
    error: Option<&str>,
) -> HarnessResult<()> {
    if !matches!(status, "passed" | "failed") {
        return Err(format!("Invalid E2E scenario status: {status}").into());
    }
    if !matches!(
        phase,
        "browser"
            | "setup"
            | "scenario"
            | "recording"
            | "cleanup"
            | "expected_failure"
            | "isolation"
    ) {
        return Err(format!("Invalid E2E scenario phase: {phase}").into());
    }
    if status == "failed" && !error.is_some_and(|message| !message.trim().is_empty()) {
        return Err("Failed E2E case must include an actionable error message".into());
    }
    if status == "passed" && error.is_some() {
        return Err("Passed E2E case cannot contain a failure message".into());
    }
    let capture = std::fs::read(directory.join("video/status.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let recording_error = std::fs::read_to_string(directory.join("video-failure.txt")).ok();
    let checkpoint_error = std::fs::read_to_string(directory.join("checkpoint-failure.txt")).ok();
    let checkpoint_status = if checkpoint_error.is_some() {
        "failed"
    } else if directory.join("timeline/steps.jsonl").is_file() {
        "captured"
    } else {
        "unavailable"
    };
    let recording_status = if recording_error.is_some() {
        "failed"
    } else if capture
        .as_ref()
        .is_some_and(|value| value["truncated"] == true)
    {
        "truncated"
    } else if capture
        .as_ref()
        .is_some_and(|value| value["captured"].as_u64() == Some(0))
    {
        "unverified"
    } else if capture
        .as_ref()
        .is_some_and(|value| value["validation"]["status"] == "valid")
    {
        "validated"
    } else if capture.is_some() {
        "unverified"
    } else {
        "unavailable"
    };
    let assertion_status = std::fs::read_to_string(directory.join("assertion-status.txt"))
        .ok()
        .map(|value| value.trim().to_owned())
        .unwrap_or_else(|| {
            if matches!(phase, "scenario" | "isolation" | "expected_failure") {
                status.to_owned()
            } else {
                "not_run".to_owned()
            }
        });
    let result = json!({
        "schema": "zipeek-e2e-case-v2",
        "browser": std::fs::read_to_string(directory.join("browser.txt")).ok().map(|value| value.trim().to_owned()),
        "scenario": directory.file_name().and_then(|name| name.to_str()).unwrap_or("unknown"),
        "status": status,
        "phase": phase,
        "error": error,
        "video": directory.join("video/recording.mp4").is_file(),
        "frames": false,
        "recording_format": "mp4",
        "diagnostics": directory.join("browser.json").is_file(),
        "recording_status": recording_status,
        "recording_error": recording_error,
        "checkpoint_status": checkpoint_status,
        "checkpoint_error": checkpoint_error,
        "recording_validation": capture.as_ref().map(|value| &value["validation"]),
        "assertion_status": assertion_status,
        "execution_status": if status == "passed" { "passed" } else { "failed" },
        "failure_screenshot": directory.join("failure.png").is_file(),
    });
    std::fs::write(
        directory.join("scenario.json"),
        serde_json::to_vec_pretty(&result).map_err(|error| error.to_string())?,
    )?;
    write_agent_feedback(directory, &result)?;
    Ok(())
}

fn write_agent_feedback(
    directory: &std::path::Path,
    result: &serde_json::Value,
) -> HarnessResult<()> {
    let scenario = result["scenario"].as_str().unwrap_or("unknown");
    let status = result["execution_status"].as_str().unwrap_or("unknown");
    let recording = result["recording_status"].as_str().unwrap_or("unavailable");
    let phase = result["phase"].as_str().unwrap_or("unknown");
    let error = result["error"].as_str().unwrap_or("None reported");
    let recording_error = result["recording_error"]
        .as_str()
        .unwrap_or("None reported");
    let checkpoint_error = result["checkpoint_error"]
        .as_str()
        .unwrap_or("None reported");
    let checkpoint_status = result["checkpoint_status"]
        .as_str()
        .unwrap_or("unavailable");
    let recording_detail = result["recording_validation"]["detail"]
        .as_str()
        .unwrap_or("Not probed");
    let capture_start = std::fs::read(directory.join("video/status.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| value["started_unix_ms"].as_u64());
    let mut report = format!(
        "# E2E evidence: {scenario}\n\nExecution: {status}\n\nAssertion: {}\n\nVideo evidence: {recording}\n\nPhase: {phase}\n\nReported failure: {error}\n\nRecording failure: {recording_error}\n\nVideo validation: {recording_detail}\n\n",
        result["assertion_status"].as_str().unwrap_or("not_run")
    );
    report.push_str(&format!(
        "Checkpoint evidence: {checkpoint_status}\n\nCheckpoint failure: {checkpoint_error}\n\n"
    ));
    if directory.join("video/recording.mp4").is_file() {
        report.push_str("Video: `video/recording.mp4`\n\n");
    }
    if directory.join("failure.png").is_file() {
        report.push_str("Failure screenshot: `failure.png`\n\n");
    }
    if directory.join("browser.json").is_file() {
        report.push_str("Browser diagnostics: `browser.json`\n\n");
    }
    let timeline = directory.join("timeline/steps.jsonl");
    if let Ok(content) = std::fs::read_to_string(timeline) {
        report.push_str("## Evidence checkpoints\n\n");
        for line in content
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        {
            let label = line["label"].as_str().unwrap_or("unknown");
            let screenshot = line["screenshot"]
                .as_str()
                .map(|name| format!(" - `timeline/{name}`"))
                .unwrap_or_default();
            let timestamp = line["unix_ms"].as_u64().unwrap_or(0);
            let relative = capture_start
                .and_then(|start| timestamp.checked_sub(start))
                .map(|elapsed| format!(", approximately {} ms after recording start", elapsed))
                .unwrap_or_default();
            report.push_str(&format!(
                "- {label} (Unix ms {timestamp}{relative}){screenshot}\n"
            ));
        }
    }
    report.push_str("\nThis report is based on recorded assertions and artifacts. Video frames have not been interpreted automatically.\n");
    std::fs::write(directory.join("agent-feedback.md"), report)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifacts_live_under_repository_target() {
        let root = Harness::repository_root();
        let directory = run_directory("location-test").unwrap();
        assert!(directory.starts_with(root.join("target/e2e")));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn artifact_names_reject_traversal_and_unsafe_characters() {
        for name in ["../outside", "a/b", "a\\b", "", "a.json", "a b"] {
            assert!(!valid_artifact_name(name));
        }
        assert!(valid_artifact_name("ai-translation_01"));
    }

    #[test]
    fn startup_failure_creates_uploadable_suite_manifest() {
        let directory =
            std::env::temp_dir().join(format!("zipeek-e2e-startup-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let failure = HarnessError::from("App server failed to start");
        suite_start_failure(&directory, &failure).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("suite.json")).unwrap()).unwrap();
        assert_eq!(report["passed"], 0);
        assert_eq!(
            report["infrastructure_failures"][0],
            "App server failed to start"
        );
        assert!(directory.join("summary.txt").is_file());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn scenario_manifest_rejects_invalid_status_and_phase() {
        let root = std::env::temp_dir().join(format!("zipeek-bad-manifest-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        assert!(case_result(&root, "unknown", "setup", None).is_err());
        assert!(case_result(&root, "failed", "unknown", Some("failure")).is_err());
        assert!(case_result(&root, "failed", "scenario", None).is_err());
        assert!(case_result(&root, "failed", "scenario", Some("   ")).is_err());
        assert!(case_result(&root, "passed", "scenario", Some("incorrect")).is_err());
        assert!(!root.join("scenario.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scenario_manifest_exposes_failure_stage() {
        let directory =
            std::env::temp_dir().join(format!("zipeek-e2e-manifest-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        case_result(
            &directory,
            "failed",
            "setup",
            Some("Dictionary unavailable"),
        )
        .unwrap();
        let content: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("scenario.json")).unwrap())
                .unwrap();
        assert_eq!(content["phase"], "setup");
        assert_eq!(content["status"], "failed");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn case_evidence_separates_assertion_and_recording_results() {
        let directory =
            std::env::temp_dir().join(format!("zipeek-evidence-separation-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("video-failure.txt"), "FFmpeg unavailable").unwrap();
        case_result(&directory, "passed", "scenario", None).unwrap();
        let data: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("scenario.json")).unwrap())
                .unwrap();
        assert_eq!(data["execution_status"], "passed");
        assert_eq!(data["assertion_status"], "passed");
        assert_eq!(data["recording_status"], "failed");
        assert!(directory.join("agent-feedback.md").is_file());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn case_feedback_reports_failed_checkpoint_capture() {
        let directory =
            std::env::temp_dir().join(format!("zipeek-checkpoint-failed-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("checkpoint-failure.txt"),
            "Screenshot unavailable",
        )
        .unwrap();
        case_result(&directory, "passed", "scenario", None).unwrap();
        let data: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("scenario.json")).unwrap())
                .unwrap();
        assert_eq!(data["checkpoint_status"], "failed");
        let feedback = std::fs::read_to_string(directory.join("agent-feedback.md")).unwrap();
        assert!(feedback.contains("Screenshot unavailable"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn case_feedback_preserves_failed_assertion_and_evidence_links() {
        let directory =
            std::env::temp_dir().join(format!("zipeek-evidence-failed-{}", std::process::id()));
        std::fs::create_dir_all(directory.join("timeline")).unwrap();
        std::fs::write(
            directory.join("timeline/steps.jsonl"),
            r#"{"label":"incorrect-pair","unix_ms":100,"screenshot":"002-incorrect-pair.png"}"#,
        )
        .unwrap();
        case_result(
            &directory,
            "failed",
            "scenario",
            Some("Expected selected pair to flash red"),
        )
        .unwrap();
        let feedback = std::fs::read_to_string(directory.join("agent-feedback.md")).unwrap();
        assert!(feedback.contains("Expected selected pair to flash red"));
        assert!(feedback.contains("timeline/002-incorrect-pair.png"));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
