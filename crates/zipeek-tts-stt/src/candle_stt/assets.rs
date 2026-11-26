use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenAsrAssets {
    pub checkpoint: PathBuf,
    pub config: PathBuf,
    pub generation_config: PathBuf,
    pub preprocessor_config: PathBuf,
    pub vocabulary: PathBuf,
    pub merges: PathBuf,
}

impl QwenAsrAssets {
    pub fn from_checkpoint(checkpoint: &Path) -> Self {
        let root = checkpoint.parent().unwrap_or_else(|| Path::new("."));
        Self {
            checkpoint: checkpoint.to_owned(),
            config: root.join("config.json"),
            generation_config: root.join("generation_config.json"),
            preprocessor_config: root.join("preprocessor_config.json"),
            vocabulary: root.join("vocab.json"),
            merges: root.join("merges.txt"),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.checkpoint.file_name().and_then(|name| name.to_str()) != Some("model.safetensors") {
            return Err("Qwen3-ASR checkpoint must be model.safetensors");
        }
        for path in [
            &self.checkpoint,
            &self.config,
            &self.generation_config,
            &self.preprocessor_config,
            &self.vocabulary,
            &self.merges,
        ] {
            let metadata =
                fs::metadata(path).map_err(|_| "Qwen3-ASR model asset missing or unreadable")?;
            if !metadata.is_file() || metadata.len() == 0 {
                return Err("Qwen3-ASR model asset empty or not a regular file");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_published_qwen_asr_files() {
        let directory =
            std::env::temp_dir().join(format!("zipeek-qwen-asr-assets-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let assets = QwenAsrAssets::from_checkpoint(&directory.join("model.safetensors"));
        assert!(assets.validate().is_err());
        for file in [
            &assets.checkpoint,
            &assets.config,
            &assets.generation_config,
            &assets.preprocessor_config,
            &assets.vocabulary,
            &assets.merges,
        ] {
            fs::write(file, b"placeholder").unwrap();
        }
        assert!(assets.validate().is_ok());
        fs::write(&assets.merges, b"").unwrap();
        assert!(assets.validate().is_err());
        let _ = fs::remove_dir_all(directory);
    }
}
