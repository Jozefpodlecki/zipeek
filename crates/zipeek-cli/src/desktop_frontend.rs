use std::{fs, path::Path, process::Command};

use crate::{archive::Result, web};

const STATIC_LINK: &str =
    "<link data-trunk rel=\"copy-dir\" href=\"static\" data-target-path=\".\" />";
const VENDOR_LINK: &str = "<link data-trunk rel=\"copy-dir\" href=\"vendor\" />";

fn desktop_html(source: &str) -> Result<String> {
    if source.matches("<head>").count() != 1
        || source.matches(STATIC_LINK).count() != 1
        || source.matches(VENDOR_LINK).count() != 1
    {
        return Err(
            "Desktop frontend requires one head element, one static and one vendor copy directive"
                .into(),
        );
    }
    let mut html = String::with_capacity(source.len());
    let mut remaining = source;
    while let Some(start) = remaining.find("<link") {
        html.push_str(&remaining[..start]);
        remaining = &remaining[start..];
        let end = remaining
            .find('>')
            .ok_or("Unterminated HTML link element")?
            + 1;
        let link = &remaining[..end];
        if !link.contains("hanzi.manifest.json") && !link.contains(".duckdb") {
            html.push_str(link);
        }
        remaining = &remaining[end..];
    }
    html.push_str(remaining);
    Ok(html.replace(STATIC_LINK, "<link data-trunk rel=\"copy-dir\" href=\".desktop-assets/static\" data-target-path=\".\" />")
        .replace(VENDOR_LINK, "<link data-trunk rel=\"copy-dir\" href=\".desktop-assets/vendor\" data-target-path=\"vendor\" />")
        .replacen("<head>", "<head>\n<meta name=\"zipeek-platform\" content=\"desktop\">", 1))
}

fn browser_dictionary_asset(name: &str) -> bool {
    name.ends_with(".duckdb")
        || name.ends_with(".duckdb.gz")
        || name.ends_with(".duckdb.zst")
        || matches!(
            name,
            "hanzi.manifest.json" | "hanzi.diagnostics.json" | "INPUTS.toml"
        )
}

fn copy_tree(source: &Path, destination: &Path, filter_dictionary: bool) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if filter_dictionary && browser_dictionary_asset(&name.to_string_lossy()) {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(
                format!("Desktop asset symlink refused: {}", entry.path().display()).into(),
            );
        }
        let output = destination.join(&name);
        if kind.is_dir() {
            copy_tree(&entry.path(), &output, filter_dictionary)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), output)?;
        } else {
            return Err("Desktop assets must be regular files or directories".into());
        }
    }
    Ok(())
}

pub fn prepare(root: &Path) -> Result<()> {
    web::verify_desktop(root)?;
    let web = root.join("crates/zipeek-web");
    let html = desktop_html(&fs::read_to_string(web.join("index.html"))?)?;
    let assets = web.join(".desktop-assets");
    if assets.exists() {
        if fs::symlink_metadata(&assets)?.file_type().is_symlink()
            || !assets.join(".zipeek-generated").is_file()
            || !assets.canonicalize()?.starts_with(root.canonicalize()?)
        {
            return Err("Refusing to replace an unowned desktop asset directory".into());
        }
        fs::remove_dir_all(&assets)?;
    }
    fs::create_dir_all(&assets)?;
    fs::write(
        assets.join(".zipeek-generated"),
        "zipeek desktop frontend assets\n",
    )?;
    copy_tree(&web.join("static"), &assets.join("static"), true)?;
    copy_tree(
        &web.join("vendor/summary"),
        &assets.join("vendor/summary"),
        false,
    )?;
    fs::create_dir_all(assets.join("static/notices"))?;
    fs::copy(
        web.join("scripts/duckdb-LICENSE.txt"),
        assets.join("static/notices/DUCKDB-LICENSE.txt"),
    )?;
    fs::write(web.join("index.desktop.html"), html)?;
    Ok(())
}

