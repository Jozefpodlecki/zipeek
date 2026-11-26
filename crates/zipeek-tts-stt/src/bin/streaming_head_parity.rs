use std::{env, fs, path::Path, process};

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use serde_json::Value;
use zipeek_tts_stt::candle_streaming_stt::checkpoint_head::{Zipformer32Decoder, Zipformer32Joiner};

fn number(value: &Value) -> Result<f32, String> {
    let number = value.as_f64().ok_or("expected numeric tensor element")?;
    if !number.is_finite() || number.abs() > f32::MAX as f64 {
        return Err("non-finite or out-of-range reference tensor element".into());
    }
    Ok(number as f32)
}

fn vector(value: &Value, expected: usize) -> Result<Vec<f32>, String> {
    let values = value.as_array().ok_or("expected reference tensor array")?;
    if values.len() != expected {
        return Err(format!("reference tensor length {}, expected {expected}", values.len()));
    }
    values.iter().map(number).collect()
}

fn compare(label: &str, actual: &[f32], reference: &[f32], absolute: f32, relative: f32) -> Result<(), String> {
    if actual.len() != reference.len() {
        return Err(format!("{label} shape mismatch"));
    }
    let mut maximum_error = 0.0_f32;
    let mut failing = None;
    for (index, (&got, &want)) in actual.iter().zip(reference).enumerate() {
        if !got.is_finite() {
            return Err(format!("{label}[{index}] produced a non-finite number"));
        }
        let error = (got - want).abs();
        maximum_error = maximum_error.max(error);
        if error > absolute + relative * want.abs() && failing.is_none() {
            failing = Some((index, got, want, error));
        }
    }
    println!("{label}: {} elements, maximum absolute error {maximum_error:.7}", actual.len());
    if let Some((index, got, want, error)) = failing {
        return Err(format!("{label} failed at {index}: Candle {got}, ONNX {want}, absolute error {error}"));
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("usage: streaming_head_parity DECODER.safetensors JOINER.safetensors reference.json".into());
    }
    let file = fs::read(&args[2]).map_err(|error| format!("reference fixture: {error}"))?;
    if file.len() > 8 * 1024 * 1024 {
        return Err("reference fixture exceeds 8 MiB".into());
    }
    let cases: Value = serde_json::from_slice(&file).map_err(|error| error.to_string())?;
    let cases = cases.get("cases").and_then(Value::as_array).ok_or("fixture must contain cases array")?;
    if cases.is_empty() || cases.len() > 8 {
        return Err("fixture must contain 1..=8 reference cases".into());
    }
    let device = Device::Cpu;
    let decoder_files = [Path::new(&args[0])];
    let joiner_files = [Path::new(&args[1])];
    let decoder_vb = unsafe { VarBuilder::from_mmaped_safetensors(&decoder_files, DType::F32, &device) }
        .map_err(|error| format!("decoder weights: {error}"))?;
    let joiner_vb = unsafe { VarBuilder::from_mmaped_safetensors(&joiner_files, DType::F32, &device) }
        .map_err(|error| format!("joiner weights: {error}"))?;
    let decoder = Zipformer32Decoder::load(decoder_vb).map_err(|error| format!("decoder load: {error}"))?;
    let joiner = Zipformer32Joiner::load(joiner_vb).map_err(|error| format!("joiner load: {error}"))?;
    for (index, case) in cases.iter().enumerate() {
        let tokens = case.get("tokens").and_then(Value::as_array).ok_or("missing token context")?;
        if tokens.len() != 2 {
            return Err("token context must contain two IDs".into());
        }
        let ids = [
            tokens[0].as_u64().and_then(|value| u32::try_from(value).ok()).ok_or("invalid token ID")?,
            tokens[1].as_u64().and_then(|value| u32::try_from(value).ok()).ok_or("invalid token ID")?,
        ];
        let encoded = vector(case.get("encoder").ok_or("missing encoder input")?, 512)?;
        let expected_decoder = vector(case.get("decoder").ok_or("missing decoder reference")?, 512)?;
        let expected_logits = vector(case.get("logits").ok_or("missing logits reference")?, 11661)?;
        let encoded = Tensor::from_vec(encoded, (1, 512), &device).map_err(|error| error.to_string())?;
        let decoded = decoder.forward(&ids, &device).map_err(|error| format!("case {index} decoder inference: {error}"))?;
        compare(&format!("case {index} decoder"), &decoded.flatten_all().and_then(|v| v.to_vec1::<f32>()).map_err(|error| error.to_string())?, &expected_decoder, 0.0002, 0.0002)?;
        let logits = joiner.logits(&encoded, &decoded).map_err(|error| format!("case {index} joiner inference: {error}"))?;
        compare(&format!("case {index} joiner"), &logits.flatten_all().and_then(|v| v.to_vec1::<f32>()).map_err(|error| error.to_string())?, &expected_logits, 0.002, 0.0005)?;
    }
    println!("All {} reference cases passed", cases.len());
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
    fn comparison_rejects_wrong_logits() {
        assert!(compare("same", &[0.2, -0.4], &[0.2, -0.4], 0.0001, 0.0001).is_ok());
        assert!(compare("different", &[0.2, -0.3], &[0.2, -0.4], 0.0001, 0.0001).is_err());
        assert!(compare("shape", &[0.2], &[0.2, -0.4], 0.0001, 0.0001).is_err());
        assert!(compare("nan", &[f32::NAN], &[1.0], 0.0001, 0.0001).is_err());
    }

    #[test]
    fn fixture_vectors_require_finite_fixed_width_floats() {
        assert!(vector(&serde_json::json!([0.25, 1.0]), 2).is_ok());
        assert!(vector(&serde_json::json!([0.25]), 2).is_err());
        assert!(vector(&serde_json::json!([0.25, "unknown"]), 2).is_err());
    }
}
