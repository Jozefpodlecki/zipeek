use std::path::Path;

use candle_core::{D, DType, Device, Module, Result, Tensor};
use candle_nn::{Embedding, Linear, VarBuilder};

use super::{
    config::CustomVoiceConfig,
    sampling::{GenerationSampling, SamplingRng, sample_acoustic_code, sample_residual_code},
    transformer::CausalTransformer,
};

pub struct QwenTtsTalker {
    pub config: CustomVoiceConfig,
    text_embedding: Embedding,
    codec_embedding: Embedding,
    transformer: CausalTransformer,
    codec_head: Linear,
    text_fc1: Linear,
    text_fc2: Linear,
}

#[derive(Clone, Debug)]
pub struct AcousticFrameDiagnostic {
    pub frame: usize,
    pub winning_code: u32,
    pub winning_score: f32,
    pub eos_score: f32,
    pub selected_eos: bool,
    pub selected_code: Option<u32>,
}

pub struct QwenTtsCodePredictor {
    transformer: CausalTransformer,
    embeddings: Vec<Embedding>,
    heads: Vec<Linear>,
    codebook_size: usize,
}

fn checkpoint_var_builder<'a>(checkpoint: &'a Path, device: &'a Device) -> Result<VarBuilder<'a>> {
    let files = [checkpoint];
    unsafe { VarBuilder::from_mmaped_safetensors(&files, DType::F32, device) }
}

#[cfg(test)]
fn select_acoustic_token(
    logits: &Tensor,
    codebook_size: usize,
    eos_token: usize,
) -> Result<Option<u32>> {
    let (batch, vocabulary) = logits.dims2()?;
    if batch != 1 || codebook_size == 0 || codebook_size > vocabulary || eos_token >= vocabulary {
        candle_core::bail!("invalid Qwen3-TTS generation vocabulary")
    }
    let scores = logits.to_dtype(DType::F32)?;
    let candidates = scores.narrow(1, 0, codebook_size)?;
    let winning_code = candidates.argmax(D::Minus1)?.to_vec1::<u32>()?[0];
    let winning_score = candidates.max(D::Minus1)?.to_vec1::<f32>()?[0];
    let eos_score = scores
        .narrow(1, eos_token, 1)?
        .flatten_all()?
        .to_vec1::<f32>()?[0];
    if !winning_score.is_finite() || !eos_score.is_finite() {
        candle_core::bail!("nonfinite Qwen3-TTS logits")
    }
    Ok((eos_score <= winning_score).then_some(winning_code))
}

impl QwenTtsTalker {
    pub fn load(root: &Path, device: &Device) -> Result<Self> {
        let config = CustomVoiceConfig::from_directory(root).map_err(candle_core::Error::Msg)?;
        let checkpoint = root.join("model.safetensors");
        let vb = checkpoint_var_builder(&checkpoint, device)?;
        Self::load_weights(config, vb)
    }

    pub fn load_weights(config: CustomVoiceConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let talker = vb.pp("talker");
        let model = talker.pp("model");
        let text_embedding = candle_nn::embedding(
            config.text_vocab_size,
            config.text_hidden_size,
            model.pp("text_embedding"),
        )?;
        let codec_embedding = candle_nn::embedding(
            config.talker.vocab_size,
            config.talker.hidden_size,
            model.pp("codec_embedding"),
        )?;
        let transformer = CausalTransformer::load(&config.talker, model)?;
        let text_projection = talker.pp("text_projection");
        let text_fc1 = candle_nn::linear(
            config.text_hidden_size,
            config.text_hidden_size,
            text_projection.pp("linear_fc1"),
        )?;
        let text_fc2 = candle_nn::linear(
            config.text_hidden_size,
            config.talker.hidden_size,
            text_projection.pp("linear_fc2"),
        )?;
        let codec_head = candle_nn::linear_no_bias(
            config.talker.hidden_size,
            config.talker.vocab_size,
            talker.pp("codec_head"),
        )?;
        Ok(Self {
            config,
            text_embedding,
            codec_embedding,
            transformer,
            codec_head,
            text_fc1,
            text_fc2,
        })
    }

