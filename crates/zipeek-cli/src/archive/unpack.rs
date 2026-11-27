use super::{
    Result,
    paths::{collect, ignored, safe_relative},
};
use std::{
    collections::HashSet,
    fs::File,
    io::{self, Read},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use zip::ZipArchive;

const MAX_ARCHIVE_ENTRIES: usize = 40_000;
const MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone, Copy)]
struct ArchiveLimits {
    entries: usize,
    entry_bytes: u64,
    total_bytes: u64,
}

const SOURCE_LIMITS: ArchiveLimits = ArchiveLimits {
    entries: MAX_ARCHIVE_ENTRIES,
    entry_bytes: MAX_ENTRY_BYTES,
    total_bytes: MAX_ARCHIVE_BYTES,
};

pub async fn unpack(root: &Path, archive: &Path) -> Result<()> {
    let target = root.join("target/zipeek-unpack");
    let stage = target.join("stage");
    let rollback_area = target.join("rollback");
    if tokio::fs::try_exists(&rollback_area).await? {
        return Err(format!(
            "Previous source installation needs manual recovery from {}",
            rollback_area.display()
        )
        .into());
    }
    if tokio::fs::try_exists(&stage).await? {
        tokio::fs::remove_dir_all(&stage).await?;
    }
    tokio::fs::create_dir_all(&stage).await?;
    let input = archive.to_owned();
    let output = stage.clone();

    let extraction = tokio::task::spawn_blocking(move || extract(&input, &output)).await?;
    if let Err(error) = extraction {
        tokio::fs::remove_dir_all(&stage).await?;
        return Err(error);
    }
    if let Err(error) = validate_source_stage(root, &stage) {
        tokio::fs::remove_dir_all(&stage).await?;
        return Err(error);
    }
    let result = restore(root, &stage, archive).await;
    if result.is_err() && tokio::fs::try_exists(&rollback_area).await? {
        return result;
    }
    if let Err(error) = tokio::fs::remove_dir_all(&stage).await {
        eprintln!("Source install succeeded, but unpack stage cleanup failed: {error}");
    }
    result
}

fn extract(archive: &Path, stage: &Path) -> Result<()> {
    extract_with_limits(archive, stage, SOURCE_LIMITS)
}

fn extract_with_limits(archive: &Path, stage: &Path, limits: ArchiveLimits) -> Result<()> {
    let mut zip = ZipArchive::new(File::open(archive)?)?;
    if zip.len() > limits.entries {
        return Err("archive contains too many entries".into());
    }
    let mut total_bytes = 0_u64;
    let mut seen = HashSet::new();
    let mut normalized_paths = HashSet::new();
    let mut created_directories = HashSet::new();
    for index in 0..zip.len() {
        let entry = zip.by_index(index)?;

        if entry.is_dir() {
            continue;
        }

        if entry.size() > limits.entry_bytes {
            return Err("archive entry exceeds the source extraction size limit".into());
        }

        total_bytes = total_bytes
            .checked_add(entry.size())
            .ok_or("archive extraction size overflow")?;

        if total_bytes > limits.total_bytes {
            return Err("archive exceeds the total source extraction size limit".into());
        }
        let name = entry.name()?;
        let relative = safe_relative(name.as_ref())?;
        if !seen.insert(relative.clone()) {
            return Err("duplicate archive entry refused".into());
        }
        let normalized = relative.to_string_lossy().to_lowercase();
        if !normalized_paths.insert(normalized) {
            return Err("case-insensitive archive path collision refused".into());
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err("archive symlink refused".into());
        }
        if relative
            .components()
            .any(|part| ignored(&part.as_os_str().to_string_lossy()))
        {
            return Err("archive contains excluded build paths".into());
        }
        if relative.starts_with("crates/zipeek-web/static/hanzi-strokes") {
            continue;
        }
        let path = stage.join(relative);
        let parent = path.parent().ok_or("invalid destination")?;
        if created_directories.insert(parent.to_owned()) {
            std::fs::create_dir_all(parent)?;
        }
        let expected_size = entry.size();
        let mut destination = File::create(path)?;
        let copied = io::copy(&mut entry.take(limits.entry_bytes + 1), &mut destination)?;
        if copied > limits.entry_bytes || copied != expected_size {
            return Err("archive entry expanded beyond its declared source size".into());
        }
    }
    Ok(())
}

