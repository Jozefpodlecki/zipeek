use serde_json::{Value, json};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::archive::Result;

fn eligible(path: &Path, diagnostics: bool, include_videos: bool) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    matches!(
        name,
        "steps.jsonl"
            | "scenario.json"
            | "suite.json"
            | "passed.txt"
            | "failure.txt"
            | "failure.png"
            | "summary.txt"
            | "setup-failure.txt"
            | "browser-failure.txt"
            | "video-failure.txt"
            | "status.json"
            | "build.log"
            | "trunk.log"
            | "static-server.log"
            | "fixture-server.log"
            | "http.log"
    ) || include_videos && name == "recording.mp4"
        || diagnostics && matches!(name, "browser.json" | "ai-fixture-requests.json")
        || path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            == Some("timeline")
            && path.extension().and_then(|ext| ext.to_str()) == Some("png")
}

fn collect(
    folder: &Path,
    base: &Path,
    diagnostics: bool,
    include_videos: bool,
    output: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(format!("Evidence symlink refused: {}", path.display()).into());
        }
        if path.is_dir() {
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("dist" | "target" | "node_modules" | "vendor")
            ) {
                continue;
            }
            collect(&path, base, diagnostics, include_videos, output)?;
        } else if path.is_file() && eligible(&path, diagnostics, include_videos) {
            let _ = path.strip_prefix(base)?;
            output.push(path);
        }
    }
    Ok(())
}

pub fn clean(root: &Path) -> Result<usize> {
    let target = root.join("target");
    let e2e = target.join("e2e");
    if !fs::exists(&e2e)? {
        if fs::symlink_metadata(&e2e).is_ok() {
            return Err("Refusing to clean E2E evidence through a symlink".into());
        }
        return Ok(0);
    }
    if fs::symlink_metadata(&target)?.file_type().is_symlink()
        || fs::symlink_metadata(&e2e)?.file_type().is_symlink()
    {
        return Err("Refusing to clean E2E evidence through a symlink".into());
    }
    if !e2e.is_dir() {
        return Err("E2E evidence path is not a directory".into());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&e2e)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if !kind.is_dir() && !kind.is_file() {
            return Err(format!(
                "Refusing to clean special entry: {}",
                entry.path().display()
            )
            .into());
        }
        entries.push((entry.path(), kind.is_dir()));
    }
    for (path, is_dir) in &entries {
        if *is_dir {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    println!(
        "Cleaned {} E2E evidence entries in {}",
        entries.len(),
        e2e.display()
    );
    Ok(entries.len())
}

fn latest_suite(e2e: &Path) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(e2e)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() || !entry.file_name().to_string_lossy().starts_with("root-")
        {
            continue;
        }
        let path = entry.path();
        let has_summary = path.join("suite.json").exists() || path.join("summary.txt").exists();
        let mut scenario_files = Vec::new();
        if !has_summary {
            collect(&path, &path, false, false, &mut scenario_files)?;
        }
        if has_summary
            || scenario_files
                .iter()
                .any(|file| file.file_name().is_some_and(|name| name == "scenario.json"))
        {
            candidates.push((entry.metadata()?.modified()?, path));
        }
    }
    candidates.sort_by(|a, b| a.cmp(b));
    Ok(candidates
        .pop()
        .ok_or("No completed E2E evidence run found")?
        .1
        .canonicalize()?)
}

