use candle_core::{Module, Result, Tensor};
use candle_nn::{Conv2d, Conv2dConfig, Linear, VarBuilder};

use super::{
    frontend::{CHUNK_FRAMES, MEL_BINS},
    tensor::QwenAudioInput,
};

pub const QWEN_ASR_06B_CONV_CHANNELS: usize = 480;
pub const QWEN_ASR_06B_ENCODER_WIDTH: usize = 896;
pub const CONV_FREQUENCY_BINS: usize = 16;
pub const TOKENS_PER_CONV_CHUNK: usize = 13;

pub fn sinusoidal_positions(steps: usize, channels: usize) -> Vec<f32> {
    let half = channels / 2;
    let scale = 10_000.0_f64.ln() / (half - 1) as f64;
    let mut values = Vec::with_capacity(steps * channels);
    for position in 0..steps {
        for channel in 0..half {
            let angle = position as f64 * (-(channel as f64) * scale).exp();
            values.push(angle.sin() as f32);
        }
        for channel in 0..half {
            let angle = position as f64 * (-(channel as f64) * scale).exp();
            values.push(angle.cos() as f32);
        }
    }
    values
}

pub struct QwenAsrConvStem {
    conv1: Conv2d,
    conv2: Conv2d,
    conv3: Conv2d,
    projection: Linear,
}

impl QwenAsrConvStem {
    pub fn load(vb: VarBuilder<'_>) -> Result<Self> {
        let vb = vb.pp("thinker.audio_tower");
        let config = Conv2dConfig {
            stride: 2,
            padding: 1,
            ..Default::default()
        };
        let channels = QWEN_ASR_06B_CONV_CHANNELS;
        let conv1 = candle_nn::conv2d(1, channels, 3, config, vb.pp("conv2d1"))?;
        let conv2 = candle_nn::conv2d(channels, channels, 3, config, vb.pp("conv2d2"))?;
        let conv3 = candle_nn::conv2d(channels, channels, 3, config, vb.pp("conv2d3"))?;
        let projection = candle_nn::linear_no_bias(
            channels * CONV_FREQUENCY_BINS,
            QWEN_ASR_06B_ENCODER_WIDTH,
            vb.pp("conv_out"),
        )?;
        Ok(Self {
            conv1,
            conv2,
            conv3,
            projection,
        })
    }

    pub fn forward(&self, input: &QwenAudioInput) -> Result<Tensor> {
        let (batch, bands, frames) = input.features.dims3()?;
        if batch != 1
            || bands != MEL_BINS
            || frames != input.padded_frames
            || input.valid_frames == 0
            || input.valid_frames > frames
            || !frames.is_multiple_of(CHUNK_FRAMES)
        {
            candle_core::bail!("invalid Qwen3-ASR acoustic feature dimensions")
        }
        let positions = sinusoidal_positions(TOKENS_PER_CONV_CHUNK, QWEN_ASR_06B_ENCODER_WIDTH);
        let positions = Tensor::from_vec(
            positions,
            (TOKENS_PER_CONV_CHUNK, QWEN_ASR_06B_ENCODER_WIDTH),
            input.features.device(),
        )?;
        let mut chunks = Vec::with_capacity(input.valid_frames.div_ceil(CHUNK_FRAMES));
        for start in (0..input.valid_frames).step_by(CHUNK_FRAMES) {
            let valid_length = (input.valid_frames - start).min(CHUNK_FRAMES);
            let chunk = input
                .features
                .narrow(2, start, CHUNK_FRAMES)?
                .unsqueeze(1)?;
            let hidden = candle_nn::Activation::Gelu.forward(&chunk.apply(&self.conv1)?)?;
            let hidden = candle_nn::Activation::Gelu.forward(&hidden.apply(&self.conv2)?)?;
            let hidden = candle_nn::Activation::Gelu.forward(&hidden.apply(&self.conv3)?)?;
            let (batch, channels, frequency, tokens) = hidden.dims4()?;
            if frequency != CONV_FREQUENCY_BINS
                || channels != QWEN_ASR_06B_CONV_CHANNELS
                || tokens != TOKENS_PER_CONV_CHUNK
            {
                candle_core::bail!("unexpected Qwen3-ASR convolutional output shape")
            }
            let hidden =
                hidden
                    .permute((0, 3, 1, 2))?
                    .reshape((batch, tokens, channels * frequency))?;
            let projected = hidden.apply(&self.projection)?.squeeze(0)?;
            let positioned = projected.broadcast_add(&positions)?;
            chunks.push(positioned.narrow(0, 0, valid_length.div_ceil(8))?);
        }
        let parts: Vec<&Tensor> = chunks.iter().collect();
        Tensor::cat(&parts, 0)
    }
}

pub fn projected_token_count(valid_frames: usize) -> usize {
    (0..valid_frames)
        .step_by(CHUNK_FRAMES)
        .map(|start| (valid_frames - start).min(CHUNK_FRAMES).div_ceil(8))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_chunk_has_fewer_than_thirteen_tokens() {
        assert_eq!(projected_token_count(50), 7);
        assert_eq!(projected_token_count(100), 13);
        assert_eq!(projected_token_count(101), 14);
        assert_eq!(projected_token_count(199), 26);
        assert_eq!(projected_token_count(800), 104);
        assert_eq!(projected_token_count(3000), 390);
    }

    #[test]
    fn stem_dimensions_match_published_zero_point_six_billion_model() {
        assert_eq!(MEL_BINS.div_ceil(8), CONV_FREQUENCY_BINS);
        assert_eq!(QWEN_ASR_06B_CONV_CHANNELS * CONV_FREQUENCY_BINS, 7680);
        assert_eq!(QWEN_ASR_06B_ENCODER_WIDTH, 896);
        assert_eq!(TOKENS_PER_CONV_CHUNK, CHUNK_FRAMES.div_ceil(8));
    }

    #[test]
    fn sinusoidal_positions_match_upstream_non_interleaved_layout() {
        let values = sinusoidal_positions(2, QWEN_ASR_06B_ENCODER_WIDTH);
        let width = QWEN_ASR_06B_ENCODER_WIDTH;
        let half = width / 2;
        assert_eq!(values.len(), width * 2);
        assert!(values[..half].iter().all(|value| *value == 0.0));
        assert!(values[half..width].iter().all(|value| *value == 1.0));
        assert!((values[width] - 1.0_f32.sin()).abs() < 1e-6);
        assert!((values[width + half] - 1.0_f32.cos()).abs() < 1e-6);
        let final_frequency = 1.0 / 10_000.0_f32;
        assert!((values[width + half - 1] - final_frequency.sin()).abs() < 1e-6);
    }

    #[test]
    fn padded_convolution_chunks_keep_only_valid_projected_tokens() {
        for (frames, expected) in [(50, 7), (100, 13), (101, 14), (150, 20), (3000, 390)] {
            let count = (0..frames)
                .step_by(CHUNK_FRAMES)
                .map(|start| (frames - start).min(CHUNK_FRAMES).div_ceil(8))
                .sum::<usize>();
            assert_eq!(count, expected);
            assert_eq!(projected_token_count(frames), expected);
        }
    }
}
