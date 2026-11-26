use candle_core::{D, Module, Result, Tensor};
use candle_nn::{LayerNorm, Linear, VarBuilder};

use super::{
    layout::encoder_attention_windows,
    stem::{QWEN_ASR_06B_ENCODER_WIDTH, QwenAsrConvStem},
    tensor::QwenAudioInput,
};

pub const AUDIO_ENCODER_LAYERS: usize = 18;
pub const AUDIO_ATTENTION_HEADS: usize = 14;
pub const AUDIO_HEAD_DIM: usize = 64;
pub const AUDIO_FFN_WIDTH: usize = 3584;
pub const AUDIO_OUTPUT_WIDTH: usize = 1024;
const LAYER_NORM_EPS: f64 = 1e-5;

struct AcousticAttention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    out_proj: Linear,
}

impl AcousticAttention {
    fn load(vb: VarBuilder<'_>) -> Result<Self> {
        let width = QWEN_ASR_06B_ENCODER_WIDTH;
        Ok(Self {
            q_proj: candle_nn::linear(width, width, vb.pp("q_proj"))?,
            k_proj: candle_nn::linear(width, width, vb.pp("k_proj"))?,
            v_proj: candle_nn::linear(width, width, vb.pp("v_proj"))?,
            out_proj: candle_nn::linear(width, width, vb.pp("out_proj"))?,
        })
    }

    fn forward_window(&self, hidden: &Tensor) -> Result<Tensor> {
        let (tokens, width) = hidden.dims2()?;
        if tokens == 0 || width != QWEN_ASR_06B_ENCODER_WIDTH {
            candle_core::bail!("invalid Qwen3-ASR attention window dimensions")
        }
        let shape = (tokens, AUDIO_ATTENTION_HEADS, AUDIO_HEAD_DIM);
        let q = self
            .q_proj
            .forward(hidden)?
            .reshape(shape)?
            .transpose(0, 1)?;
        let k = self
            .k_proj
            .forward(hidden)?
            .reshape(shape)?
            .transpose(0, 1)?;
        let v = self
            .v_proj
            .forward(hidden)?
            .reshape(shape)?
            .transpose(0, 1)?;
        let logits = q
            .contiguous()?
            .matmul(&k.transpose(1, 2)?.contiguous()?)?
            .affine(1.0 / (AUDIO_HEAD_DIM as f64).sqrt(), 0.0)?;
        let attention = candle_nn::ops::softmax(&logits, D::Minus1)?;
        let attended = attention.contiguous()?.matmul(&v.contiguous()?)?;
        let attended = attended
            .transpose(0, 1)?
            .contiguous()?
            .reshape((tokens, width))?;
        self.out_proj.forward(&attended)
    }
}

struct AcousticEncoderLayer {
    self_attn: AcousticAttention,
    self_attn_layer_norm: LayerNorm,
    fc1: Linear,
    fc2: Linear,
    final_layer_norm: LayerNorm,
}

impl AcousticEncoderLayer {
    fn load(vb: VarBuilder<'_>) -> Result<Self> {
        let width = QWEN_ASR_06B_ENCODER_WIDTH;
        Ok(Self {
            self_attn: AcousticAttention::load(vb.pp("self_attn"))?,
            self_attn_layer_norm: candle_nn::layer_norm(
                width,
                LAYER_NORM_EPS,
                vb.pp("self_attn_layer_norm"),
            )?,
            fc1: candle_nn::linear(width, AUDIO_FFN_WIDTH, vb.pp("fc1"))?,
            fc2: candle_nn::linear(AUDIO_FFN_WIDTH, width, vb.pp("fc2"))?,
            final_layer_norm: candle_nn::layer_norm(
                width,
                LAYER_NORM_EPS,
                vb.pp("final_layer_norm"),
            )?,
        })
    }

    fn forward_window(&self, hidden: &Tensor) -> Result<Tensor> {
        let normalized = self.self_attn_layer_norm.forward(hidden)?;
        let attended = self.self_attn.forward_window(&normalized)?;
        let hidden = hidden.broadcast_add(&attended)?;
        let normalized = self.final_layer_norm.forward(&hidden)?;
        let feed_forward = candle_nn::Activation::Gelu.forward(&self.fc1.forward(&normalized)?)?;
        hidden.broadcast_add(&self.fc2.forward(&feed_forward)?)
    }
}

pub struct QwenAsrAudioEncoder {
    stem: QwenAsrConvStem,
    layers: Vec<AcousticEncoderLayer>,
    ln_post: LayerNorm,
    proj1: Linear,
    proj2: Linear,
}

impl QwenAsrAudioEncoder {
    pub fn load(vb: VarBuilder<'_>) -> Result<Self> {
        let stem = QwenAsrConvStem::load(vb.clone())?;
        let vb = vb.pp("thinker.audio_tower");
        let layers = (0..AUDIO_ENCODER_LAYERS)
            .map(|index| AcousticEncoderLayer::load(vb.pp(format!("layers.{index}"))))
            .collect::<Result<Vec<_>>>()?;
        let width = QWEN_ASR_06B_ENCODER_WIDTH;
        Ok(Self {
            stem,
            layers,
            ln_post: candle_nn::layer_norm(width, LAYER_NORM_EPS, vb.pp("ln_post"))?,
            proj1: candle_nn::linear(width, width, vb.pp("proj1"))?,
            proj2: candle_nn::linear(width, AUDIO_OUTPUT_WIDTH, vb.pp("proj2"))?,
        })
    }