fn validate_source_stage(root: &Path, stage: &Path) -> Result<()> {
    if !root.join("Cargo.toml").exists() {
        return Ok(());
    }
    for required in [
        "Cargo.toml",
        "Cargo.lock",
        "Makefile.toml",
        "crates/zipeek-cli/Cargo.toml",
        "crates/zipeek-web/Cargo.toml",
    ] {
        if !stage.join(required).is_file() {
            return Err(format!(
                "Archive is not a complete Zipeek source release: missing {required}"
            )
            .into());
        }
    }
    Ok(())
}

fn protected_generated_asset(relative: &Path) -> bool {
    relative == Path::new("crates/zipeek-web/index.html")
        || (relative.starts_with("crates/zipeek-web/static")
            && !relative.starts_with("crates/zipeek-web/static/courses")
            && !relative.starts_with("crates/zipeek-web/static/brand")
            && !relative.starts_with("crates/zipeek-web/static/practice"))
}

async fn restore(root: &Path, stage: &Path, archive: &Path) -> Result<()> {
    let rollback_area = root.join("target/zipeek-unpack/rollback");
    if tokio::fs::try_exists(&rollback_area).await? {
        return Err(format!(
            "Previous source installation needs manual recovery from {}",
            rollback_area.display()
        )
        .into());
    }
    let backup = rollback_area.as_path();
    let mut managed = vec![
        "docs".to_owned(),
        "scripts".to_owned(),
        "crates/zipeek-web/scripts".to_owned(),
        "crates/zipeek-web/static/courses".to_owned(),
        "crates/zipeek-web/static/practice".to_owned(),
        "crates/zipeek-web/static/brand".to_owned(),
    ];
    for base in [root.join("crates"), stage.join("crates")] {
        if !tokio::fs::try_exists(&base).await? {
            continue;
        }
        let mut entries = tokio::fs::read_dir(base).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                for directory in ["src", "js", "tests"] {
                    managed.push(format!("crates/{name}/{directory}"));
                }
            }
        }
    }
    managed.sort();
    managed.dedup();
    for relative in &managed {
        let source = stage.join(relative);
        if tokio::fs::try_exists(&source).await? && !tokio::fs::metadata(&source).await?.is_dir() {
            return Err(format!(
                "Archive contains a file where a source directory is required: {relative}"
            )
            .into());
        }
    }
    tokio::fs::create_dir_all(backup).await?;
    let mut replaced_directories = Vec::new();
    let mut copied_files = Vec::new();
    let installation = async {
        for relative in &managed {
            let source = stage.join(relative);
            let dest = root.join(relative);
            let had_original = tokio::fs::try_exists(&dest).await?;
            if had_original {
                let saved = backup.join(relative);
                tokio::fs::create_dir_all(saved.parent().ok_or("invalid backup path")?).await?;
                tokio::fs::rename(&dest, saved).await?;
            }
            replaced_directories.push((relative.clone(), had_original));
            if tokio::fs::try_exists(&source).await? {
                tokio::fs::create_dir_all(dest.parent().ok_or("invalid target path")?).await?;
                tokio::fs::rename(source, dest).await?;
            }
        }

        for file in collect(stage, stage).await? {
            install_source_file(root, stage, &backup, &file, &mut copied_files).await?;
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    }
    .await;
    if let Err(error) = installation {
        let recovery = rollback(root, backup, &copied_files, &replaced_directories).await;
        if let Err(recovery_error) = recovery {
            return Err(format!(
                "Installation failed: {error}; rollback failed: {recovery_error}. Recovery files retained at {}",
                backup.display()
            )
            .into());
        }
        tokio::fs::remove_dir_all(backup).await?;
        return Err(format!("Installation failed and was rolled back: {error}").into());
    }
    tokio::fs::remove_dir_all(backup).await?;
    println!("Unpacked archive: {}", archive.display());
    Ok(())
}

