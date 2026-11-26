use std::{fs, path::Path};

use candle_core::{DType, Device, Module, Result, Tensor};
use candle_nn::{Conv1d, Conv1dConfig, VarBuilder};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeechDecoderConfig {
    pub codebook_size: usize,
    pub semantic_codebook_size: usize,
    pub codebook_dim: usize,
    pub quantizers: usize,
    pub latent_dim: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub layers: usize,
    pub heads: usize,
    pub head_dim: usize,
    pub sliding_window: usize,
    pub decoder_dim: usize,
    pub upsample_rates: Vec<usize>,
    pub upsampling_ratios: Vec<usize>,
    pub decode_upsample_rate: usize,
}

impl SpeechDecoderConfig {
    pub fn from_directory(root: &Path) -> std::result::Result<Self, String> {
        let text = fs::read_to_string(root.join("speech_tokenizer/config.json"))
            .map_err(|error| format!("Qwen3-TTS speech tokenizer config: {error}"))?;
        Self::from_json(&text)
    }

    pub fn from_json(text: &str) -> std::result::Result<Self, String> {
        let root: Value = serde_json::from_str(text)
            .map_err(|error| format!("Qwen3-TTS speech tokenizer configuration: {error}"))?;
        let config = root
            .get("decoder_config")
            .ok_or("missing Qwen3-TTS speech tokenizer decoder_config")?;
        let number = |source: &Value, name: &str| {
            source
                .get(name)
                .and_then(Value::as_u64)
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| format!("invalid speech decoder parameter: {name}"))
        };
        let vector = |name: &str| -> std::result::Result<Vec<usize>, String> {
            config
                .get(name)
                .and_then(Value::as_array)
                .ok_or_else(|| format!("invalid speech decoder parameter: {name}"))?
                .iter()
                .map(|item| {
                    item.as_u64()
                        .and_then(|n| usize::try_from(n).ok())
                        .ok_or_else(|| format!("invalid speech decoder factor: {name}"))
                })
                .collect()
        };
        let result = Self {
            codebook_size: number(config, "codebook_size")?,
            semantic_codebook_size: number(config, "semantic_codebook_size")?,
            codebook_dim: number(config, "codebook_dim")?,
            quantizers: number(config, "num_quantizers")?,
            latent_dim: number(config, "latent_dim")?,
            hidden_size: number(config, "hidden_size")?,
            intermediate_size: number(config, "intermediate_size")?,
            layers: number(config, "num_hidden_layers")?,
            heads: number(config, "num_attention_heads")?,
            head_dim: number(config, "head_dim")?,
            sliding_window: number(config, "sliding_window")?,
            decoder_dim: number(config, "decoder_dim")?,
            upsample_rates: vector("upsample_rates")?,
            upsampling_ratios: vector("upsampling_ratios")?,
            decode_upsample_rate: number(&root, "decode_upsample_rate")?,
        };
        if result.quantizers != 16
            || result.codebook_size != 2048
            || result.semantic_codebook_size != 4096
            || result.codebook_dim == 0
            || result.codebook_dim % 2 != 0
            || result.codebook_dim > 4096
            || result.latent_dim == 0
            || result.decode_upsample_rate != 1920
            || result.hidden_size != 512
            || result.intermediate_size != 1024
            || result.layers != 8
            || result.heads != 16
            || result.head_dim != 64
            || result.sliding_window != 72
            || result.decoder_dim != 1536
            || result.upsample_rates != [8, 5, 4, 3]
            || result.upsampling_ratios != [2, 2]
        {
            return Err("unsupported Qwen3-TTS 12Hz speech tokenizer configuration".into());
        }
        Ok(result)
    }
}

struct Codebook {
    embedding_sum: Tensor,
    cluster_usage: Tensor,
}

impl Codebook {
    fn load(vb: VarBuilder<'_>, size: usize, dim: usize) -> Result<Self> {
        let weights = vb.pp("_codebook");
        Ok(Self {
            embedding_sum: weights.get((size, dim), "embedding_sum")?,
            cluster_usage: weights.get(size, "cluster_usage")?,
        })
    }