    pub fn embed_text(&self, ids: &[u32], device: &Device) -> Result<Tensor> {
        if ids.is_empty() {
            candle_core::bail!("Qwen3-TTS cannot embed an empty text prompt")
        }
        self.text_embedding.forward(&Tensor::new(ids, device)?)
    }

    pub fn project_text(&self, embeddings: &Tensor) -> Result<Tensor> {
        let activated = candle_nn::Activation::Silu.forward(&self.text_fc1.forward(embeddings)?)?;
        self.text_fc2.forward(&activated)
    }

    pub fn embed_codec(&self, ids: &[u32], device: &Device) -> Result<Tensor> {
        if ids.is_empty() {
            candle_core::bail!("Qwen3-TTS cannot embed empty codec tokens")
        }
        self.codec_embedding.forward(&Tensor::new(ids, device)?)
    }

    pub fn forward_codec_embeddings(&mut self, inputs: &Tensor) -> Result<(Tensor, Tensor)> {
        let hidden = self.transformer.forward(inputs)?;
        let last = hidden.narrow(0, hidden.dim(0)? - 1, 1)?;
        let logits = self.codec_head.forward(&last)?;
        Ok((hidden, logits))
    }

    pub fn generate_custom_voice_codes(
        &mut self,
        predictor: &mut QwenTtsCodePredictor,
        assistant_text_ids: &[u32],
        device: &Device,
        max_frames: usize,
    ) -> Result<Vec<Vec<u32>>> {
        self.generate_custom_voice_codes_with_progress(
            predictor,
            assistant_text_ids,
            device,
            max_frames,
            &mut |_| {},
        )
    }

    pub fn generate_custom_voice_codes_with_progress(
        &mut self,
        predictor: &mut QwenTtsCodePredictor,
        assistant_text_ids: &[u32],
        device: &Device,
        max_frames: usize,
        on_frame: &mut impl FnMut(usize),
    ) -> Result<Vec<Vec<u32>>> {
        self.generate_custom_voice_codes_with_control(
            predictor,
            assistant_text_ids,
            device,
            max_frames,
            on_frame,
            &|| false,
        )
    }

    pub fn generate_custom_voice_codes_with_control(
        &mut self,
        predictor: &mut QwenTtsCodePredictor,
        assistant_text_ids: &[u32],
        device: &Device,
        max_frames: usize,
        on_frame: &mut impl FnMut(usize),
        should_cancel: &impl Fn() -> bool,
    ) -> Result<Vec<Vec<u32>>> {
        self.generate_custom_voice_codes_observed(
            predictor,
            assistant_text_ids,
            device,
            max_frames,
            on_frame,
            should_cancel,
            None,
            SamplingRng::from_entropy(),
        )
    }

    pub fn generate_custom_voice_codes_with_diagnostics(
        &mut self,
        predictor: &mut QwenTtsCodePredictor,
        assistant_text_ids: &[u32],
        device: &Device,
        max_frames: usize,
        on_frame: &mut impl FnMut(usize),
        on_diagnostic: &mut impl FnMut(AcousticFrameDiagnostic),
    ) -> Result<Vec<Vec<u32>>> {
        self.generate_custom_voice_codes_observed(
            predictor,
            assistant_text_ids,
            device,
            max_frames,
            on_frame,
            &|| false,
            Some(on_diagnostic),
            SamplingRng::seeded(659),
        )
    }

