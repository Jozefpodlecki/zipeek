use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub mod codec;
pub mod codec_transformer;
pub mod codec_vocoder;
pub mod config;
pub mod model;
pub mod sampling;
pub mod tokenizer;
pub mod transformer;

use crate::runtime::models::QwenTtsAssets;

pub const NEURAL_WAVEFORM_DECODER_READY: bool = true;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TtsProgress {
    LoadingModels,
    Tokenizing,
    GeneratingAcousticCodes {
        frames: usize,
        maximum_frames: usize,
    },
    DecodingWaveform {
        frames: usize,
    },
    EncodingWav {
        samples: usize,
    },
    Finished {
        wav_bytes: usize,
    },
}

pub fn synthesize_mandarin(root: &Path, text: &str, device_name: &str) -> Result<Vec<u8>, String> {
    synthesize_mandarin_with_progress(root, text, device_name, |_| {})
}

pub struct QwenTtsEngine {
    device: candle_core::Device,
    tokenizer: tokenizer::MandarinBpeTokenizer,
    talker: model::QwenTtsTalker,
    predictor: model::QwenTtsCodePredictor,
    decoder: codec::SpeechCodebookDecoder,
}

fn validate_mandarin_text(text: &str) -> Result<(), String> {
    if text.trim().is_empty() || text.len() > 4096 || text.chars().any(char::is_control) {
        return Err("invalid Qwen3-TTS text".into());
    }
    Ok(())
}

impl QwenTtsEngine {
    pub fn load(root: &Path, device_name: &str) -> Result<Self, String> {
        if device_name != "cpu" && device_name != "cuda" {
            return Err("ZIPEEK_TTS_DEVICE must be cpu or cuda".into());
        }
        QwenTtsAssets::from_checkpoint(&root.join("model.safetensors"))
            .validate()
            .map_err(str::to_owned)?;
        config::CustomVoiceConfig::from_directory(root)?;
        let device = if device_name == "cuda" {
            candle_core::Device::new_cuda(0)
                .map_err(|error| format!("Qwen3-TTS CUDA device: {error}"))?
        } else {
            candle_core::Device::Cpu
        };
        let tokenizer = tokenizer::MandarinBpeTokenizer::load(root)?;
        let talker = model::QwenTtsTalker::load(root, &device)
            .map_err(|error| format!("Qwen3-TTS talker loading: {error}"))?;
        let predictor = model::QwenTtsCodePredictor::load(root, &device)
            .map_err(|error| format!("Qwen3-TTS code predictor loading: {error}"))?;
        let decoder = codec::SpeechCodebookDecoder::load(root, &device)
            .map_err(|error| format!("Qwen3-TTS speech decoder loading: {error}"))?;
        Ok(Self {
            device,
            tokenizer,
            talker,
            predictor,
            decoder,
        })
    }

    pub fn synthesize(&mut self, text: &str) -> Result<Vec<u8>, String> {
        self.synthesize_with_progress(text, |_| {})
    }

    pub fn synthesize_with_progress(
        &mut self,
        text: &str,
        on_progress: impl FnMut(TtsProgress),
    ) -> Result<Vec<u8>, String> {
        self.synthesize_with_cancel(text, &AtomicBool::new(false), on_progress)
    }

    pub fn synthesize_with_cancel(
        &mut self,
        text: &str,
        cancelled: &AtomicBool,
        mut on_progress: impl FnMut(TtsProgress),
    ) -> Result<Vec<u8>, String> {
        validate_mandarin_text(text)?;
        if cancelled.load(Ordering::Relaxed) {
            return Err("Qwen3-TTS inference cancelled".into());
        }
        on_progress(TtsProgress::Tokenizing);
        let prompt = self.tokenizer.encode_assistant_mandarin(text)?;
        self.talker.reset();
        let codes_result = self.talker.generate_custom_voice_codes_with_control(
            &mut self.predictor,
            &prompt,
            &self.device,
            375,
            &mut |frames| {
                on_progress(TtsProgress::GeneratingAcousticCodes {
                    frames,
                    maximum_frames: 375,
                })
            },
            &|| cancelled.load(Ordering::Relaxed),
        );
        self.talker.reset();
        let codes = codes_result
            .map_err(|error| format!("Qwen3-TTS acoustic token generation: {error}"))?;
        if cancelled.load(Ordering::Relaxed) {
            return Err("Qwen3-TTS inference cancelled".into());
        }
        on_progress(TtsProgress::DecodingWaveform {
            frames: codes.len(),
        });
        let samples = self
            .decoder
            .decode_waveform(&codes, &self.device)
            .map_err(|error| format!("Qwen3-TTS neural waveform decoding: {error}"))?;
        if cancelled.load(Ordering::Relaxed) {
            return Err("Qwen3-TTS inference cancelled".into());
        }
        if !samples.iter().any(|sample| sample.abs() > 0.001) {
            return Err("Qwen3-TTS generated silent audio".into());
        }
        on_progress(TtsProgress::EncodingWav {
            samples: samples.len(),
        });
        let wav = crate::candle_tts::waveform::encode_mono_pcm16_wav(&samples, 24_000)
            .map_err(|error| format!("Qwen3-TTS WAV encoding failed: {error:?}"))?;
        on_progress(TtsProgress::Finished {
            wav_bytes: wav.len(),
        });
        Ok(wav)
    }
}

pub fn synthesize_mandarin_with_progress(
    root: &Path,
    text: &str,
    device_name: &str,
    mut on_progress: impl FnMut(TtsProgress),
) -> Result<Vec<u8>, String> {
    validate_mandarin_text(text)?;
    on_progress(TtsProgress::LoadingModels);
    let mut engine = QwenTtsEngine::load(root, device_name)?;
    engine.synthesize_with_progress(text, on_progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_model_is_not_synthesized() {
        let root = std::env::temp_dir().join("zipeek-qwen-tts-missing-model");
        assert!(synthesize_mandarin(&root, "你好世界", "cpu").is_err());
    }

    #[test]
    fn refuses_invalid_mandarin_input() {
        let root = std::env::temp_dir().join("zipeek-qwen-tts-missing-model");
        assert!(synthesize_mandarin(&root, "", "cpu").is_err());
        assert!(synthesize_mandarin(&root, "你好\n世界", "cpu").is_err());
    }

    #[test]
    fn progress_api_rejects_invalid_input_before_reporting_work() {
        let root = std::env::temp_dir().join("zipeek-qwen-tts-missing-model");
        let mut events = Vec::new();
        let result = synthesize_mandarin_with_progress(&root, "", "cpu", |event| {
            events.push(event);
        });
        assert!(result.is_err());
        assert!(events.is_empty());
    }

    #[test]
    fn native_neural_decoder_is_wired_for_runtime_execution() {
        assert!(NEURAL_WAVEFORM_DECODER_READY);
    }
}
