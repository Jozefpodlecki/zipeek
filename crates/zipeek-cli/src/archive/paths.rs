use super::Result;
use std::path::{Component, Path, PathBuf};

pub(super) fn ignored(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "target"
            | "node_modules"
            | "dist"
            | "dist-desktop"
            | ".desktop-assets"
            | "index.desktop.html"
            | "vendor"
            | ".unpack-backups"
            | "__pycache__"
            | ".venv"
            | ".idea"
    )
}

fn excluded_review_directory(root: &Path, path: &Path) -> Result<bool> {
    let relative = path.strip_prefix(root)?;
    Ok(relative == Path::new("assets")
        || relative == Path::new("models")
        || relative == Path::new("crates/zipeek-app/binaries")
        || relative == Path::new("crates/zipeek-web/public")
        || relative == Path::new("crates/zipeek-web/static/hanzi-strokes"))
}

fn is_duckdb_file(name: &str) -> bool {
    let name = name.to_ascii_lowercase();

    name.ends_with(".duckdb.gz")
        || name.ends_with(".duckdb.zst")
        || name.ends_with(".ddb.gz")
        || name.ends_with(".ddb.zst")
        || name.ends_with(".duckdb")
        || name.ends_with(".duckdb.wal")
        || name.ends_with(".duckdb.tmp")
        || name.ends_with(".ddb")
        || name.ends_with(".ddb.wal")
        || name.ends_with(".ddb.tmp")
}

pub(super) fn safe_relative(name: &str) -> Result<PathBuf> {
    if name.contains('\\')
        || name.contains(':')
        || name.starts_with('/')
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("invalid archive path: {name}").into());
    }

    let path = Path::new(name);

    if path
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("unsafe archive path: {name}").into());
    }

    Ok(path.to_owned())
}

pub(super) async fn collect(root: &Path, directory: &Path) -> Result<Vec<PathBuf>> {
    let mut stack = vec![directory.to_owned()];
    let mut files = Vec::new();

    while let Some(dir) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if ignored(&name) || is_duckdb_file(&name) {
                continue;
            }

            let meta = tokio::fs::symlink_metadata(&path).await?;

            if meta.file_type().is_symlink() {
                return Err(format!("symlinks are not supported: {}", path.display()).into());
            }

            if meta.is_dir() {
                if excluded_review_directory(root, &path)? {
                    continue;
                }
                stack.push(path);
            } else if meta.is_file() {
                if path.strip_prefix(root)?.components().count() == 1
                    && path.extension().is_some_and(|ext| ext == "zip")
                {
                    continue;
                }

                files.push(path);
            }
        }
    }

    files.sort();

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_paths() {
        for bad in [
            "../outside",
            "a/../b",
            "/tmp/outside",
            "C:/outside",
            "a\\b",
            "a//b",
        ] {
            assert!(safe_relative(bad).is_err(), "accepted {bad}");
        }

        assert_eq!(
            safe_relative("crates/zipeek-web/src/main.rs").unwrap(),
            PathBuf::from("crates/zipeek-web/src/main.rs")
        );
    }

    #[test]
    fn excludes_duckdb_files() {
        for name in [
            "dictionary.duckdb",
            "hanzi-20261109-194823.duckdb.gz",
            "dictionary.duckdb.zst",
            "dictionary.ddb.gz",
            "DICTIONARY.DUCKDB.GZ",
            "dictionary.duckdb.wal",
            "dictionary.duckdb.tmp",
            "dictionary.ddb",
            "dictionary.ddb.wal",
            "dictionary.ddb.tmp",
            "DICTIONARY.DUCKDB",
            "data/words.duckdb",
        ] {
            let filename = Path::new(name).file_name().unwrap().to_string_lossy();

            assert!(
                is_duckdb_file(&filename),
                "DuckDB file was not excluded: {name}"
            );
        }
    }

    #[test]
    fn review_exclusions_are_scoped_to_generated_asset_trees() {
        let root = Path::new("repo");
        for excluded in [
            "repo/assets",
            "repo/crates/zipeek-app/binaries",
            "repo/crates/zipeek-web/public",
            "repo/crates/zipeek-web/static/hanzi-strokes",
        ] {
            assert!(excluded_review_directory(root, Path::new(excluded)).unwrap());
        }
        for retained in [
            "repo/crates/zipeek-web/src/assets",
            "repo/crates/zipeek-web/static",
            "repo/crates/zipeek-web/static/other-generated",
            "repo/crates/zipeek-web/styles",
            "repo/crates/zipeek-web/scripts",
            "repo/crates/zipeek-web/src",
            "repo/crates/zipeek-e2e/public",
            "repo/docs/assets",
        ] {
            assert!(!excluded_review_directory(root, Path::new(retained)).unwrap());
        }
    }

    #[tokio::test]
    async fn collect_skips_hanzi_strokes_but_keeps_other_static_files() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-hanzi-pack-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let strokes = root.join("crates/zipeek-web/static/hanzi-strokes");
        tokio::fs::create_dir_all(&strokes).await?;
        tokio::fs::write(strokes.join("characters.json"), "{}").await?;
        tokio::fs::write(strokes.join("manifest.json"), "{}").await?;
        tokio::fs::write(root.join("crates/zipeek-web/static/keep.txt"), "keep").await?;
        let files = collect(&root, &root).await?;
        let relative: Vec<_> = files
            .iter()
            .map(|file| file.strip_prefix(&root).unwrap().to_owned())
            .collect();
        assert_eq!(
            relative,
            vec![PathBuf::from("crates/zipeek-web/static/keep.txt")]
        );
        tokio::fs::remove_dir_all(&root).await?;
        Ok(())
    }

    #[test]
    fn preserves_other_database_files() {
        for name in [
            "data.db",
            "data.sqlite",
            "data.sqlite3",
            "database.rs",
            "schema.sql",
            "migration.sql",
            "dictionary.json",
            "dictionary.parquet",
            "database.toml",
            "README.md",
        ] {
            assert!(
                !is_duckdb_file(name),
                "non-DuckDB file was excluded: {name}"
            );
        }
    }
}
