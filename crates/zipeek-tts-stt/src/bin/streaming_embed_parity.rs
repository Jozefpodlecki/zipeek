use std::{env, fs, path::Path, process};

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use serde_json::Value;
use zipeek_tts_stt::candle_streaming_stt::encoder_embed::Zipformer32EncoderEmbedding;

fn input_vector(value: &Value, length: usize) -> Result<Vec<f32>, String> {
    let values = value.as_array().ok_or("reference requires a numeric tensor array")?;
    if values.len() != length {
        return Err(format!("unexpected tensor length {}, expected {length}", values.len()));
    }
    values.iter().map(|value| {
        let number = value.as_f64().ok_or("non-numeric reference value")?;
        if !number.is_finite() || number.abs() > f32::MAX as f64 {
            return Err("non-finite or out-of-range reference value".into());
        }
        Ok(number as f32)
    }).collect()
}

fn compare(actual: &[f32], reference: &[f32]) -> Result<(), String> {
    let mut worst = (0usize, 0.0f32);
    for (index, (&got, &expected)) in actual.iter().zip(reference).enumerate() {
        let error = (got - expected).abs();
        if !got.is_finite() {
            return Err(format!("non-finite embedding cache at {index}"));
        }
        if error > worst.1 { worst = (index, error); }
    }
    println!("Embedding ConvNeXt cache: {} elements, maximum absolute error {:.7}", actual.len(), worst.1);
    let index = worst.0;
    let reference_value = reference[index];
    if worst.1 > 0.001 + 0.0005 * reference_value.abs() {
        return Err(format!("embedding cache diverges at {index}: Candle {}, ONNX {reference_value}", actual[index]));
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: streaming_embed_parity ENCODER.safetensors embedding-reference.json".into());
    }
    let bytes = fs::read(&args[1]).map_err(|error| format!("reference fixture: {error}"))?;
    if bytes.len() > 2 * 1024 * 1024 { return Err("embedding fixture exceeds 2 MiB".into()); }
    let fixture: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let features = input_vector(fixture.get("features").ok_or("missing features")?, 77 * 80)?;
    let reference = input_vector(fixture.get("next_embed_states").ok_or("missing cache")?, 128 * 3 * 19)?;
    let device = Device::Cpu;
    let files = [Path::new(&args[0])];
    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&files, DType::F32, &device) }
        .map_err(|error| format!("encoder weights: {error}"))?;
    let embed = Zipformer32EncoderEmbedding::load(vb).map_err(|error| format!("embedding load: {error}"))?;
    let features = Tensor::from_vec(features, (1, 77, 80), &device).map_err(|error| error.to_string())?;
    let cache = Tensor::zeros((1, 128, 3, 19), DType::F32, &device).map_err(|error| error.to_string())?;
    let (output, next_cache) = embed.forward_chunk(&features, &cache).map_err(|error| format!("embedding inference: {error}"))?;
    if output.dims() != [1, 32, 192] {
        return Err("unexpected embedding output dimensions".into());
    }
    let actual = next_cache.flatten_all().and_then(|value| value.to_vec1::<f32>()).map_err(|error| error.to_string())?;
    compare(&actual, &reference)?;
    println!("Streaming embedding reference passed");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_rejects_nan_and_wrong_lengths() {
        assert!(input_vector(&serde_json::json!([1, 2]), 1).is_err());
        assert!(input_vector(&serde_json::json!(["invalid"]), 1).is_err());
        assert!(input_vector(&serde_json::json!([1.0]), 1).is_ok());
    }
}
