use super::inputs::{inputs, web_root};
use super::{Asset, PREPARE_VERSION};
use crate::archive::Result;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

pub(super) fn digest(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let size = file.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hash.update(&buffer[..size]);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub(super) fn fingerprint(web: &Path, asset: Asset) -> Result<Value> {
    let mut items = Vec::new();
    for path in inputs(web, asset)? {
        items.push(json!([
            path.strip_prefix(web)?.to_string_lossy(),
            digest(&path)?
        ]));
    }
    Ok(json!({"format": PREPARE_VERSION, "asset": asset.name(), "inputs": items}))
}

pub(super) fn stamp_path(web: &Path, asset: Asset) -> PathBuf {
    web.join("vendor")
        .join(asset.name())
        .join(".zipeek-prepared.json")
}

pub fn verify(root: &Path, asset: Asset) -> Result<()> {
    let web = web_root(root);
    let stamp = stamp_path(&web, asset);
    let data: Value = serde_json::from_slice(&fs::read(&stamp).map_err(|_| {
        format!(
            "Missing prepared {} assets ({}). Run cargo make setup",
            asset.name(),
            stamp.display()
        )
    })?)?;
    let expected = fingerprint(&web, asset)?;
    if data["source"] != expected {
        return Err(format!(
            "Stale {} assets; run cargo make prepare-assets",
            asset.name()
        )
        .into());
    }
    let dir = web.join("vendor").join(asset.name());
    for name in asset.files() {
        let path = dir.join(name);
        if !path.is_file() {
            return Err(format!(
                "Missing prepared asset {}; run cargo make prepare-assets",
                path.display()
            )
            .into());
        }
        let hash = digest(&path)?;
        if data["outputs"][*name].as_str() != Some(hash.as_str()) {
            return Err(format!(
                "Missing or changed prepared asset {}; run cargo make prepare-assets",
                path.display()
            )
            .into());
        }
    }
    Ok(())
}

pub fn verify_all(root: &Path) -> Result<()> {
    for asset in [Asset::Duckdb, Asset::Summary] {
        verify(root, asset)?;
    }
    println!("Web assets verified (DuckDB-WASM, Arrow, summary)");
    Ok(())
}

pub(super) fn stamp(root: &Path, asset: Asset) -> Result<()> {
    let web = web_root(root);
    let output = web.join("vendor").join(asset.name());
    let mut outputs = serde_json::Map::new();
    for name in asset.files() {
        outputs.insert((*name).to_string(), json!(digest(&output.join(name))?));
    }
    let stamp = json!({"source": fingerprint(&web, asset)?, "outputs": outputs});
    fs::write(stamp_path(&web, asset), serde_json::to_vec_pretty(&stamp)?)?;
    verify(root, asset)
}
