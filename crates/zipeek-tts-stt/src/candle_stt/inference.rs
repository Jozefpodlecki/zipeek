use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;

use super::{
    decoder::QwenAsrTextDecoder,
    encoder::QwenAsrAudioEncoder,
    tensor::pcm_to_qwen_audio_input,
    tokenizer::{AUDIO_PAD, END_OF_MESSAGE, END_OF_TEXT, QwenVocabulary, asr_prompt},
    weights::probe_qwen_asr_checkpoint,
};

pub const MAX_GENERATED_TOKENS: usize = 192;

pub struct QwenAsrEngine {
    device: Device,
    vocabulary: QwenVocabulary,
    audio_encoder: QwenAsrAudioEncoder,
    decoder: QwenAsrTextDecoder,
}

impl QwenAsrEngine {
    pub fn load(checkpoint: &Path, device: &Device) -> Result<Self, String> {
        probe_qwen_asr_checkpoint(checkpoint, device)?;
        let assets = super::QwenAsrAssets::from_checkpoint(checkpoint);
        let vocabulary = QwenVocabulary::load(&assets.vocabulary)?;
        let files = [checkpoint];
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&files, DType::F32, device) }
            .map_err(|error| format!("Qwen3-ASR checkpoint load failed: {error}"))?;
        let audio_encoder = QwenAsrAudioEncoder::load(vb.clone())
            .map_err(|error| format!("Qwen3-ASR audio encoder load failed: {error}"))?;
        let decoder = QwenAsrTextDecoder::load(vb)
            .map_err(|error| format!("Qwen3-ASR text decoder load failed: {error}"))?;
        Ok(Self {
            device: device.clone(),
            vocabulary,
            audio_encoder,
            decoder,
        })
    }

    pub fn transcribe(&mut self, samples: &[f32]) -> Result<String, String> {
        self.transcribe_with_cancel(samples, &AtomicBool::new(false))
    }

    pub fn transcribe_with_cancel(
        &mut self,
        samples: &[f32],
        cancelled: &AtomicBool,
    ) -> Result<String, String> {
        self.decoder.reset();
        let result = self.transcribe_inner(samples, cancelled);
        self.decoder.reset();
        result
    }

    fn transcribe_inner(
        &mut self,
        samples: &[f32],
        cancelled: &AtomicBool,
    ) -> Result<String, String> {
        if samples.is_empty() || samples.len() > 16_000 * 30 {
            return Err("invalid Qwen3-ASR PCM duration".into());
        }
        if cancelled.load(Ordering::Relaxed) {
            return Err("Qwen3-ASR inference cancelled".into());
        }
        let device = &self.device;
        let audio_encoder = &self.audio_encoder;
        let decoder = &mut self.decoder;
        let vocabulary = &self.vocabulary;
        let audio_input = pcm_to_qwen_audio_input(samples, device)
            .map_err(|error| format!("Qwen3-ASR Mel frontend failed: {error}"))?;
        let audio_embeddings = audio_encoder
            .forward(&audio_input)
            .map_err(|error| format!("Qwen3-ASR audio inference failed: {error}"))?;
        if cancelled.load(Ordering::Relaxed) {
            return Err("Qwen3-ASR inference cancelled".into());
        }
        let (audio_count, audio_width) = audio_embeddings
            .dims2()
            .map_err(|error| format!("invalid audio encoder output: {error}"))?;
        if audio_width != super::decoder::TEXT_WIDTH {
            return Err("Qwen3-ASR audio and text widths disagree".into());
        }
        let prompt = asr_prompt(audio_count)?;
        let embeddings = decoder
            .embed(&prompt, device)
            .map_err(|error| format!("Qwen3-ASR prompt embeddings failed: {error}"))?;
        let mut chunks = Vec::with_capacity(3);
        let prefix = prompt
            .iter()
            .position(|token| *token == AUDIO_PAD)
            .ok_or("missing Qwen3-ASR audio prompt placeholder")?;
        chunks.push(embeddings.narrow(0, 0, prefix).map_err(|e| e.to_string())?);
        chunks.push(audio_embeddings);
        chunks.push(
            embeddings
                .narrow(0, prefix + audio_count, prompt.len() - prefix - audio_count)
                .map_err(|e| e.to_string())?,
        );
        let refs = chunks.iter().collect::<Vec<&Tensor>>();
        let prefill = Tensor::cat(&refs, 0).map_err(|e| e.to_string())?;
        let mut next = decoder
            .next_token(&prefill)
            .map_err(|error| format!("Qwen3-ASR prefill failed: {error}"))?;
        let mut generated = Vec::with_capacity(MAX_GENERATED_TOKENS);
        for _ in 0..MAX_GENERATED_TOKENS {
            if cancelled.load(Ordering::Relaxed) {
                return Err("Qwen3-ASR inference cancelled".into());
            }
            generated.push(next);
            if next == END_OF_MESSAGE || next == END_OF_TEXT {
                break;
            }
            let input = decoder
                .embed(&[next], device)
                .map_err(|error| error.to_string())?;
            next = decoder
                .next_token(&input)
                .map_err(|error| format!("Qwen3-ASR generation failed: {error}"))?;
        }
        if !generated
            .last()
            .is_some_and(|token| *token == END_OF_MESSAGE || *token == END_OF_TEXT)
        {
            return Err("Qwen3-ASR generation exceeded its token limit without EOS".into());
        }
        vocabulary.decode_asr(&generated)
    }
}

pub fn transcribe_pcm(
    checkpoint: &Path,
    samples: &[f32],
    device: &Device,
) -> Result<String, String> {
    QwenAsrEngine::load(checkpoint, device)?.transcribe(samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generation_has_bounded_token_budget() {
        assert!((64..=512).contains(&MAX_GENERATED_TOKENS));
    }
    #[test]
    #[ignore = "requires the local Qwen3-ASR checkpoint and reference speech"]
    fn real_checkpoint_transcribes_reference_audio() {
        let checkpoint = std::env::var_os("ZIPEEK_CANDLE_STT_MODEL")
            .expect("set ZIPEEK_CANDLE_STT_MODEL to Qwen3-ASR model.safetensors");
        let wav = include_bytes!("../../tests/fixtures/mandarin_nihao_shijie.wav");
        let samples = crate::speech::decode_pcm16_wav(wav).unwrap();
        let result = transcribe_pcm(Path::new(&checkpoint), &samples, &Device::Cpu).unwrap();
        assert!(!result.is_empty());
        println!("Qwen3-ASR transcription: {result}");
    }
}
