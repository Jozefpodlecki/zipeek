use std::path::Path;

use candle_core::{DType, Device};
use candle_nn::VarBuilder;

use super::{
    QwenAsrAssets,
    decoder::{
        TEXT_FFN_WIDTH, TEXT_HEAD_WIDTH, TEXT_HEADS, TEXT_KV_HEADS, TEXT_LAYERS, TEXT_WIDTH,
    },
    encoder::{AUDIO_ENCODER_LAYERS, AUDIO_FFN_WIDTH, AUDIO_OUTPUT_WIDTH},
    stem::{QWEN_ASR_06B_CONV_CHANNELS, QWEN_ASR_06B_ENCODER_WIDTH},
};

pub fn probe_qwen_asr_checkpoint(checkpoint: &Path, device: &Device) -> Result<(), String> {
    QwenAsrAssets::from_checkpoint(checkpoint)
        .validate()
        .map_err(str::to_owned)?;
    let filenames = [checkpoint];
    let weights = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, DType::F32, device) }
        .map_err(|error| format!("Candle could not map Qwen3-ASR safetensors: {error}"))?;
    let vb = weights.pp("thinker.audio_tower");
    let shapes = [
        ("conv2d1.weight", vec![QWEN_ASR_06B_CONV_CHANNELS, 1, 3, 3]),
        ("conv_out.weight", vec![QWEN_ASR_06B_ENCODER_WIDTH, 7680]),
        ("ln_post.weight", vec![QWEN_ASR_06B_ENCODER_WIDTH]),
        (
            "proj1.weight",
            vec![QWEN_ASR_06B_ENCODER_WIDTH, QWEN_ASR_06B_ENCODER_WIDTH],
        ),
        (
            "proj2.weight",
            vec![AUDIO_OUTPUT_WIDTH, QWEN_ASR_06B_ENCODER_WIDTH],
        ),
    ];
    for (name, shape) in shapes {
        vb.get(shape, name)
            .map_err(|error| format!("Qwen3-ASR acoustic weight {name}: {error}"))?;
    }
    for index in [0, AUDIO_ENCODER_LAYERS - 1] {
        let layer = vb.pp(format!("layers.{index}"));
        for (name, shape) in [
            ("self_attn.q_proj.weight", vec![896, 896]),
            ("self_attn.k_proj.bias", vec![896]),
            ("self_attn_layer_norm.weight", vec![896]),
            ("fc1.weight", vec![AUDIO_FFN_WIDTH, 896]),
            ("fc2.weight", vec![896, AUDIO_FFN_WIDTH]),
            ("final_layer_norm.weight", vec![896]),
        ] {
            layer.get(shape, name).map_err(|error| {
                format!("Qwen3-ASR acoustic layer {index}, weight {name}: {error}")
            })?;
        }
    }
    let decoder = weights.pp("thinker.model");
    decoder
        .get(vec![TEXT_WIDTH], "norm.weight")
        .map_err(|error| format!("Qwen3-ASR decoder final normalization: {error}"))?;
    for index in [0, TEXT_LAYERS - 1] {
        let layer = decoder.pp(format!("layers.{index}"));
        for (name, shape) in [
            ("input_layernorm.weight", vec![TEXT_WIDTH]),
            (
                "self_attn.q_proj.weight",
                vec![TEXT_HEADS * TEXT_HEAD_WIDTH, TEXT_WIDTH],
            ),
            (
                "self_attn.k_proj.weight",
                vec![TEXT_KV_HEADS * TEXT_HEAD_WIDTH, TEXT_WIDTH],
            ),
            ("self_attn.q_norm.weight", vec![TEXT_HEAD_WIDTH]),
            ("self_attn.k_norm.weight", vec![TEXT_HEAD_WIDTH]),
            ("mlp.gate_proj.weight", vec![TEXT_FFN_WIDTH, TEXT_WIDTH]),
            ("post_attention_layernorm.weight", vec![TEXT_WIDTH]),
        ] {
            layer.get(shape, name).map_err(|error| {
                format!("Qwen3-ASR decoder layer {index}, weight {name}: {error}")
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_qwen_checkpoint_is_reported_before_mapping() {
        let error = probe_qwen_asr_checkpoint(
            Path::new("missing-qwen3-asr-model.safetensors"),
            &Device::Cpu,
        )
        .unwrap_err();
        assert!(error.contains("checkpoint must be model.safetensors"));
    }
}
