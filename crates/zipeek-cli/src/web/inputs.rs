use super::{Asset, CHART_VERSION, DUCKDB_VERSION, ESBUILD_VERSION};
use crate::archive::Result;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn web_root(root: &Path) -> PathBuf {
    root.join("crates/zipeek-web")
}

pub(super) fn package_version(root: &Path) -> Result<String> {
    let data: Value = serde_json::from_slice(&fs::read(root.join("package.json"))?)?;
    Ok(data["version"]
        .as_str()
        .ok_or("npm package version is missing")?
        .to_string())
}

pub(super) fn check_version(root: &Path, expected: &str) -> Result<()> {
    let actual = package_version(root).unwrap_or_else(|_| "not installed".to_string());
    if actual != expected {
        return Err(format!(
            "Expected npm package {} at {}, got {actual}. Run cargo make setup",
            expected,
            root.display()
        )
        .into());
    }
    Ok(())
}

pub(super) fn module_roots(web: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let modules = web.join("node_modules");
    (
        modules.join("@duckdb/duckdb-wasm"),
        modules.join("apache-arrow"),
        modules.join("chart.js"),
        modules.join("esbuild"),
    )
}

pub(super) fn inputs(web: &Path, asset: Asset) -> Result<Vec<PathBuf>> {
    let (duckdb, arrow, chart, esbuild) = module_roots(web);
    let mut paths = vec![web.join("package-lock.json")];
    match asset {
        Asset::Duckdb => {
            check_version(&duckdb, DUCKDB_VERSION)?;
            paths.extend([
                duckdb.join("package.json"),
                duckdb.join("dist/duckdb-browser.mjs"),
                duckdb.join("dist/duckdb-base.wasm"),
                duckdb.join("dist/duckdb-browser-base.worker.js"),
                arrow.join("package.json"),
                arrow.join("Arrow.es2015.min.js"),
                arrow.join("NOTICE.txt"),
                web.join("scripts/arrow.mjs"),
            ]);
            paths.push(license(
                &duckdb,
                Some(&web.join("scripts/duckdb-LICENSE.txt")),
            )?);
            paths.push(license(&arrow, None)?);
        }
        Asset::Summary => {
            check_version(&chart, CHART_VERSION)?;
            check_version(&esbuild, ESBUILD_VERSION)?;
            let color = web.join("node_modules/@kurkle/color");
            paths.extend([
                chart.join("package.json"),
                chart.join("dist/chart.js"),
                esbuild.join("package.json"),
                color.join("package.json"),
                web.join("scripts/summary.mjs"),
                web.join("scripts/prepare-summary.cjs"),
            ]);
            for module in [&chart, &esbuild, &color] {
                paths.push(module.join("LICENSE.md"));
            }
        }
    }
    for path in &paths {
        if !path.is_file() {
            return Err(format!(
                "Missing required vendor input {}. Run cargo make setup",
                path.display()
            )
            .into());
        }
    }
    Ok(paths)
}

pub(super) fn license(module: &Path, fallback: Option<&Path>) -> Result<PathBuf> {
    if module.is_dir() {
        for entry in fs::read_dir(module)? {
            let path = entry?.path();
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_ascii_uppercase();
            if path.is_file() && (name == "LICENSE" || name.starts_with("LICENSE.")) {
                return Ok(path);
            }
        }
    }
    if let Some(path) = fallback {
        return Ok(path.to_path_buf());
    }
    Err(format!("Missing license for {}", module.display()).into())
}