pub fn run(root: &Path, serve: bool, release: bool, prepare_only: bool) -> Result<()> {
    prepare(root)?;
    if prepare_only {
        return Ok(());
    }
    let mut command = Command::new("trunk");
    command
        .current_dir(root.join("crates/zipeek-web"))
        .arg(if serve { "serve" } else { "build" })
        .arg("index.desktop.html")
        .args([
            "--dist",
            "dist-desktop",
            "--public-url",
            "/",
            "--html-output",
            "index.html",
        ]);
    if serve {
        command.args(["--port", "1421"]);
    }
    if release {
        command.arg("--release");
    }
    if !command.status()?.success() {
        return Err("Desktop frontend Trunk command failed".into());
    }
    verify_output(&root.join("crates/zipeek-web/dist-desktop"))
}

fn verify_output(directory: &Path) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if browser_dictionary_asset(&entry.file_name().to_string_lossy())
            || matches!(
                entry.file_name().to_str(),
                Some(
                    "duckdb-base.wasm"
                        | "duckdb-browser.mjs"
                        | "duckdb-browser-base.worker.js"
                        | "arrow.mjs"
                        | "arrow.js"
                )
            )
        {
            return Err(format!(
                "Browser database asset in desktop output: {}",
                entry.path().display()
            )
            .into());
        }
        if entry.file_type()?.is_dir() {
            if entry.file_name() == "duckdb" {
                return Err(
                    "DuckDB-WASM assets must not be bundled in the desktop frontend".into(),
                );
            }
            verify_output(&entry.path())?;
        }
    }
    let index = directory.join("index.html");
    if index.is_file() && fs::read_to_string(index)?.contains(".duckdb") {
        return Err("Desktop HTML contains a browser dictionary reference".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_html_removes_dictionary_preloads_and_preserves_shared_assets() {
        let source = format!(
            "<head>{STATIC_LINK}{VENDOR_LINK}<link\n rel=\"preload\" href=\"hanzi.manifest.json\"><link rel=\"preload\" href=\"hanzi-20261124-214934.duckdb.gz\"><link rel=\"stylesheet\" href=\"styles.css\"></head>"
        );
        let html = desktop_html(&source).unwrap();
        assert!(!html.contains("hanzi.manifest.json"));
        assert!(!html.contains(".duckdb"));
        assert!(html.contains(".desktop-assets/vendor"));
        assert!(html.contains("styles.css"));
        assert!(html.contains("content=\"desktop\""));
        assert!(source.contains("hanzi-20261124-214934.duckdb.gz"));
    }

    #[test]
    fn changed_copy_contract_is_rejected() {
        assert!(desktop_html("<head></head>").is_err());
    }

    #[test]
    fn staging_excludes_database_assets_and_keeps_notices_and_other_workers() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-desktop-assets-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let source = root.join("source");
        fs::create_dir_all(source.join("notices"))?;
        fs::create_dir_all(source.join("pedagogy-worker"))?;
        for name in [
            "hanzi-20261124-214934.duckdb",
            "hanzi-20261124-214934.duckdb.gz",
            "hanzi.manifest.json",
            "hanzi.diagnostics.json",
            "INPUTS.toml",
            "notices/CC-BY-SA-4.0.txt",
            "pedagogy-worker/pedagogy_wasm_bg.wasm",
            "ATTRIBUTION.md",
        ] {
            fs::write(source.join(name), name)?;
        }
        let output = root.join("output");
        copy_tree(&source, &output, true)?;
        verify_output(&output)?;
        assert!(output.join("notices/CC-BY-SA-4.0.txt").is_file());
        assert!(
            output
                .join("pedagogy-worker/pedagogy_wasm_bg.wasm")
                .is_file()
        );
        assert!(output.join("ATTRIBUTION.md").is_file());
        assert!(!output.join("hanzi.manifest.json").exists());
        fs::create_dir_all(output.join("vendor/duckdb"))?;
        assert!(verify_output(&output).is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
