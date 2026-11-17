use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};

use candle_core::{D, DType, Device, IndexOp, Tensor};
use candle_nn::Module;
use candle_transformers::models::stable_diffusion;
use tokenizers::Tokenizer;

use super::{GeneratedImage, ImageBackend};
use crate::error::SdxlSetupError;

#[derive(Clone)]
pub struct SdxlBackend {
    model_root: PathBuf,
    steps: usize,
    guidance: f64,
}

impl SdxlBackend {
    pub fn new(model_root: PathBuf, steps: usize) -> Result<Self, SdxlSetupError> {
        if !(1..=100).contains(&steps) {
            return Err(SdxlSetupError::InvalidSteps(steps));
        }
        for path in REQUIRED_FILES {
            let full = model_root.join(path);
            let metadata =
                std::fs::metadata(&full).map_err(|source| SdxlSetupError::MissingFile {
                    path: full.clone(),
                    source,
                })?;
            if !metadata.is_file() {
                return Err(SdxlSetupError::NotAFile(full));
            }
            if metadata.len() == 0 {
                return Err(SdxlSetupError::EmptyFile(full));
            }
        }
        Ok(Self {
            model_root,
            steps,
            guidance: 7.5,
        })
    }

    fn file(&self, relative: &str) -> PathBuf {
        self.model_root.join(relative)
    }

    fn generate_blocking(
        &self,
        prompt: &str,
        width: u32,
        height: u32,
    ) -> Result<GeneratedImage, String> {
        self.generate_candle(prompt, width as usize, height as usize)
            .map_err(|err| format!("SDXL inference: {err}"))
    }

