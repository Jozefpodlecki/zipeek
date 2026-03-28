use std::{fs, path::Path};
use anyhow::Result;
  
pub fn clear_dir(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub fn folder_has_files<P: AsRef<Path>>(path: P) -> Result<bool> {
    let path = path.as_ref();

    if !path.exists() || !path.is_dir() {
        return Ok(false);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            return Ok(true);
        }
    }

    Ok(false)
}