    pub fn forward(&self, input: &QwenAudioInput) -> Result<Tensor> {
        let hidden = self.stem.forward(input)?;
        let windows = encoder_attention_windows(input.valid_frames)
            .map_err(|_| candle_core::Error::Msg("invalid Qwen3-ASR encoder layout".into()))?;
        let (token_count, width) = hidden.dims2()?;
        if width != QWEN_ASR_06B_ENCODER_WIDTH
            || windows.last().map(|window| window.token_end) != Some(token_count)
        {
            candle_core::bail!("Qwen3-ASR stem tokens disagree with attention window plan")
        }
        let mut encoded = Vec::with_capacity(windows.len());
        for window in windows {
            let mut state =
                hidden.narrow(0, window.token_start, window.token_end - window.token_start)?;
            for layer in &self.layers {
                state = layer.forward_window(&state)?;
            }
            state = self.ln_post.forward(&state)?;
            state = self.proj1.forward(&state)?;
            state = candle_nn::Activation::Gelu.forward(&state)?;
            encoded.push(self.proj2.forward(&state)?);
        }
        let parts = encoded.iter().collect::<Vec<_>>();
        Tensor::cat(&parts, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transposed_acoustic_attention_operands_support_batched_matmul() {
        let device = candle_core::Device::Cpu;
        let tokens = 20;
        let hidden = Tensor::ones(
            (tokens, AUDIO_ATTENTION_HEADS * AUDIO_HEAD_DIM),
            candle_core::DType::F32,
            &device,
        )
        .unwrap();
        let q = hidden
            .reshape((tokens, AUDIO_ATTENTION_HEADS, AUDIO_HEAD_DIM))
            .unwrap()
            .transpose(0, 1)
            .unwrap();
        let k = q.clone();
        let v = q.clone();
        let scores = q
            .contiguous()
            .unwrap()
            .matmul(&k.transpose(1, 2).unwrap().contiguous().unwrap())
            .unwrap();
        assert_eq!(scores.dims(), &[AUDIO_ATTENTION_HEADS, tokens, tokens]);
        let attended = scores
            .contiguous()
            .unwrap()
            .matmul(&v.contiguous().unwrap())
            .unwrap();
        assert_eq!(attended.dims(), &[AUDIO_ATTENTION_HEADS, tokens, AUDIO_HEAD_DIM]);
    }

    #[test]
    fn published_audio_encoder_dimensions() {
        assert_eq!(
            AUDIO_ATTENTION_HEADS * AUDIO_HEAD_DIM,
            QWEN_ASR_06B_ENCODER_WIDTH
        );
        assert_eq!(AUDIO_ENCODER_LAYERS, 18);
        assert_eq!(AUDIO_FFN_WIDTH, 3584);
        assert_eq!(AUDIO_OUTPUT_WIDTH, 1024);
    }

    #[test]
    fn attention_windows_cover_all_projected_tokens_without_overlap() {
        for frames in [1, 50, 100, 101, 799, 800, 801, 1600, 3000] {
            let windows = encoder_attention_windows(frames).unwrap();
            let mut next = 0;
            for window in &windows {
                assert_eq!(window.token_start, next);
                assert!(window.token_end > window.token_start);
                assert!(window.token_end - window.token_start <= 104);
                next = window.token_end;
            }
            assert_eq!(next, super::super::stem::projected_token_count(frames));
        }
    }

    #[test]
    #[ignore = "requires the user's downloaded 1.9 GB Qwen3-ASR checkpoint"]
    fn real_checkpoint_runs_full_acoustic_encoder() {
        let checkpoint = std::env::var_os("ZIPEEK_CANDLE_STT_MODEL").expect(
            "set ZIPEEK_CANDLE_STT_MODEL to models/speech/Qwen3-ASR-0.6B/model.safetensors",
        );
        let checkpoint = std::path::PathBuf::from(checkpoint);
        let device = candle_core::Device::Cpu;
        super::super::weights::probe_qwen_asr_checkpoint(&checkpoint, &device).unwrap();
        let files = [checkpoint.as_path()];
        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(&files, candle_core::DType::F32, &device)
        }
        .unwrap();
        let model = QwenAsrAudioEncoder::load(vb).unwrap();
        let input =
            super::super::tensor::pcm_to_qwen_audio_input(&vec![0.0; 16_000 / 2], &device).unwrap();
        let result = model.forward(&input).unwrap();
        assert_eq!(
            result.dims(),
            &[
                super::super::stem::projected_token_count(input.valid_frames),
                AUDIO_OUTPUT_WIDTH
            ]
        );
        assert!(
            result
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap()
                .iter()
                .all(|value| value.is_finite())
        );
    }
}
