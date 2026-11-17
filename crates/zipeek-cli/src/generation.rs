use std::{
    fs,
    path::{Component, Path},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};

use crate::{archive::Result, database};

#[cfg(windows)]
fn cached_duckdb_directory(root: &Path) -> Result<Option<std::path::PathBuf>> {
    if let Some(configured) = std::env::var_os("DUCKDB_LIB_DIR") {
        let directory = std::path::PathBuf::from(configured);
        if !directory.join("duckdb.dll").is_file() {
            return Err(format!(
                "DUCKDB_LIB_DIR does not contain duckdb.dll: {}",
                directory.display()
            )
            .into());
        }
        return Ok(Some(directory));
    }

    let cache = root.join("target/duckdb-download");
    if !cache.is_dir() {
        return Ok(None);
    }
    let mut directories = Vec::new();
    let mut pending = vec![(cache, 0_usize)];
    while let Some((directory, depth)) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_file() && entry.file_name() == "duckdb.dll" {
                directories.push(
                    entry
                        .path()
                        .parent()
                        .ok_or("Missing DLL parent directory")?
                        .to_path_buf(),
                );
            } else if file_type.is_dir() && depth < 5 {
                pending.push((entry.path(), depth + 1));
            }
        }
    }
    directories.sort();
    directories.dedup();
    match directories.len() {
        0 => Ok(None),
        1 => Ok(directories.pop()),
        _ => Err(format!(
            "Found multiple cached duckdb.dll locations; set DUCKDB_LIB_DIR to the directory used for this generator build: {}",
            directories.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
        ).into()),
    }
}

#[cfg(windows)]
fn configure_duckdb_runtime(command: &mut Command, root: &Path) -> Result<()> {
    if let Some(directory) = cached_duckdb_directory(root)? {
        let existing = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = vec![directory];
        paths.extend(std::env::split_paths(&existing));
        command.env("PATH", std::env::join_paths(paths)?);
    }
    Ok(())
}

pub fn generate(root: &Path, selected_classification_cache: bool) -> Result<()> {
    let workflow = root.join("target/db-workflow");
    let generated = workflow.join("generated");
    fs::create_dir_all(&generated)?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let output = generated.join(format!("candidate-{timestamp}"));
    let binary = root.join("target/release").join(if cfg!(windows) {
        "zipeek-db-gen.exe"
    } else {
        "zipeek-db-gen"
    });
    if !binary.is_file() {
        return Err(format!(
            "Missing {}. Build zipeek-db-gen in release mode first.",
            binary.display()
        )
        .into());
    }
    let mut command = Command::new(&binary);
    command.arg("generate").arg("--output").arg(&output);
    if selected_classification_cache {
        let selection: Value = serde_json::from_reader(fs::File::open(
            workflow.join("selected-classification.json"),
        )?)?;
        if selection["version"] != 1 {
            return Err("Invalid classification cache selection version".into());
        }
        let relative = selection["cache"]
            .as_str()
            .ok_or("Missing selected classification cache")?;
        let path = Path::new(relative);
        if !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(
                "Classification cache path must be relative and cannot traverse directories".into(),
            );
        }
        let selected = workflow.join(path).canonicalize()?;
        if !selected.starts_with(workflow.canonicalize()?) || !selected.is_file() {
            return Err(
                "Selected classification cache is not inside the workflow directory".into(),
            );
        }
        command.arg("--classification-cache").arg(selected);
    }
    #[cfg(windows)]
    configure_duckdb_runtime(&mut command, root)?;
    let status = command.current_dir(root).status()?;
    if !status.success() {
        #[cfg(windows)]
        if status.code() == Some(0xC0000135_u32 as i32) {
            return Err(format!(
                "Dictionary generator could not start: Windows STATUS_DLL_NOT_FOUND (0xc0000135). A native DLL needed by {} was not found. Check its dependencies with 'dumpbin /dependents {}', and verify the matching DuckDB v2.0.0 duckdb.dll is available in target/duckdb-download or set DUCKDB_LIB_DIR to its directory before retrying. Installed bundle unchanged.",
                binary.display(), binary.display()
            ).into());
        }
        return Err(
            format!("Dictionary generation failed ({status}); installed bundle unchanged").into(),
        );
    }
    database::validate_candidate(root, &output)?;
    let relative = output
        .strip_prefix(root)?
        .to_string_lossy()
        .replace('\\', "/");
    let pointer = workflow.join("latest.json");
    let temporary = workflow.join(format!("latest-{timestamp}.json"));
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&json!({"directory": relative}))?,
    )?;
    fs::rename(&temporary, &pointer)?;
    println!("Verified candidate: {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::{Component, Path};

    #[test]
    fn selection_paths_must_have_only_normal_components() {
        for path in ["../escape.json", "/absolute.json", "folder/../escape.json"] {
            assert!(
                !Path::new(path)
                    .components()
                    .all(|part| matches!(part, Component::Normal(_)))
            );
        }
        assert!(
            Path::new("classifications/validated.json")
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        );
    }

    #[cfg(windows)]
    #[test]
    fn missing_cached_library_does_not_change_runtime_path() {
        let root =
            std::env::temp_dir().join(format!("zipeek-empty-library-{}", std::process::id()));
        assert!(super::cached_duckdb_directory(&root).unwrap().is_none());
    }
}