pub fn pack(
    root: &Path,
    run: Option<&Path>,
    output: Option<&Path>,
    diagnostics: bool,
    recorder: Option<&Path>,
    all: bool,
    include_videos: bool,
) -> Result<()> {
    let e2e = root.join("target/e2e");
    if all && run.is_some() {
        return Err("--all cannot be combined with --run".into());
    }
    let source = if all {
        e2e.canonicalize()?
    } else {
        match run {
            Some(path) => path.canonicalize()?,
            None => latest_suite(&e2e)?,
        }
    };
    let evidence_root = e2e.canonicalize()?;
    if all && source != evidence_root || !all && source.parent() != Some(evidence_root.as_path()) {
        return Err("E2E suite evidence must be a direct child of target/e2e".into());
    }
    let recorder_bytes = recorder
        .map(|path| -> Result<Vec<u8>> {
            if fs::symlink_metadata(path)?.file_type().is_symlink() {
                return Err("Recorder JSON symlinks are refused".into());
            }
            let bytes = fs::read(path)?;
            let parsed: Value = serde_json::from_slice(&bytes)?;
            if !parsed["steps"].is_array() {
                return Err("Recorder JSON must contain a steps array".into());
            }
            Ok(bytes)
        })
        .transpose()?;
    let mut paths = Vec::new();
    if all {
        for entry in fs::read_dir(&source)? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().starts_with("root-") {
                if entry.file_type()?.is_symlink() {
                    return Err("E2E suite evidence symlinks are refused".into());
                }
                if entry.file_type()?.is_dir() {
                    collect(
                        &entry.path(),
                        &source,
                        diagnostics,
                        include_videos,
                        &mut paths,
                    )?;
                }
            }
        }
    } else {
        collect(&source, &source, diagnostics, include_videos, &mut paths)?;
    }
    if paths.is_empty() {
        return Err("No E2E evidence files available for packaging".into());
    }
    paths.sort();
    let generated_at = chrono::Utc::now();
    let timestamp = generated_at.format("%Y%m%d-%H%M%S");
    let destination = output.map(Path::to_path_buf).unwrap_or_else(|| {
        if all {
            e2e.join(format!("zipeek-e2e-all-{timestamp}.zip"))
        } else {
            e2e.join(format!(
                "zipeek-e2e-{}-{timestamp}.zip",
                source.file_name().unwrap().to_string_lossy()
            ))
        }
    });
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let staging = destination.with_extension("zip.part");
    let result = (|| -> Result<()> {
        let mut writer = ZipWriter::new(fs::File::create(&staging)?);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        let mut cases = 0;
        let mut failed = 0;
        let mut videos = 0;
        for path in paths {
            let relative = path
                .strip_prefix(&source)?
                .to_string_lossy()
                .replace('\\', "/");
            if path.file_name().is_some_and(|name| name == "scenario.json") {
                cases += 1;
                let value: Value = serde_json::from_reader(fs::File::open(&path)?)?;
                if value["status"] == "failed" {
                    failed += 1;
                }
            }
            if path.file_name().is_some_and(|name| name == "recording.mp4") {
                videos += 1;
            }
            writer.start_file(relative, options)?;
            io::copy(&mut fs::File::open(path)?, &mut writer)?;
        }
        if let Some(bytes) = &recorder_bytes {
            writer.start_file("chrome-recorder.json", options)?;
            io::copy(&mut bytes.as_slice(), &mut writer)?;
        }
        writer.start_file("manifest.json", options)?;
        let metadata = json!({
            "format": "zipeek-e2e-evidence-v2",
            "generatedAtUtc": generated_at.to_rfc3339(),
            "scope": if all { "all-suites" } else { "single-suite" },
            "scenarioCount": cases,
            "failedCount": failed,
            "recordingCount": videos,
            "videosIncluded": include_videos,
            "diagnosticsIncluded": diagnostics,
            "recordedFlowIncluded": recorder_bytes.is_some(),
            "warning": "Review evidence before sharing. Screenshots and logs may contain private text or URLs."
        });
        let serialized = serde_json::to_vec_pretty(&metadata)?;
        io::copy(&mut serialized.as_slice(), &mut writer)?;
        writer.finish()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&staging);
    }
    result?;
    fs::rename(&staging, &destination)?;
    println!("E2E evidence: {}", destination.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{clean, eligible};
    use std::path::Path;

    #[test]
    fn export_only_permitted_evidence() {
        assert!(eligible(
            Path::new("run/scenario/steps.jsonl"),
            false,
            false
        ));
        assert!(eligible(Path::new("run/build.log"), false, false));
        assert!(eligible(
            Path::new("run/scenario/video/recording.mp4"),
            false,
            true
        ));
        assert!(!eligible(
            Path::new("run/scenario/video/recording.mp4"),
            false,
            false
        ));
        assert!(eligible(
            Path::new("run/scenario/timeline/checkpoint.png"),
            false,
            false
        ));
        assert!(!eligible(
            Path::new("run/scenario/secret.json"),
            true,
            false
        ));
        assert!(!eligible(
            Path::new("run/scenario/browser.json"),
            false,
            false
        ));
        assert!(eligible(
            Path::new("run/scenario/browser.json"),
            true,
            false
        ));
    }

    #[test]
    fn cleaning_removes_previous_runs_but_preserves_non_e2e_target_files() {
        let root = std::env::temp_dir().join(format!(
            "zipeek-e2e-clean-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let evidence = root.join("target/e2e");
        std::fs::create_dir_all(evidence.join("root-123/scene")).unwrap();
        std::fs::write(evidence.join("root-123/scene/scenario.json"), "{}").unwrap();
        std::fs::write(evidence.join("old.zip"), "old").unwrap();
        std::fs::write(root.join("target/keep.txt"), "keep").unwrap();
        assert_eq!(clean(&root).unwrap(), 2);
        assert!(evidence.is_dir());
        assert_eq!(
            std::fs::read_to_string(root.join("target/keep.txt")).unwrap(),
            "keep"
        );
        assert_eq!(clean(&root).unwrap(), 0);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cleaning_preflight_refuses_symlink_entries_without_deleting_other_runs() {
        use std::os::unix::fs::symlink;
        let root =
            std::env::temp_dir().join(format!("zipeek-e2e-entry-link-{}", std::process::id()));
        let evidence = root.join("target/e2e");
        std::fs::create_dir_all(&evidence).unwrap();
        std::fs::write(evidence.join("root-123.json"), "preserve").unwrap();
        symlink(root.join("elsewhere"), evidence.join("unsafe")).unwrap();
        assert!(clean(&root).is_err());
        assert!(evidence.join("root-123.json").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cleaning_refuses_symlinked_evidence_directory() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("zipeek-e2e-link-{}", std::process::id()));
        let outside = root.join("outside");
        std::fs::create_dir_all(root.join("target")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("preserve.txt"), "safe").unwrap();
        symlink(&outside, root.join("target/e2e")).unwrap();
        assert!(clean(&root).is_err());
        assert!(outside.join("preserve.txt").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn all_suite_archive_keeps_evidence_from_separate_runs_in_distinct_paths() {
        use super::pack;
        let root = std::env::temp_dir().join(format!(
            "zipeek-e2e-all-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let evidence = root.join("target/e2e");
        for (suite, status) in [("root-one", "passed"), ("root-two", "failed")] {
            let case = evidence.join(suite).join("sample");
            std::fs::create_dir_all(&case).unwrap();
            let report = serde_json::json!({"status": status});
            std::fs::write(
                case.join("scenario.json"),
                serde_json::to_vec(&report).unwrap(),
            )
            .unwrap();
        }
        let output = root.join("export.zip");
        pack(&root, None, Some(&output), false, None, true, false).unwrap();
        let mut archive = zip::ZipArchive::new(std::fs::File::open(output).unwrap()).unwrap();
        assert!(archive.by_name("root-one/sample/scenario.json").is_ok());
        assert!(archive.by_name("root-two/sample/scenario.json").is_ok());
        let mut manifest = String::new();
        use std::io::Read;
        archive
            .by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut manifest)
            .unwrap();
        let summary: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(summary["scope"], "all-suites");
        assert_eq!(summary["scenarioCount"], 2);
        assert_eq!(summary["failedCount"], 1);
        drop(archive);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn default_evidence_archive_has_utc_timestamp_and_contains_recordings() {
        use super::pack;
        use std::io::Read;
        let root = std::env::temp_dir().join(format!(
            "zipeek-e2e-timestamp-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let suite = root.join("target/e2e/root-one/sample");
        std::fs::create_dir_all(suite.join("video")).unwrap();
        std::fs::write(suite.join("scenario.json"), r#"{"status":"passed"}"#).unwrap();
        std::fs::write(suite.join("video/recording.mp4"), b"video-fixture").unwrap();
        let before = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        pack(&root, None, None, false, None, true, true).unwrap();
        let after = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let archives = std::fs::read_dir(root.join("target/e2e"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "zip"))
            .collect::<Vec<_>>();
        assert_eq!(archives.len(), 1);
        let name = archives[0].file_name().unwrap().to_string_lossy();
        assert!(
            name == format!("zipeek-e2e-all-{before}.zip")
                || name == format!("zipeek-e2e-all-{after}.zip"),
            "unexpected evidence archive name: {name}"
        );
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&archives[0]).unwrap()).unwrap();
        assert!(
            archive
                .by_name("root-one/sample/video/recording.mp4")
                .is_ok()
        );
        let mut manifest = String::new();
        archive
            .by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut manifest)
            .unwrap();
        let metadata: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(metadata["videosIncluded"], true);
        assert_eq!(metadata["recordingCount"], 1);
        assert!(metadata["generatedAtUtc"].as_str().is_some());
        drop(archive);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn all_suite_archive_does_not_package_unrelated_target_directories() {
        use super::pack;
        let root = std::env::temp_dir().join(format!("zipeek-e2e-scope-{}", std::process::id()));
        let evidence = root.join("target/e2e");
        let case = evidence.join("root-one/scenario");
        std::fs::create_dir_all(&case).unwrap();
        std::fs::write(case.join("summary.txt"), "valid").unwrap();
        let unrelated = evidence.join("other-folder");
        std::fs::create_dir_all(&unrelated).unwrap();
        std::fs::write(unrelated.join("failure.txt"), "must not export").unwrap();
        let output = root.join("evidence.zip");
        pack(&root, None, Some(&output), false, None, true, false).unwrap();
        let mut archive = zip::ZipArchive::new(std::fs::File::open(output).unwrap()).unwrap();
        assert!(archive.by_name("root-one/scenario/summary.txt").is_ok());
        assert!(archive.by_name("other-folder/failure.txt").is_err());
        drop(archive);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pack_skips_dist_and_recordings_by_default_but_preserves_failure_evidence() {
        use super::pack;
        use std::io::Read;
        let root = std::env::temp_dir().join(format!(
            "zipeek-e2e-small-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let suite = root.join("target/e2e/root-evidence");
        let case = suite.join("broken");
        std::fs::create_dir_all(case.join("video")).unwrap();
        std::fs::create_dir_all(suite.join("dist/vendor")).unwrap();
        std::fs::write(suite.join("dist/hanzi.duckdb"), vec![7; 1024]).unwrap();
        std::fs::write(suite.join("dist/vendor/wasm.wasm"), vec![8; 1024]).unwrap();
        std::fs::write(case.join("scenario.json"), r#"{"status":"failed"}"#).unwrap();
        std::fs::write(case.join("failure.txt"), "failed step").unwrap();
        std::fs::write(suite.join("trunk.log"), "startup log").unwrap();
        std::fs::write(case.join("video/recording.mp4"), vec![3; 1024]).unwrap();
        let output = root.join("compact.zip");
        pack(&root, None, Some(&output), false, None, true, false).unwrap();
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&output).unwrap()).unwrap();
        assert!(archive.by_name("root-evidence/broken/failure.txt").is_ok());
        assert!(archive.by_name("root-evidence/trunk.log").is_ok());
        assert!(
            archive
                .by_name("root-evidence/broken/scenario.json")
                .is_ok()
        );
        assert!(
            archive
                .by_name("root-evidence/broken/video/recording.mp4")
                .is_err()
        );
        assert!(archive.by_name("root-evidence/dist/hanzi.duckdb").is_err());
        let mut manifest = String::new();
        archive
            .by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut manifest)
            .unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&manifest).unwrap()["recordingCount"],
            0
        );
        drop(archive);
        let videos = root.join("with-videos.zip");
        pack(&root, None, Some(&videos), false, None, true, true).unwrap();
        let mut archive = zip::ZipArchive::new(std::fs::File::open(videos).unwrap()).unwrap();
        assert!(
            archive
                .by_name("root-evidence/broken/video/recording.mp4")
                .is_ok()
        );
        assert!(
            archive
                .by_name("root-evidence/dist/vendor/wasm.wasm")
                .is_err()
        );
        drop(archive);
        std::fs::remove_dir_all(root).unwrap();
    }
}
