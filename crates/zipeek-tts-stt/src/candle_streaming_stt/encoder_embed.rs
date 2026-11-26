use candle_core::{DType, Module, Result, Tensor};
use candle_nn::{Conv2d, Conv2dConfig, VarBuilder};

use super::zipformer32::{FEATURES_PER_CHUNK, FEATURE_BINS};

const EMBED_CHANNELS: usize = 128;
const EMBED_FREQ: usize = 19;
const EMBED_LEFT_CONTEXT: usize = 3;
const EMBED_OUTPUT: usize = 192;
const STREAMING_OUTPUT_FRAMES: usize = 32;

fn swoosh(input: &Tensor, shift: f64, offset: f64) -> Result<Tensor> {
    let shifted = input.affine(1.0, -shift)?;
    let softplus = shifted.exp()?.affine(1.0, 1.0)?.log()?;
    (&softplus - &input.affine(0.08, offset)?)
}

fn swoosh_right(input: &Tensor) -> Result<Tensor> {
    swoosh(input, 1.0, 0.313261687)
}

fn swoosh_left(input: &Tensor) -> Result<Tensor> {
    swoosh(input, 4.0, 0.035)
}

pub struct Zipformer32EncoderEmbedding {
    conv_first: Conv2d,
    conv_downsample: Conv2d,
    conv_frequency: Conv2d,
    depthwise: Conv2d,
    pointwise_in: Conv2d,
    pointwise_out: Conv2d,
    projection_weight: Tensor,
    projection_bias: Tensor,
    normalization_bias: Tensor,
    normalization_log_scale: Tensor,
}

impl Zipformer32EncoderEmbedding {
    pub fn load(vb: VarBuilder<'_>) -> Result<Self> {
        let projection_weight = vb.get((EMBED_CHANNELS * EMBED_FREQ, EMBED_OUTPUT), "onnx::MatMul_9326")?;
        let vb = vb.pp("encoder_embed");
        let standard = Conv2dConfig::default();
        let downsample = Conv2dConfig { stride: 2, ..Default::default() };
        let depthwise = Conv2dConfig { groups: EMBED_CHANNELS, ..Default::default() };
        let norm = vb.pp("out_norm");
        Ok(Self {
            conv_first: candle_nn::conv2d(1, 8, 3, standard, vb.pp("conv.0"))?,
            conv_downsample: candle_nn::conv2d(8, 32, 3, downsample, vb.pp("conv.4"))?,
            conv_frequency: candle_nn::conv2d(32, EMBED_CHANNELS, 3, standard, vb.pp("conv.7"))?,
            depthwise: candle_nn::conv2d(EMBED_CHANNELS, EMBED_CHANNELS, 7, depthwise, vb.pp("convnext.depthwise_conv"))?,
            pointwise_in: candle_nn::conv2d(EMBED_CHANNELS, 384, 1, standard, vb.pp("convnext.pointwise_conv1"))?,
            pointwise_out: candle_nn::conv2d(384, EMBED_CHANNELS, 1, standard, vb.pp("convnext.pointwise_conv2"))?,
            projection_weight,
            projection_bias: vb.pp("out").get((EMBED_OUTPUT,), "bias")?,
            normalization_bias: norm.get((EMBED_OUTPUT,), "bias")?,
            normalization_log_scale: norm.get((), "log_scale")?,
        })
    }