    fn generate_candle(
        &self,
        prompt: &str,
        width: usize,
        height: usize,
    ) -> Result<GeneratedImage, Box<dyn std::error::Error + Send + Sync>> {
        let device = Device::new_cuda(0)?;
        let dtype = DType::F16;
        let config = stable_diffusion::StableDiffusionConfig::sdxl(None, Some(height), Some(width));
        let mut scheduler = config.build_scheduler(self.steps)?;

        let chunk_count = ["clip/tokenizer.json", "clip2/tokenizer.json"]
            .iter()
            .map(|path| -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
                let tokenizer = Tokenizer::from_file(self.file(path))?;
                let tokens = tokenizer.encode(prompt, true)?;
                let content = tokens.len().saturating_sub(2);
                if content > MAX_PROMPT_CONTENT_TOKENS {
                    return Err(format!("Prompt contains {content} content tokens for {path}; maximum is {MAX_PROMPT_CONTENT_TOKENS}").into());
                }
                Ok(content.div_ceil(75).max(1))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter().max().unwrap_or(1);
        let clip1 = encode_text(
            prompt,
            "",
            &self.file("clip/tokenizer.json"),
            &self.file("base/text_encoder/model.fp16.safetensors"),
            &config.clip,
            &device,
            dtype,
            self.guidance > 1.0,
            chunk_count,
        )?;
        let clip2 = encode_text(
            prompt,
            "",
            &self.file("clip2/tokenizer.json"),
            &self.file("base/text_encoder_2/model.fp16.safetensors"),
            config
                .clip2
                .as_ref()
                .ok_or("SDXL CLIP2 configuration missing")?,
            &device,
            dtype,
            self.guidance > 1.0,
            chunk_count,
        )?;
        let conditioning = Tensor::cat(&[clip1, clip2], D::Minus1)?;
        tracing::info!("loading SDXL U-Net");
        let unet = config.build_unet(
            self.file("base/unet/diffusion_pytorch_model.fp16.safetensors"),
            &device,
            4,
            false,
            dtype,
        )?;
        let latents = Tensor::randn(0f32, 1f32, (1, 4, height / 8, width / 8), &device)?;
        let mut latents = (latents * scheduler.init_noise_sigma())?.to_dtype(dtype)?;
        let timesteps = scheduler.timesteps().to_vec();
        for (index, &timestep) in timesteps.iter().enumerate() {
            let input = if self.guidance > 1.0 {
                Tensor::cat(&[&latents, &latents], 0)?
            } else {
                latents.clone()
            };
            let input = scheduler.scale_model_input(input, timestep)?;
            let prediction = unet.forward(&input, timestep as f64, &conditioning)?;
            let prediction = if self.guidance > 1.0 {
                let parts = prediction.chunk(2, 0)?;
                (&parts[0] + ((&parts[1] - &parts[0])? * self.guidance)?)?
            } else {
                prediction
            };
            latents = scheduler.step(&prediction, timestep, &latents)?;
            tracing::debug!(step = index + 1, total = self.steps, "SDXL denoising");
        }
        drop(unet);
        tracing::info!("decoding SDXL latents with VAE");
        let vae = config.build_vae(
            self.file("vae/diffusion_pytorch_model.safetensors"),
            &device,
            dtype,
        )?;
        let decoded = vae.decode(&(&latents / 0.18215)?)?;
        let decoded = ((decoded / 2.)? + 0.5)?.to_device(&Device::Cpu)?;
        let decoded = (decoded.clamp(0f32, 1.)? * 255.)?.to_dtype(DType::U8)?;
        let pixels = decoded
            .i(0)?
            .permute((1, 2, 0))?
            .contiguous()?
            .flatten_all()?
            .to_vec1::<u8>()?;
        let bytes = super::png::encode_rgb8(width as u32, height as u32, &pixels)?;
        Ok(GeneratedImage {
            bytes,
            media_type: "image/png",
        })
    }
}

const REQUIRED_FILES: [&str; 6] = [
    "base/unet/diffusion_pytorch_model.fp16.safetensors",
    "base/text_encoder/model.fp16.safetensors",
    "base/text_encoder_2/model.fp16.safetensors",
    "vae/diffusion_pytorch_model.safetensors",
    "clip/tokenizer.json",
    "clip2/tokenizer.json",
];

const MAX_PROMPT_CONTENT_TOKENS: usize = 300;

fn clip_chunks(
    tokens: &[u32],
    window: usize,
    pad: u32,
    count: usize,
) -> Result<Vec<Vec<u32>>, String> {
    if window < 3 || tokens.len() < 2 || count == 0 {
        return Err("CLIP context or tokenizer special tokens are invalid".to_owned());
    }
    let bos = tokens[0];
    let eos = *tokens.last().expect("at least two special tokens");
    let content = &tokens[1..tokens.len() - 1];
    let chunk_size = window - 2;
    if content.len() > MAX_PROMPT_CONTENT_TOKENS {
        return Err(format!(
            "Prompt contains {} content tokens; the maximum is {MAX_PROMPT_CONTENT_TOKENS}",
            content.len()
        ));
    }
    if content.len() > count * chunk_size {
        return Err("CLIP chunk count cannot contain the complete prompt".to_owned());
    }
    let mut chunks = Vec::with_capacity(count);
    for index in 0..count {
        let start = index * chunk_size;
        let end = content.len().min(start + chunk_size);
        let mut window_tokens = vec![bos];
        if start < content.len() {
            window_tokens.extend_from_slice(&content[start..end]);
        }
        window_tokens.push(eos);
        window_tokens.resize(window, pad);
        chunks.push(window_tokens);
    }
    Ok(chunks)
}

fn encode_text(
    prompt: &str,
    negative: &str,
    tokenizer_path: &Path,
    weights: &Path,
    clip_config: &stable_diffusion::clip::Config,
    device: &Device,
    dtype: DType,
    guided: bool,
    chunk_count: usize,
) -> Result<Tensor, Box<dyn std::error::Error + Send + Sync>> {
    let tokenizer = Tokenizer::from_file(tokenizer_path)?;
    let pad_token = clip_config.pad_with.as_deref().unwrap_or("<|endoftext|>");
    let pad = *tokenizer
        .get_vocab(true)
        .get(pad_token)
        .ok_or("CLIP tokenizer missing padding token")?;
    let positive = tokenizer.encode(prompt, true)?.get_ids().to_vec();
    let negative = tokenizer.encode(negative, true)?.get_ids().to_vec();
    let window = clip_config.max_position_embeddings;
    if window < 3 || positive.len() < 2 || negative.len() < 2 {
        return Err("CLIP tokenizer did not return a valid context".into());
    }
    let content_count = positive.len() - 2;
    let positive_chunks = clip_chunks(&positive, window, pad, chunk_count)?;
    let negative_chunks = if guided {
        Some(clip_chunks(&negative, window, pad, chunk_count)?)
    } else {
        None
    };
    if chunk_count > 1 {
        tracing::info!(
            content_tokens = content_count,
            chunks = chunk_count,
            clip_window = window,
            "encoding SDXL long prompt in CLIP windows"
        );
    }
    let model = stable_diffusion::build_clip_transformer(clip_config, weights, device, DType::F32)?;
    let run_chunks = |chunks: &[Vec<u32>]| -> candle_core::Result<Tensor> {
        let mut output = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let input = Tensor::new(chunk.as_slice(), device)?.unsqueeze(0)?;
            output.push(model.forward(&input)?);
        }
        let refs = output.iter().collect::<Vec<_>>();
        Tensor::cat(&refs, 1)
    };
    let positive_embeddings = run_chunks(&positive_chunks)?;
    let embeddings = if let Some(negative_chunks) = negative_chunks {
        let negative_embeddings = run_chunks(&negative_chunks)?;
        Tensor::cat(&[negative_embeddings, positive_embeddings], 0)?
    } else {
        positive_embeddings
    };
    Ok(embeddings.to_dtype(dtype)?)
}

