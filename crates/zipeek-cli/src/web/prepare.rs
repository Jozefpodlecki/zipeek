use super::Asset;
use super::fingerprint::{stamp, verify, verify_all};
use super::inputs::{inputs, license, module_roots, web_root};
use crate::archive::Result;
use std::{fs, path::Path, process::Command};

pub(super) fn rewrite_arrow(source: &str) -> Result<String> {
    let candidates = [
        "from\"apache-arrow\"",
        "from'apache-arrow'",
        "from \"apache-arrow\"",
        "from 'apache-arrow'",
    ];
    let mut changed = source.to_owned();
    let mut count = 0;
    for candidate in candidates {
        let matched = changed.matches(candidate).count();
        if matched > 0 {
            changed = changed.replace(candidate, "from\"./arrow.mjs\"");
            count += matched;
        }
    }
    if count == 0 {
        return Err("No apache-arrow module imports found in pinned DuckDB module".into());
    }
    if candidates
        .iter()
        .any(|candidate| changed.contains(candidate))
    {
        return Err("Unresolved apache-arrow module import after rewriting".into());
    }
    Ok(changed)
}

pub(super) fn prepare_duckdb(root: &Path, output: &Path) -> Result<()> {
    let web = web_root(root);
    let (duckdb, arrow, _, _) = module_roots(&web);
    fs::create_dir_all(output)?;
    for name in ["duckdb-base.wasm", "duckdb-browser-base.worker.js"] {
        fs::copy(duckdb.join("dist").join(name), output.join(name))?;
    }
    let module = fs::read_to_string(duckdb.join("dist/duckdb-browser.mjs"))?;
    fs::write(output.join("duckdb-browser.mjs"), rewrite_arrow(&module)?)?;
    fs::copy(arrow.join("Arrow.es2015.min.js"), output.join("arrow.js"))?;
    fs::copy(web.join("scripts/arrow.mjs"), output.join("arrow.mjs"))?;
    fs::copy(
        license(&duckdb, Some(&web.join("scripts/duckdb-LICENSE.txt")))?,
        output.join("DUCKDB-LICENSE.txt"),
    )?;
    fs::copy(license(&arrow, None)?, output.join("ARROW-LICENSE.txt"))?;
    fs::copy(arrow.join("NOTICE.txt"), output.join("ARROW-NOTICE.txt"))?;
    Ok(())
}

pub(super) fn prepare_summary(root: &Path, output: &Path) -> Result<()> {
    let web = web_root(root);
    let status = Command::new("node")
        .arg("scripts/prepare-summary.cjs")
        .arg(output)
        .current_dir(web)
        .status()?;
    if !status.success() {
        return Err("Summary asset bundling failed".into());
    }
    Ok(())
}

pub(super) fn publish(stage: &Path, destination: &Path) -> Result<()> {
    let previous = destination.with_extension("zipeek-previous");
    if previous.exists() {
        fs::remove_dir_all(&previous)?;
    }
    let had_previous = destination.exists();
    if had_previous {
        fs::rename(destination, &previous)?;
    }
    if let Err(error) = fs::rename(stage, destination) {
        if had_previous {
            fs::rename(&previous, destination)?;
        }
        return Err(error.into());
    }
    if had_previous {
        fs::remove_dir_all(&previous)?;
    }
    Ok(())
}

pub fn prepare(root: &Path, asset: Asset) -> Result<()> {
    if verify(root, asset).is_ok() {
        println!("{} assets already current", asset.name());
        return Ok(());
    }
    let web = web_root(root);
    inputs(&web, asset)?;
    let output = web.join("vendor").join(asset.name());
    fs::create_dir_all(output.parent().ok_or("Missing vendor directory")?)?;
    let stage = output.with_extension("zipeek-staging");
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    fs::create_dir_all(&stage)?;
    let result = match asset {
        Asset::Duckdb => prepare_duckdb(root, &stage),
        Asset::Summary => prepare_summary(root, &stage),
    };
    if let Err(error) = result {
        fs::remove_dir_all(&stage)?;
        return Err(error);
    }
    for name in asset.files() {
        if !stage.join(name).is_file() {
            fs::remove_dir_all(&stage)?;
            return Err(format!("Prepared {} is missing {name}", asset.name()).into());
        }
    }
    publish(&stage, &output)?;
    stamp(root, asset)?;
    println!("Prepared {} assets", asset.name());
    Ok(())
}

pub fn prepare_all(root: &Path) -> Result<()> {
    for asset in [Asset::Duckdb, Asset::Summary] {
        prepare(root, asset)?;
    }
    verify_all(root)?;
    super::backgrounds::prepare(root)?;
    super::backgrounds::verify(root)
}