    fn generate_custom_voice_codes_observed(
        &mut self,
        predictor: &mut QwenTtsCodePredictor,
        assistant_text_ids: &[u32],
        device: &Device,
        max_frames: usize,
        on_frame: &mut impl FnMut(usize),
        should_cancel: &impl Fn() -> bool,
        mut on_diagnostic: Option<&mut dyn FnMut(AcousticFrameDiagnostic)>,
        mut rng: SamplingRng,
    ) -> Result<Vec<Vec<u32>>> {
        if assistant_text_ids.len() < 6 || max_frames == 0 || max_frames > 375 {
            candle_core::bail!("Qwen3-TTS requires bounded nonempty ChatML text tokens")
        }
        let text = &assistant_text_ids[3..assistant_text_ids.len() - 2];
        if text.is_empty() {
            candle_core::bail!("Qwen3-TTS requires a nonempty text body")
        }
        let config = &self.config;
        let mut pieces = Vec::new();
        pieces.push(self.project_text(&self.embed_text(&assistant_text_ids[..3], device)?)?);
        let codec_prefix = self.embed_codec(&config.mandarin_vivian_codec_prefix(), device)?;
        let tts_pad = self.project_text(&self.embed_text(&[config.pad_token], device)?)?;
        let tts_bos = self.project_text(&self.embed_text(&[config.bos_token], device)?)?;
        let tts_eos = self.project_text(&self.embed_text(&[config.eos_token], device)?)?;
        let prefill_count = codec_prefix.dim(0)? - 1;
        let mut prefix_text = Vec::new();
        for _ in 0..prefill_count - 1 {
            prefix_text.push(tts_pad.clone());
        }
        prefix_text.push(tts_bos.clone());
        let prefix_text = Tensor::cat(&prefix_text.iter().collect::<Vec<_>>(), 0)?;
        pieces.push(prefix_text.broadcast_add(&codec_prefix.narrow(0, 0, prefill_count)?)?);
        let mut body_text = self.project_text(&self.embed_text(text, device)?)?;
        body_text = Tensor::cat(&[&body_text, &tts_eos], 0)?;
        let codec_padding =
            self.embed_codec(&vec![config.codec_pad_token; text.len() + 1], device)?;
        pieces.push(body_text.broadcast_add(&codec_padding)?);
        pieces.push(tts_pad.broadcast_add(&self.embed_codec(&[config.codec_bos_token], device)?)?);
        let input = Tensor::cat(&pieces.iter().collect::<Vec<_>>(), 0)?;
        self.transformer.reset();
        let mut outputs = Vec::new();
        let mut code_history = Vec::new();
        let parameters = GenerationSampling::default();
        let mut repeated_code = None;
        let mut repeated_frames = 0_usize;
        let mut hidden = self.transformer.forward(&input)?;
        for _ in 0..max_frames {
            if should_cancel() {
                candle_core::bail!("Qwen3-TTS inference cancelled");
            }
            let final_hidden = hidden.narrow(0, hidden.dim(0)? - 1, 1)?;
            let logits = self.codec_head.forward(&final_hidden)?;
            let scores = logits.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
            let chosen_code = sample_acoustic_code(
                &scores,
                self.config.predictor.vocab_size,
                self.config.codec_eos_token as usize,
                &code_history,
                parameters,
                &mut rng,
            )
            .map_err(candle_core::Error::Msg)?;
            if let Some(observe) = on_diagnostic.as_mut() {
                let candidates = &scores[..self.config.predictor.vocab_size];
                let (winning_code, &winning_score) = candidates
                    .iter()
                    .enumerate()
                    .max_by(|left, right| {
                        left.1
                            .total_cmp(right.1)
                            .then_with(|| right.0.cmp(&left.0))
                    })
                    .ok_or_else(|| candle_core::Error::Msg("empty acoustic vocabulary".into()))?;
                observe(AcousticFrameDiagnostic {
                    frame: outputs.len() + 1,
                    winning_code: winning_code as u32,
                    winning_score,
                    eos_score: scores[self.config.codec_eos_token as usize],
                    selected_eos: chosen_code.is_none(),
                    selected_code: chosen_code,
                });
            }
            let Some(code) = chosen_code else {
                if outputs.is_empty() {
                    candle_core::bail!("Qwen3-TTS stopped before generating speech")
                }
                return Ok(outputs);
            };
            if repeated_code == Some(code) {
                repeated_frames += 1;
            } else {
                repeated_code = Some(code);
                repeated_frames = 1;
            }
            if repeated_frames >= 32 {
                candle_core::bail!("Qwen3-TTS acoustic generation entered a repeated-code loop");
            }
            code_history.push(code);
            let first_embedding = self.embed_codec(&[code], device)?;
            let frame_codes = predictor.generate_sampled(
                code,
                &final_hidden,
                &first_embedding,
                parameters,
                &mut rng,
            )?;
            let combined_embedding =
                predictor.combine_codebook_embeddings(&frame_codes, &first_embedding, device)?;
            outputs.push(frame_codes);
            on_frame(outputs.len());
            hidden = self
                .transformer
                .forward(&tts_pad.broadcast_add(&combined_embedding)?)?;
        }
        candle_core::bail!("Qwen3-TTS generation reached maximum acoustic frames without EOS")
    }