    fn decode(&self, ids: &[u32], device: &Device) -> Result<Tensor> {
        let size = self.cluster_usage.dim(0)?;
        if ids.iter().any(|&id| id as usize >= size) {
            candle_core::bail!("Qwen3-TTS acoustic code exceeds codebook vocabulary")
        }
        let indices = Tensor::new(ids, device)?;
        let weighted = self.embedding_sum.index_select(&indices, 0)?;
        let usage = self
            .cluster_usage
            .index_select(&indices, 0)?
            .clamp(1e-5f64, f64::MAX)?
            .unsqueeze(1)?;
        weighted.broadcast_div(&usage)
    }
}

struct CodebookBranch {
    books: Vec<Codebook>,
    output_projection: Tensor,
}

impl CodebookBranch {
    fn load(
        vb: VarBuilder<'_>,
        number: usize,
        size: usize,
        half: usize,
        full: usize,
    ) -> Result<Self> {
        let quantizers = vb.pp("vq");
        let books = (0..number)
            .map(|index| Codebook::load(quantizers.pp(format!("layers.{index}")), size, half))
            .collect::<Result<Vec<_>>>()?;
        let output_projection = vb
            .pp("output_proj")
            .get((full, half, 1), "weight")?
            .reshape((full, half))?;
        Ok(Self {
            books,
            output_projection,
        })
    }

    fn decode(&self, codes: &[Vec<u32>], offset: usize, device: &Device) -> Result<Tensor> {
        let mut accumulated = self.books[0].decode(
            &codes.iter().map(|frame| frame[offset]).collect::<Vec<_>>(),
            device,
        )?;
        for (index, book) in self.books.iter().enumerate().skip(1) {
            let codes_for_book = codes
                .iter()
                .map(|frame| frame[offset + index])
                .collect::<Vec<_>>();
            accumulated = accumulated.broadcast_add(&book.decode(&codes_for_book, device)?)?;
        }
        accumulated.matmul(&self.output_projection.t()?)
    }
}

struct CausalConv {
    convolution: Conv1d,
    left_padding: usize,
}

impl CausalConv {
    fn load(
        input_channels: usize,
        output_channels: usize,
        kernel: usize,
        vb: VarBuilder<'_>,
    ) -> Result<Self> {
        let cfg = Conv1dConfig::default();
        Ok(Self {
            convolution: candle_nn::conv1d(
                input_channels,
                output_channels,
                kernel,
                cfg,
                vb.pp("conv"),
            )?,
            left_padding: kernel - 1,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        self.convolution
            .forward(&input.pad_with_zeros(2, self.left_padding, 0)?)
    }
}

pub struct SpeechCodebookDecoder {
    config: SpeechDecoderConfig,
    semantic: CodebookBranch,
    residual: CodebookBranch,
    pre_conv: CausalConv,
    transformer: super::codec_transformer::AudioDecoderTransformer,
    vocoder: super::codec_vocoder::SpeechVocoder,
}

impl SpeechCodebookDecoder {
    pub fn load(root: &Path, device: &Device) -> Result<Self> {
        let config = SpeechDecoderConfig::from_directory(root).map_err(candle_core::Error::Msg)?;
        let checkpoint = root.join("speech_tokenizer/model.safetensors");
        let files = [checkpoint.as_path()];
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&files, DType::F32, device)? };
        Self::load_weights(config, vb.pp("decoder"))
    }