impl ImageBackend for SdxlBackend {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
        width: u32,
        height: u32,
    ) -> Pin<Box<dyn Future<Output = Result<GeneratedImage, String>> + Send + 'a>> {
        let backend = self.clone();
        let prompt = prompt.to_owned();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || backend.generate_blocking(&prompt, width, height))
                .await
                .map_err(|err| format!("SDXL worker failed: {err}"))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_model_files_are_reported_without_cuda_initialization() {
        let missing = std::env::temp_dir().join("zipeek-sdxl-missing-model");
        assert!(SdxlBackend::new(missing, 20).is_err());
    }
    #[test]
    fn model_error_reports_missing_file_path() {
        let missing = std::env::temp_dir().join("zipeek-sdxl-missing-test");
        let error = SdxlBackend::new(missing, 20).err().expect("missing assets");
        assert!(matches!(error, SdxlSetupError::MissingFile { .. }));
        assert!(
            error
                .to_string()
                .contains("diffusion_pytorch_model.fp16.safetensors")
        );
    }
    #[test]
    fn clip_chunks_preserve_all_250_content_tokens() {
        let mut tokens = vec![49406];
        tokens.extend(1000..1250);
        tokens.push(49407);
        let chunks = clip_chunks(&tokens, 77, 49407, 4).unwrap();
        assert_eq!(chunks.len(), 4);
        assert!(chunks.iter().all(|chunk| chunk.len() == 77));
        let recovered = chunks
            .iter()
            .flat_map(|chunk| &chunk[1..76])
            .filter(|&&id| id != 49407)
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(recovered, (1000..1250).collect::<Vec<_>>());
        assert_eq!(chunks[3][25], 49407);
    }

    #[test]
    fn guided_negative_windows_match_positive_sequence_length() {
        let positive = (0..250).chain(std::iter::once(49407)).collect::<Vec<_>>();
        let positive = [vec![49406], positive].concat();
        let negative = vec![49406, 49407];
        let positive_chunks = clip_chunks(&positive, 77, 49407, 4).unwrap();
        let negative_chunks = clip_chunks(&negative, 77, 49407, 4).unwrap();
        assert_eq!(positive_chunks.len() * 77, negative_chunks.len() * 77);
        assert_eq!(negative_chunks[3][1], 49407);
    }

    #[test]
    fn long_prompt_limit_is_explicit_not_silent_truncation() {
        let mut tokens = vec![49406];
        tokens.extend(0..301);
        tokens.push(49407);
        assert!(
            clip_chunks(&tokens, 77, 49407, 5)
                .unwrap_err()
                .contains("maximum is 300")
        );
    }

    #[test]
    #[ignore = "requires local SDXL model weights and CUDA; run explicitly"]
    fn sdxl_generates_png_from_250_token_prompt() {
        let model_root = std::env::var_os("ZIPEEK_IMAGE_MODEL_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .join("models/sdxl")
            });
        let tokenizer_path = model_root.join("clip/tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .unwrap_or_else(|error| panic!("cannot load {}: {error}", tokenizer_path.display()));
        let prompt = "tree ".repeat(250);
        let encoded = tokenizer
            .encode(prompt.as_str(), true)
            .expect("tokenize long prompt");
        assert!(encoded.len() >= 250);
        let backend = SdxlBackend::new(model_root, 1).expect("required SDXL model files");
        let image = backend
            .generate_blocking(&prompt, 256, 256)
            .expect("250-token prompt should generate using extended conditioning");
        assert_eq!(image.media_type, "image/png");
        assert!(image.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(image.bytes.len() > 100);
    }

    #[test]
    fn step_count_is_bounded() {
        assert!(SdxlBackend::new(PathBuf::new(), 0).is_err());
        assert!(SdxlBackend::new(PathBuf::new(), 101).is_err());
    }
}