    pub fn forward_chunk(&self, features: &Tensor, cached_left_pad: &Tensor) -> Result<(Tensor, Tensor)> {
        let (batch, frames, bands) = features.dims3()?;
        if batch == 0 || batch > 16 || frames != FEATURES_PER_CHUNK || bands != FEATURE_BINS || features.dtype() != DType::F32 {
            candle_core::bail!("Zipformer embedding expects [batch, 77, 80] FP32 input")
        }
        if cached_left_pad.dims() != [batch, EMBED_CHANNELS, EMBED_LEFT_CONTEXT, EMBED_FREQ]
            || cached_left_pad.dtype() != DType::F32
        {
            candle_core::bail!("Zipformer embedding cache shape, dtype or device mismatch")
        }
        let x = features.unsqueeze(1)?.pad_with_zeros(3, 1, 1)?;
        let x = swoosh_right(&self.conv_first.forward(&x)?)?;
        let x = swoosh_right(&self.conv_downsample.forward(&x)?)?;
        let x = self.conv_frequency.forward(&x)?;
        let even_frequency = Tensor::from_vec((0..EMBED_FREQ).map(|index| (index * 2) as u32).collect(), (EMBED_FREQ,), features.device())?;
        let x = swoosh_right(&x.index_select(&even_frequency, 3)?)?;
        let (_, _, subsampled_frames, subsampled_bands) = x.dims4()?;
        if subsampled_frames != STREAMING_OUTPUT_FRAMES + EMBED_LEFT_CONTEXT || subsampled_bands != EMBED_FREQ {
            candle_core::bail!("Zipformer convolutional subsampling dimensions mismatch")
        }
        let combined = Tensor::cat(&[cached_left_pad, &x], 2)?;
        let next_left_pad = combined.narrow(2, STREAMING_OUTPUT_FRAMES, EMBED_LEFT_CONTEXT)?.contiguous()?;
        let bypass = x.narrow(2, 0, STREAMING_OUTPUT_FRAMES)?;
        let padded = combined.pad_with_zeros(3, 3, 3)?;
        let conv = self.depthwise.forward(&padded)?;
        let conv = self.pointwise_in.forward(&conv)?;
        let conv = swoosh_left(&conv)?;
        let conv = self.pointwise_out.forward(&conv)?;
        let mixed = (&bypass + &conv)?;
        let flattened = mixed.transpose(1, 2)?.contiguous()?.reshape((batch, STREAMING_OUTPUT_FRAMES, EMBED_CHANNELS * EMBED_FREQ))?;
        let projected = flattened
            .reshape((batch * STREAMING_OUTPUT_FRAMES, EMBED_CHANNELS * EMBED_FREQ))?
            .matmul(&self.projection_weight)?
            .broadcast_add(&self.projection_bias)?
            .reshape((batch, STREAMING_OUTPUT_FRAMES, EMBED_OUTPUT))?;
        let centered = projected.broadcast_sub(&self.normalization_bias)?;
        let rms = centered.sqr()?.mean_keepdim(2)?.affine(1.0, 1e-12)?.sqrt()?;
        let scale = self.normalization_log_scale.exp()?;
        let normalized = projected.broadcast_div(&rms)?.broadcast_mul(&scale)?;
        Ok((normalized, next_left_pad))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;

    #[test]
    fn streaming_embedding_has_exact_chunk_and_cache_shapes() {
        let device = Device::Cpu;
        let vb = VarBuilder::zeros(DType::F32, &device);
        let embedding = Zipformer32EncoderEmbedding::load(vb).unwrap();
        let frames = Tensor::zeros((1, FEATURES_PER_CHUNK, FEATURE_BINS), DType::F32, &device).unwrap();
        let cache = Tensor::zeros((1, 128, 3, 19), DType::F32, &device).unwrap();
        let (output, next_cache) = embedding.forward_chunk(&frames, &cache).unwrap();
        assert_eq!(output.dims(), &[1, 32, 192]);
        assert_eq!(next_cache.dims(), &[1, 128, 3, 19]);
        assert_eq!(embedding.forward_chunk(&frames, &next_cache).unwrap().0.dims(), &[1, 32, 192]);
    }

    #[test]
    fn embedding_rejects_unexpected_chunk_or_cache() {
        let device = Device::Cpu;
        let embedding = Zipformer32EncoderEmbedding::load(VarBuilder::zeros(DType::F32, &device)).unwrap();
        let cache = Tensor::zeros((1, 128, 3, 19), DType::F32, &device).unwrap();
        let invalid_features = Tensor::zeros((1, 76, 80), DType::F32, &device).unwrap();
        assert!(embedding.forward_chunk(&invalid_features, &cache).is_err());
        let features = Tensor::zeros((1, 77, 80), DType::F32, &device).unwrap();
        let invalid_cache = Tensor::zeros((1, 128, 4, 19), DType::F32, &device).unwrap();
        assert!(embedding.forward_chunk(&features, &invalid_cache).is_err());
    }
}