    pub fn load_weights(config: SpeechDecoderConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let half = config.codebook_dim / 2;
        let quantizer = vb.pp("quantizer");
        let pre_conv =
            CausalConv::load(config.codebook_dim, config.latent_dim, 3, vb.pp("pre_conv"))?;
        let transformer = super::codec_transformer::AudioDecoderTransformer::load(
            &config,
            vb.pp("pre_transformer"),
        )?;
        let vocoder = super::codec_vocoder::SpeechVocoder::load(&config, vb.clone())?;
        Ok(Self {
            semantic: CodebookBranch::load(
                quantizer.pp("rvq_first"),
                1,
                config.codebook_size,
                half,
                config.codebook_dim,
            )?,
            residual: CodebookBranch::load(
                quantizer.pp("rvq_rest"),
                config.quantizers - 1,
                config.codebook_size,
                half,
                config.codebook_dim,
            )?,
            pre_conv,
            transformer,
            vocoder,
            config,
        })
    }

    pub fn reconstruct(&self, codes: &[Vec<u32>], device: &Device) -> Result<Tensor> {
        if codes.is_empty()
            || codes.len() > 375
            || codes
                .iter()
                .any(|frame| frame.len() != self.config.quantizers)
        {
            candle_core::bail!("Qwen3-TTS speech decoder expects 1..=375 frames of 16 codebooks")
        }
        if codes
            .iter()
            .flatten()
            .any(|&code| code as usize >= self.config.codebook_size)
        {
            candle_core::bail!("Qwen3-TTS speech token out of codebook range")
        }
        self.semantic
            .decode(codes, 0, device)?
            .broadcast_add(&self.residual.decode(codes, 1, device)?)
    }

    pub fn reconstruct_latents(&self, codes: &[Vec<u32>], device: &Device) -> Result<Tensor> {
        let quantized = self.reconstruct(codes, device)?;
        let input = quantized.t()?.unsqueeze(0)?;
        let convolved = self.pre_conv.forward(&input)?;
        let frames = convolved.squeeze(0)?.t()?;
        let transformed = self.transformer.forward(&frames)?;
        transformed.t()?.unsqueeze(0)
    }

    pub fn decode_waveform(&self, codes: &[Vec<u32>], device: &Device) -> Result<Vec<f32>> {
        let latents = self.reconstruct_latents(codes, device)?;
        let samples = self.vocoder.forward(&latents)?.squeeze(0)?.squeeze(0)?;
        let expected = self
            .output_samples(codes.len())
            .ok_or_else(|| candle_core::Error::Msg("invalid acoustic duration".into()))?;
        let output = samples.to_dtype(DType::F32)?.to_vec1::<f32>()?;
        if output.len() < expected || output[..expected].iter().any(|sample| !sample.is_finite()) {
            candle_core::bail!("Qwen3-TTS waveform decoder produced invalid samples")
        }
        Ok(output[..expected].to_vec())
    }

    pub fn output_samples(&self, frames: usize) -> Option<usize> {
        if frames == 0 || frames > 375 {
            return None;
        }
        frames.checked_mul(self.config.decode_upsample_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_published_12hz_decoder_contract() {
        let config = SpeechDecoderConfig::from_json(r#"{
            "decoder_config":{"codebook_size":2048,"semantic_codebook_size":4096,"codebook_dim":512,"num_quantizers":16,"latent_dim":1024,"hidden_size":512,"intermediate_size":1024,"num_hidden_layers":8,"num_attention_heads":16,"head_dim":64,"sliding_window":72,"decoder_dim":1536,"upsample_rates":[8,5,4,3],"upsampling_ratios":[2,2]},
            "decode_upsample_rate":1920
        }"#).unwrap();
        assert_eq!(config.quantizers, 16);
        assert_eq!(config.decode_upsample_rate, 1920);
        assert_eq!(config.codebook_dim / 2, 256);
        assert!(SpeechDecoderConfig::from_json("{}").is_err());
        assert!(SpeechDecoderConfig::from_json(r#"{"decoder_config":{"codebook_size":1024,"codebook_dim":1024,"num_quantizers":16,"latent_dim":1024,"hidden_size":512,"intermediate_size":1024,"num_hidden_layers":8,"num_attention_heads":16,"head_dim":64,"sliding_window":72,"decoder_dim":1536,"upsample_rates":[8,5,4,3],"upsampling_ratios":[2,2]},"decode_upsample_rate":1920}"#).is_err());
    }
}