    pub fn reset(&mut self) {
        self.transformer.reset();
    }
}

impl QwenTtsCodePredictor {
    pub fn load(root: &Path, device: &Device) -> Result<Self> {
        let config = CustomVoiceConfig::from_directory(root).map_err(candle_core::Error::Msg)?;
        let checkpoint = root.join("model.safetensors");
        let vb = checkpoint_var_builder(&checkpoint, device)?;
        Self::load_weights(&config, vb)
    }

    pub fn load_weights(config: &CustomVoiceConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let predictor = vb.pp("talker.code_predictor");
        let model = predictor.pp("model");
        let embeddings = (0..config.codebooks - 1)
            .map(|index| {
                candle_nn::embedding(
                    config.predictor.vocab_size,
                    config.predictor.hidden_size,
                    model.pp(format!("codec_embedding.{index}")),
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let transformer = CausalTransformer::load(&config.predictor, model)?;
        let heads = (0..config.codebooks - 1)
            .map(|index| {
                candle_nn::linear_no_bias(
                    config.predictor.hidden_size,
                    config.predictor.vocab_size,
                    predictor.pp(format!("lm_head.{index}")),
                )
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            transformer,
            embeddings,
            heads,
            codebook_size: config.predictor.vocab_size,
        })
    }

    pub fn combine_codebook_embeddings(
        &self,
        codes: &[u32],
        first_embedding: &Tensor,
        device: &Device,
    ) -> Result<Tensor> {
        if codes.len() != self.embeddings.len() + 1 || first_embedding.dims2()?.0 != 1 {
            candle_core::bail!("Qwen3-TTS expects one embedding for each acoustic codebook")
        }
        let mut combined = first_embedding.clone();
        for (index, &code) in codes.iter().enumerate().skip(1) {
            if code as usize >= self.codebook_size {
                candle_core::bail!("Qwen3-TTS residual code exceeds codebook size")
            }
            let next = self.embeddings[index - 1].forward(&Tensor::new(&[code], device)?)?;
            combined = combined.broadcast_add(&next)?;
        }
        Ok(combined)
    }

    pub fn generate_greedy(
        &mut self,
        first_code: u32,
        talker_hidden: &Tensor,
        first_codec_embedding: &Tensor,
    ) -> Result<Vec<u32>> {
        if first_code as usize >= self.codebook_size {
            candle_core::bail!("Qwen3-TTS code predictor received invalid first code")
        }
        let (length, width) = talker_hidden.dims2()?;
        if length != 1
            || width != first_codec_embedding.dim(1)?
            || first_codec_embedding.dim(0)? != 1
        {
            candle_core::bail!("Qwen3-TTS code predictor requires one talker hidden state")
        }
        self.transformer.reset();
        let mut codes = vec![first_code];
        let prefill = Tensor::cat(&[talker_hidden, first_codec_embedding], 0)?;
        let mut hidden = self.transformer.forward(&prefill)?;
        for (index, head) in self.heads.iter().enumerate() {
            let last = hidden.narrow(0, hidden.dim(0)? - 1, 1)?;
            let logits = head.forward(&last)?.to_dtype(DType::F32)?;
            let code = logits.argmax(D::Minus1)?.to_vec1::<u32>()?[0];
            codes.push(code);
            if index + 1 < self.heads.len() {
                let next =
                    self.embeddings[index].forward(&Tensor::new(&[code], hidden.device())?)?;
                hidden = self.transformer.forward(&next)?;
            }
        }
        Ok(codes)
    }
    pub fn generate_sampled(
        &mut self,
        first_code: u32,
        talker_hidden: &Tensor,
        first_codec_embedding: &Tensor,
        parameters: GenerationSampling,
        rng: &mut SamplingRng,
    ) -> Result<Vec<u32>> {
        if first_code as usize >= self.codebook_size {
            candle_core::bail!("Qwen3-TTS code predictor received invalid first code")
        }
        let (length, width) = talker_hidden.dims2()?;
        if length != 1
            || width != first_codec_embedding.dim(1)?
            || first_codec_embedding.dim(0)? != 1
        {
            candle_core::bail!("Qwen3-TTS code predictor requires one talker hidden state")
        }
        self.transformer.reset();
        let mut codes = vec![first_code];
        let prefill = Tensor::cat(&[talker_hidden, first_codec_embedding], 0)?;
        let mut hidden = self.transformer.forward(&prefill)?;
        for (index, head) in self.heads.iter().enumerate() {
            let last = hidden.narrow(0, hidden.dim(0)? - 1, 1)?;
            let logits = head.forward(&last)?.to_dtype(DType::F32)?;
            let scores = logits.flatten_all()?.to_vec1::<f32>()?;
            let code = sample_residual_code(&scores, parameters, rng)
                .map_err(candle_core::Error::Msg)?;
            codes.push(code);
            if index + 1 < self.heads.len() {
                let next =
                    self.embeddings[index].forward(&Tensor::new(&[code], hidden.device())?)?;
                hidden = self.transformer.forward(&next)?;
            }
        }
        Ok(codes)
    }
}

#[cfg(test)]
mod tests {
    use super::{QwenTtsTalker, select_acoustic_token};
    use candle_core::{Device, Tensor};

    #[test]
    fn acoustic_token_selection_keeps_greedy_and_eos_semantics() {
        let device = Device::Cpu;
        let logits = Tensor::new(&[[0.1_f32, 0.9, 0.4, 0.3]], &device).unwrap();
        assert_eq!(select_acoustic_token(&logits, 3, 3).unwrap(), Some(1));
        let eos = Tensor::new(&[[0.1_f32, 0.9, 0.4, 1.2]], &device).unwrap();
        assert_eq!(select_acoustic_token(&eos, 3, 3).unwrap(), None);
        let tied = Tensor::new(&[[0.9_f32, 0.9, 0.2, 0.9]], &device).unwrap();
        assert_eq!(select_acoustic_token(&tied, 3, 3).unwrap(), Some(0));
        assert!(select_acoustic_token(&logits, 5, 3).is_err());
        assert!(select_acoustic_token(&logits, 3, 5).is_err());
        let nonfinite = Tensor::new(&[[0.1_f32, 0.2, 0.4, f32::NAN]], &device).unwrap();
        assert!(select_acoustic_token(&nonfinite, 3, 3).is_err());
    }

    #[test]
    fn checkpoint_loader_rejects_missing_config() {
        let root = std::env::temp_dir().join("zipeek-nonexistent-qwen3-tts-checkpoint");
        assert!(QwenTtsTalker::load(&root, &candle_core::Device::Cpu).is_err());
    }
}
