use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KokoroAssets {
    pub checkpoint: PathBuf,
    pub config: PathBuf,
    pub voice: PathBuf,
}

impl KokoroAssets {
    pub fn from_checkpoint(checkpoint: &Path) -> Self {
        let root = checkpoint.parent().unwrap_or_else(|| Path::new("."));
        Self {
            checkpoint: checkpoint.to_owned(),
            config: root.join("config.json"),
            voice: root.join("voices").join("zf_007.pt"),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.checkpoint.file_name().and_then(|name| name.to_str()) != Some("kokoro-v1_1-zh.pth")
        {
            return Err("Chinese Kokoro checkpoint must be kokoro-v1_1-zh.pth");
        }
        for path in [&self.checkpoint, &self.config, &self.voice] {
            let metadata = fs::metadata(path)
                .map_err(|_| "Chinese Kokoro model asset missing or unreadable")?;
            if !metadata.is_file() || metadata.len() == 0 {
                return Err("Chinese Kokoro asset empty or not a regular file");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_actual_chinese_kokoro_distribution() {
        let directory =
            std::env::temp_dir().join(format!("zipeek-kokoro-assets-{}", std::process::id()));
        fs::create_dir_all(directory.join("voices")).unwrap();
        let assets = KokoroAssets::from_checkpoint(&directory.join("kokoro-v1_1-zh.pth"));
        assert!(assets.validate().is_err());
        fs::write(&assets.checkpoint, b"checkpoint").unwrap();
        fs::write(&assets.config, b"{}").unwrap();
        fs::write(&assets.voice, b"voice").unwrap();
        assert!(assets.validate().is_ok());
        fs::write(&assets.voice, b"").unwrap();
        assert!(assets.validate().is_err());
        let _ = fs::remove_dir_all(directory);
    }
}
