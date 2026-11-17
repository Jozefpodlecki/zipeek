use crate::archive::Result;
use std::{path::Path, process::Command};

pub fn install(root: &Path) -> Result<()> {
    let web = root.join("crates/zipeek-web");
    let status = if cfg!(windows) {
        Command::new("cmd")
            .args(["/d", "/s", "/c", "npm ci --no-audit --no-fund"])
            .current_dir(web)
            .status()?
    } else {
        Command::new("npm")
            .args(["ci", "--no-audit", "--no-fund"])
            .current_dir(web)
            .status()?
    };
    if !status.success() {
        return Err("npm ci failed; check web package-lock.json and npm output".into());
    }
    Ok(())
}