async fn install_source_file(
    root: &Path,
    stage: &Path,
    backup: &Path,
    file: &Path,
    installed: &mut Vec<(std::path::PathBuf, bool)>,
) -> Result<()> {
    let relative = file.strip_prefix(stage)?.to_owned();
    if protected_generated_asset(&relative) {
        return Ok(());
    }
    let destination = root.join(&relative);
    let had_original = tokio::fs::try_exists(&destination).await?;
    if had_original {
        let saved = backup.join(&relative);
        tokio::fs::create_dir_all(saved.parent().ok_or("invalid backup path")?).await?;
        tokio::fs::rename(&destination, saved).await?;
    }
    installed.push((relative, had_original));
    tokio::fs::create_dir_all(destination.parent().ok_or("invalid target path")?).await?;
    tokio::fs::rename(file, destination).await?;
    Ok(())
}

async fn rollback(
    root: &Path,
    backup: &Path,
    copied_files: &[(std::path::PathBuf, bool)],
    replaced_directories: &[(String, bool)],
) -> Result<()> {
    for (relative, had_original) in copied_files.iter().rev() {
        let destination = root.join(relative);
        if tokio::fs::try_exists(&destination).await? {
            tokio::fs::remove_file(&destination).await?;
        }
        if *had_original {
            tokio::fs::rename(backup.join(relative), destination).await?;
        }
    }
    for (relative, had_original) in replaced_directories.iter().rev() {
        let destination = root.join(relative);
        if tokio::fs::try_exists(&destination).await? {
            tokio::fs::remove_dir_all(&destination).await?;
        }
        if *had_original {
            tokio::fs::create_dir_all(destination.parent().ok_or("invalid restore path")?).await?;
            tokio::fs::rename(backup.join(relative), destination).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_limits_reject_excessive_entry_size_count_and_total() -> Result<()> {
        use std::io::Write;
        use zip::{ZipWriter, write::SimpleFileOptions};
        let parent = std::env::temp_dir().join(format!(
            "zipeek-archive-limit-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        std::fs::create_dir_all(&parent)?;
        let archive = parent.join("source.zip");
        {
            let mut writer = ZipWriter::new(File::create(&archive)?);
            for name in ["first.txt", "second.txt"] {
                writer.start_file(name, SimpleFileOptions::default())?;
                writer.write_all(&[b'x'; 16])?;
            }
            writer.finish()?;
        }
        let stage = parent.join("stage");
        let defaults = ArchiveLimits {
            entries: 2,
            entry_bytes: 16,
            total_bytes: 32,
        };
        assert!(extract_with_limits(&archive, &stage, defaults).is_ok());
        let too_many = ArchiveLimits {
            entries: 1,
            ..defaults
        };
        assert!(extract_with_limits(&archive, &stage, too_many).is_err());
        let too_large = ArchiveLimits {
            entry_bytes: 15,
            ..defaults
        };
        assert!(extract_with_limits(&archive, &stage, too_large).is_err());
        let too_much = ArchiveLimits {
            total_bytes: 31,
            ..defaults
        };
        assert!(extract_with_limits(&archive, &stage, too_much).is_err());
        std::fs::remove_dir_all(&parent)?;
        Ok(())
    }

    #[test]
    fn archive_rejects_case_colliding_source_files() -> Result<()> {
        use std::io::Write;
        use zip::{ZipWriter, write::SimpleFileOptions};
        let parent = std::env::temp_dir().join(format!(
            "zipeek-case-collision-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        std::fs::create_dir_all(&parent)?;
        let archive = parent.join("source.zip");
        {
            let mut writer = ZipWriter::new(File::create(&archive)?);
            for name in ["docs/Readme.md", "docs/README.md"] {
                writer.start_file(name, SimpleFileOptions::default())?;
                writer.write_all(b"fixture")?;
            }
            writer.finish()?;
        }
        let result = extract(&archive, &parent.join("stage"));
        assert!(result.is_err());
        std::fs::remove_dir_all(parent)?;
        Ok(())
    }

    #[tokio::test]
    async fn interrupted_move_restore_uses_saved_originals() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-rollback-move-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let backup = root.join("target/zipeek-unpack/rollback");
        tokio::fs::create_dir_all(&backup).await?;
        tokio::fs::write(root.join("README.md"), "replacement").await?;
        tokio::fs::write(root.join("NEW.md"), "new").await?;
        tokio::fs::write(backup.join("README.md"), "original").await?;
        rollback(
            &root,
            &backup,
            &[("README.md".into(), true), ("NEW.md".into(), false)],
            &[],
        )
        .await?;
        assert_eq!(
            tokio::fs::read_to_string(root.join("README.md")).await?,
            "original"
        );
        assert!(!root.join("NEW.md").exists());
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[tokio::test]
    async fn incomplete_rollback_refuses_reinstall() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-recovery-guard-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let rollback = root.join("target/zipeek-unpack/rollback");
        tokio::fs::create_dir_all(&rollback).await?;
        tokio::fs::write(rollback.join("README.md"), "recoverable").await?;
        let stage = root.join("target/zipeek-unpack/stage");
        tokio::fs::create_dir_all(&stage).await?;
        tokio::fs::write(stage.join("README.md"), "new").await?;
        assert!(
            restore(&root, &stage, Path::new("fixture.zip"))
                .await
                .is_err()
        );
        assert_eq!(
            tokio::fs::read_to_string(rollback.join("README.md")).await?,
            "recoverable"
        );
        assert!(!root.join("README.md").exists());
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[tokio::test]
    async fn staged_files_install_by_move_and_remove_temporary_rollback() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-archive-move-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let stage = root.join("target/zipeek-unpack/stage");
        tokio::fs::create_dir_all(&stage).await?;
        tokio::fs::write(root.join("README.md"), "previous").await?;
        tokio::fs::write(stage.join("README.md"), "replacement").await?;
        tokio::fs::write(stage.join("NEW.md"), "new").await?;
        restore(&root, &stage, Path::new("fixture.zip")).await?;
        assert_eq!(
            tokio::fs::read_to_string(root.join("README.md")).await?,
            "replacement"
        );
        assert_eq!(tokio::fs::read_to_string(root.join("NEW.md")).await?, "new");
        assert!(!stage.join("README.md").exists());
        assert!(!stage.join("NEW.md").exists());
        assert!(!tokio::fs::try_exists(root.join("target/zipeek-unpack/rollback")).await?);
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[test]
    fn generated_web_assets_are_not_restored() {
        assert!(protected_generated_asset(Path::new(
            "crates/zipeek-web/index.html"
        )));
        assert!(protected_generated_asset(Path::new(
            "crates/zipeek-web/static/hanzi.manifest.json"
        )));
        assert!(protected_generated_asset(Path::new(
            "crates/zipeek-web/static/hanzi.duckdb.gz"
        )));
        assert!(protected_generated_asset(Path::new(
            "crates/zipeek-web/static/hanzi-strokes/characters.json"
        )));
        assert!(!protected_generated_asset(Path::new(
            "crates/zipeek-web/static/courses/grammar-autumn-bridge.png"
        )));
        assert!(!protected_generated_asset(Path::new(
            "crates/zipeek-web/static/courses/pinyin-landscape.png"
        )));
        assert!(!protected_generated_asset(Path::new(
            "crates/zipeek-web/static/brand/zipeek-mark.png"
        )));
        assert!(!protected_generated_asset(Path::new(
            "crates/zipeek-web/static/practice/dragon.webp"
        )));
        assert!(protected_generated_asset(Path::new(
            "crates/zipeek-web/static/docs/index.json"
        )));
        assert!(!protected_generated_asset(Path::new(
            "crates/zipeek-web/src/main.rs"
        )));
    }

    #[tokio::test]
    async fn restoring_source_preserves_regenerated_web_assets() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-preserve-assets-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let stage = root.join("target/fixture-stage");
        for directory in [&root, &stage] {
            tokio::fs::create_dir_all(directory.join("crates/zipeek-web/static")).await?;
            tokio::fs::create_dir_all(directory.join("crates/zipeek-web/src")).await?;
        }
        for file in [
            "index.html",
            "static/hanzi.manifest.json",
            "static/hanzi.duckdb.gz",
        ] {
            let path = format!("crates/zipeek-web/{file}");
            tokio::fs::write(root.join(&path), "locally-generated").await?;
            tokio::fs::write(stage.join(&path), "archived").await?;
        }
        for directory in [&root, &stage] {
            tokio::fs::create_dir_all(directory.join("crates/zipeek-web/static/courses")).await?;
        }
        tokio::fs::write(
            root.join("crates/zipeek-web/static/courses/grammar-autumn-bridge.png"),
            "old-artwork",
        )
        .await?;
        tokio::fs::write(
            stage.join("crates/zipeek-web/static/courses/grammar-autumn-bridge.png"),
            "new-artwork",
        )
        .await?;
        tokio::fs::write(
            stage.join("crates/zipeek-web/static/courses/pinyin-landscape.png"),
            "new-pinyin-artwork",
        )
        .await?;
        tokio::fs::write(
            stage.join("crates/zipeek-web/static/courses/vocabulary-peony-garden.png"),
            "new-vocabulary-artwork",
        )
        .await?;
        tokio::fs::create_dir_all(stage.join("crates/zipeek-web/static/practice")).await?;
        tokio::fs::write(
            stage.join("crates/zipeek-web/static/practice/dragon.webp"),
            "new-dragon-artwork",
        )
        .await?;
        tokio::fs::write(
            stage.join("crates/zipeek-web/src/main.rs"),
            "updated-source",
        )
        .await?;
        tokio::fs::write(
            root.join("crates/zipeek-web/static/courses/old-unused-image.png"),
            "obsolete",
        )
        .await?;
        tokio::fs::write(
            root.join("crates/zipeek-web/static/practice/old-creature.webp"),
            "obsolete",
        )
        .await?;
        restore(&root, &stage, Path::new("fixture.zip")).await?;
        assert!(
            !root
                .join("crates/zipeek-web/static/courses/old-unused-image.png")
                .exists()
        );
        assert!(
            !root
                .join("crates/zipeek-web/static/practice/old-creature.webp")
                .exists()
        );
        for file in [
            "index.html",
            "static/hanzi.manifest.json",
            "static/hanzi.duckdb.gz",
        ] {
            let path = format!("crates/zipeek-web/{file}");
            assert_eq!(
                tokio::fs::read_to_string(root.join(path)).await?,
                "locally-generated"
            );
        }
        assert_eq!(
            tokio::fs::read_to_string(root.join("crates/zipeek-web/src/main.rs")).await?,
            "updated-source"
        );
        assert_eq!(
            tokio::fs::read_to_string(
                root.join("crates/zipeek-web/static/courses/grammar-autumn-bridge.png")
            )
            .await?,
            "new-artwork"
        );
        assert_eq!(
            tokio::fs::read_to_string(
                root.join("crates/zipeek-web/static/courses/pinyin-landscape.png")
            )
            .await?,
            "new-pinyin-artwork"
        );

        assert_eq!(
            tokio::fs::read_to_string(
                root.join("crates/zipeek-web/static/courses/vocabulary-peony-garden.png")
            )
            .await?,
            "new-vocabulary-artwork"
        );

        assert_eq!(
            tokio::fs::read_to_string(root.join("crates/zipeek-web/static/practice/dragon.webp"))
                .await?,
            "new-dragon-artwork"
        );

        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[test]
    fn legacy_archive_hanzi_strokes_are_not_extracted() -> Result<()> {
        let parent = std::env::temp_dir().join(format!(
            "zipeek-hanzi-unpack-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        std::fs::create_dir_all(&parent)?;
        let archive = parent.join("legacy.zip");
        {
            use zip::{ZipWriter, write::SimpleFileOptions};
            let mut writer = ZipWriter::new(File::create(&archive)?);
            writer.start_file(
                "crates/zipeek-web/static/hanzi-strokes/characters.json",
                SimpleFileOptions::default(),
            )?;
            std::io::Write::write_all(&mut writer, b"archived")?;
            writer.start_file("README.md", SimpleFileOptions::default())?;
            std::io::Write::write_all(&mut writer, b"retained")?;
            writer.finish()?;
        }
        let stage = parent.join("stage");
        extract(&archive, &stage)?;
        assert!(
            !stage
                .join("crates/zipeek-web/static/hanzi-strokes")
                .exists()
        );
        assert_eq!(
            std::fs::read_to_string(stage.join("README.md"))?,
            "retained"
        );
        std::fs::remove_dir_all(parent)?;
        Ok(())
    }

    #[test]
    fn complete_source_release_requires_workspace_manifests() -> Result<()> {
        let parent = std::env::temp_dir().join(format!(
            "zipeek-release-validation-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let root = parent.join("project");
        let stage = parent.join("stage");
        std::fs::create_dir_all(&root)?;
        std::fs::create_dir_all(&stage)?;
        std::fs::write(root.join("Cargo.toml"), "[workspace]")?;
        assert!(validate_source_stage(&root, &stage).is_err());
        for required in [
            "Cargo.toml",
            "Cargo.lock",
            "Makefile.toml",
            "crates/zipeek-cli/Cargo.toml",
            "crates/zipeek-web/Cargo.toml",
        ] {
            let path = stage.join(required);
            std::fs::create_dir_all(path.parent().ok_or("invalid fixture path")?)?;
            std::fs::write(path, "fixture")?;
        }
        validate_source_stage(&root, &stage)?;
        std::fs::remove_dir_all(parent)?;
        Ok(())
    }

    #[tokio::test]
    async fn rollback_restores_replaced_files_and_directories() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-rollback-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let backup = root.join("target/zipeek-unpack/rollback");
        tokio::fs::create_dir_all(root.join("docs")).await?;
        tokio::fs::create_dir_all(backup.join("docs")).await?;
        tokio::fs::write(root.join("docs/changed.md"), "replacement").await?;
        tokio::fs::write(backup.join("docs/original.md"), "original").await?;
        tokio::fs::write(root.join("README.md"), "replacement").await?;
        tokio::fs::write(backup.join("README.md"), "original").await?;
        rollback(
            &root,
            &backup,
            &[(std::path::PathBuf::from("README.md"), true)],
            &[("docs".to_owned(), true)],
        )
        .await?;
        assert_eq!(
            tokio::fs::read_to_string(root.join("README.md")).await?,
            "original"
        );
        assert_eq!(
            tokio::fs::read_to_string(root.join("docs/original.md")).await?,
            "original"
        );
        assert!(!tokio::fs::try_exists(root.join("docs/changed.md")).await?);
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[tokio::test]
    async fn incomplete_release_rejects_before_modifying_checkout() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-incomplete-release-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        tokio::fs::create_dir_all(&root).await?;
        tokio::fs::write(root.join("Cargo.toml"), "[workspace]").await?;
        tokio::fs::write(root.join("README.md"), "old version").await?;
        let archive = root.join("target/incomplete.zip");
        crate::archive::pack(&root, &archive).await?;
        tokio::fs::write(root.join("README.md"), "local changes").await?;

        assert!(unpack(&root, &archive).await.is_err());
        assert_eq!(
            tokio::fs::read_to_string(root.join("README.md")).await?,
            "local changes"
        );
        assert!(!tokio::fs::try_exists(root.join("target/zipeek-unpack/rollback")).await?);
        assert!(!tokio::fs::try_exists(root.join("target/zipeek-unpack/stage")).await?);
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[tokio::test]
    async fn malformed_zip_does_not_modify_checkout() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-bad-zip-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        tokio::fs::create_dir_all(&root).await?;
        tokio::fs::write(root.join("README.md"), "original").await?;
        let archive = root.join("invalid.zip");
        tokio::fs::write(&archive, "not a ZIP archive").await?;
        assert!(unpack(&root, &archive).await.is_err());
        assert_eq!(
            tokio::fs::read_to_string(root.join("README.md")).await?,
            "original"
        );
        assert!(!tokio::fs::try_exists(root.join("target/zipeek-unpack/rollback")).await?);
        assert!(!tokio::fs::try_exists(root.join("target/zipeek-unpack/stage")).await?);
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[tokio::test]
    async fn old_vendor_preparation_script_is_removed() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-cli-cleanup-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let web_scripts = root.join("crates/zipeek-web/scripts");
        tokio::fs::create_dir_all(&web_scripts).await?;
        tokio::fs::write(web_scripts.join("arrow.mjs"), "export const ready = true;").await?;
        let zip = root.join("target/clean.zip");
        crate::archive::pack(&root, &zip).await?;
        tokio::fs::write(web_scripts.join("prepare-duckdb.cjs"), "outdated").await?;
        unpack(&root, &zip).await?;
        assert!(!tokio::fs::try_exists(web_scripts.join("prepare-duckdb.cjs")).await?);
        assert!(tokio::fs::try_exists(web_scripts.join("arrow.mjs")).await?);
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[tokio::test]
    async fn removed_crate_js_and_tests_are_cleared() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-cli-interop-cleanup-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        ));
        let crate_root = root.join("crates/demo");
        for (directory, filename) in [
            ("src", "lib.rs"),
            ("js", "bridge.js"),
            ("tests", "http_integration.rs"),
        ] {
            let path = crate_root.join(directory);
            tokio::fs::create_dir_all(&path).await?;
            tokio::fs::write(path.join(filename), "current").await?;
        }
        let archive = root.join("target/managed.zip");
        crate::archive::pack(&root, &archive).await?;

        tokio::fs::write(crate_root.join("src/obsolete.rs"), "stale").await?;
        tokio::fs::write(crate_root.join("js/transport.js"), "stale").await?;
        tokio::fs::write(crate_root.join("tests/obsolete.rs"), "stale").await?;
        tokio::fs::write(crate_root.join("local-notes.txt"), "preserve").await?;
        unpack(&root, &archive).await?;

        for (directory, obsolete) in [
            ("src", "obsolete.rs"),
            ("js", "transport.js"),
            ("tests", "obsolete.rs"),
        ] {
            assert!(!tokio::fs::try_exists(crate_root.join(directory).join(obsolete)).await?);
        }
        assert_eq!(
            tokio::fs::read_to_string(crate_root.join("local-notes.txt")).await?,
            "preserve",
        );
        assert!(!tokio::fs::try_exists(root.join("target/zipeek-unpack/rollback")).await?);
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }

    #[tokio::test]
    async fn archive_restores_managed_files_without_persistent_backup() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-cli-test-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let source = root.join("crates/demo/src");
        tokio::fs::create_dir_all(&source).await?;
        tokio::fs::write(source.join("lib.rs"), "original").await?;
        tokio::fs::write(source.join("stale.rs"), "stale").await?;
        let zip = root.join("target/check.zip");
        crate::archive::pack(&root, &zip).await?;
        tokio::fs::remove_file(source.join("stale.rs")).await?;
        tokio::fs::write(source.join("lib.rs"), "edited").await?;
        unpack(&root, &zip).await?;
        assert_eq!(
            tokio::fs::read_to_string(source.join("lib.rs")).await?,
            "original"
        );
        assert_eq!(
            tokio::fs::read_to_string(source.join("stale.rs")).await?,
            "stale"
        );
        assert!(!tokio::fs::try_exists(root.join("target/zipeek-unpack/rollback")).await?);
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }
}
