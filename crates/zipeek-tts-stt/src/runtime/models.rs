use crate::{candle_stt::QwenAsrAssets, candle_tts::KokoroAssets};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    QwenAsr,
    KokoroChinese,
    QwenTts,
}

impl ModelKind {
    pub fn repository(self) -> &'static str {
        match self {
            Self::QwenAsr => "Qwen/Qwen3-ASR-0.6B",
            Self::KokoroChinese => "hexgrad/Kokoro-82M-v1.1-zh",
            Self::QwenTts => "Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice",
        }
    }

    pub fn checkpoint_name(self) -> &'static str {
        match self {
            Self::QwenAsr | Self::QwenTts => "model.safetensors",
            Self::KokoroChinese => "kokoro-v1_1-zh.pth",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenTtsAssets {
    pub root: PathBuf,
}

impl QwenTtsAssets {
    pub fn from_checkpoint(checkpoint: &Path) -> Self {
        Self {
            root: checkpoint
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_owned(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        for name in [
            "model.safetensors",
            "config.json",
            "generation_config.json",
            "vocab.json",
            "merges.txt",
            "tokenizer_config.json",
            "preprocessor_config.json",
        ] {
            let metadata = std::fs::metadata(self.root.join(name))
                .map_err(|_| "Qwen3-TTS model asset missing")?;
            if !metadata.is_file() || metadata.len() == 0 {
                return Err("Qwen3-TTS model asset empty or not a regular file");
            }
        }
        let codec = self.root.join("speech_tokenizer");
        for name in [
            "model.safetensors",
            "config.json",
            "preprocessor_config.json",
        ] {
            let metadata = std::fs::metadata(codec.join(name))
                .map_err(|_| "Qwen3-TTS speech-tokenizer file missing")?;
            if !metadata.is_file() || metadata.len() == 0 {
                return Err("Qwen3-TTS speech-tokenizer file empty or not a regular file");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelAssets {
    QwenAsr(QwenAsrAssets),
    KokoroChinese(KokoroAssets),
    QwenTts(QwenTtsAssets),
}

impl ModelAssets {
    pub fn from_checkpoint(kind: ModelKind, checkpoint: &Path) -> Self {
        match kind {
            ModelKind::QwenAsr => Self::QwenAsr(QwenAsrAssets::from_checkpoint(checkpoint)),
            ModelKind::KokoroChinese => {
                Self::KokoroChinese(KokoroAssets::from_checkpoint(checkpoint))
            }
            ModelKind::QwenTts => Self::QwenTts(QwenTtsAssets::from_checkpoint(checkpoint)),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::QwenAsr(assets) => assets.validate(),
            Self::KokoroChinese(assets) => assets.validate(),
            Self::QwenTts(assets) => assets.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_registry_has_distinct_sources_and_formats() {
        assert_ne!(
            ModelKind::QwenAsr.repository(),
            ModelKind::QwenTts.repository()
        );
        assert_eq!(
            ModelKind::KokoroChinese.checkpoint_name(),
            "kokoro-v1_1-zh.pth"
        );
        assert_eq!(ModelKind::QwenAsr.checkpoint_name(), "model.safetensors");
    }

    #[test]
    fn qwen_tts_requires_actual_codec_checkpoint_and_metadata() {
        let root =
            std::env::temp_dir().join(format!("zipeek-qwen-tts-assets-{}", std::process::id()));
        std::fs::create_dir_all(root.join("speech_tokenizer")).unwrap();
        let assets = QwenTtsAssets::from_checkpoint(&root.join("model.safetensors"));
        for name in [
            "model.safetensors",
            "config.json",
            "generation_config.json",
            "vocab.json",
            "merges.txt",
            "tokenizer_config.json",
            "preprocessor_config.json",
        ] {
            std::fs::write(root.join(name), b"asset").unwrap();
        }
        assert!(assets.validate().is_err());
        for name in [
            "model.safetensors",
            "config.json",
            "preprocessor_config.json",
        ] {
            std::fs::write(root.join("speech_tokenizer").join(name), b"asset").unwrap();
        }
        assert!(assets.validate().is_ok());
        std::fs::write(root.join("speech_tokenizer/model.safetensors"), b"").unwrap();
        assert!(assets.validate().is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
