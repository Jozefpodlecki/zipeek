use candle_core::{Device, Tensor};

use super::frontend::{self, FrontendError, MEL_BINS};

#[derive(Debug)]
pub enum FeatureTensorError {
    Audio(FrontendError),
    Tensor(candle_core::Error),
}

impl std::fmt::Display for FeatureTensorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Audio(error) => write!(formatter, "invalid ASR audio input: {error:?}"),
            Self::Tensor(error) => write!(formatter, "Candle ASR input tensor: {error}"),
        }
    }
}

impl std::error::Error for FeatureTensorError {}

pub struct QwenAudioInput {
    pub features: Tensor,
    pub attention_mask: Tensor,
    pub valid_frames: usize,
    pub padded_frames: usize,
}

pub fn pcm_to_qwen_audio_input(
    samples: &[f32],
    device: &Device,
) -> Result<QwenAudioInput, FeatureTensorError> {
    let mel = frontend::extract_qwen_features(samples).map_err(FeatureTensorError::Audio)?;
    let frames = mel.padded_frames;
    let valid_frames = mel.valid_frames;
    let features = Tensor::from_vec(mel.values, (1, MEL_BINS, frames), device)
        .map_err(FeatureTensorError::Tensor)?;
    let mask = (0..frames)
        .map(|frame| u32::from(frame < valid_frames))
        .collect::<Vec<_>>();
    let attention_mask =
        Tensor::from_vec(mask, (1, frames), device).map_err(FeatureTensorError::Tensor)?;
    Ok(QwenAudioInput {
        features,
        attention_mask,
        valid_frames,
        padded_frames: frames,
    })
}

pub fn pcm_to_qwen_features(
    samples: &[f32],
    device: &Device,
) -> Result<Tensor, FeatureTensorError> {
    pcm_to_qwen_audio_input(samples, device).map(|input| input.features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_qwen3_asr_candle_tensor_on_cpu() {
        let tensor = pcm_to_qwen_features(&vec![0.0; 320], &Device::Cpu).unwrap();
        assert_eq!(tensor.dims(), &[1, 128, 100]);
        let shape = tensor.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        assert!(shape.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn mask_distinguishes_observed_and_chunk_padding_frames() {
        let input = pcm_to_qwen_audio_input(&vec![0.5; 16_160], &Device::Cpu).unwrap();
        assert_eq!(input.features.dims(), &[1, 128, 200]);
        assert_eq!(input.attention_mask.dims(), &[1, 200]);
        assert_eq!(input.valid_frames, 101);
        let mask = input
            .attention_mask
            .flatten_all()
            .unwrap()
            .to_vec1::<u32>()
            .unwrap();
        assert!(mask[..101].iter().all(|value| *value == 1));
        assert!(mask[101..].iter().all(|value| *value == 0));
    }

    #[test]
    fn rejects_non_finite_input_before_tensor_construction() {
        assert!(matches!(
            pcm_to_qwen_features(&[f32::NAN], &Device::Cpu),
            Err(FeatureTensorError::Audio(FrontendError::NonFinite))
        ));
    }
}
