use candle_core::{DType, Device, Result as CandleResult, Tensor};

use super::onnx_inspect::{Dimension, OnnxModelReport, TensorPort, TensorWeight};

pub const FEATURES_PER_CHUNK: usize = 77;
pub const FEATURE_BINS: usize = 80;
pub const ENCODER_DIM: usize = 512;
pub const VOCABULARY_SIZE: usize = 11661;
pub const DECODER_CONTEXT: usize = 2;
pub const EMBED_STATE_SHAPE: [usize; 4] = [1, 128, 3, 19];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayerCacheShape {
    pub history: usize,
    pub key_channels: usize,
    pub attention_channels: usize,
    pub value_channels: usize,
    pub convolution_channels: usize,
    pub convolution_history: usize,
}

pub const LAYER_CACHE_SHAPES: [LayerCacheShape; 12] = [
    LayerCacheShape { history: 128, key_channels: 128, attention_channels: 144, value_channels: 48, convolution_channels: 192, convolution_history: 15 },
    LayerCacheShape { history: 128, key_channels: 128, attention_channels: 144, value_channels: 48, convolution_channels: 192, convolution_history: 15 },
    LayerCacheShape { history: 64, key_channels: 128, attention_channels: 192, value_channels: 48, convolution_channels: 256, convolution_history: 15 },
    LayerCacheShape { history: 64, key_channels: 128, attention_channels: 192, value_channels: 48, convolution_channels: 256, convolution_history: 15 },
    LayerCacheShape { history: 32, key_channels: 128, attention_channels: 192, value_channels: 48, convolution_channels: 256, convolution_history: 7 },
    LayerCacheShape { history: 32, key_channels: 128, attention_channels: 192, value_channels: 48, convolution_channels: 256, convolution_history: 7 },
    LayerCacheShape { history: 16, key_channels: 256, attention_channels: 192, value_channels: 96, convolution_channels: 256, convolution_history: 7 },
    LayerCacheShape { history: 16, key_channels: 256, attention_channels: 192, value_channels: 96, convolution_channels: 256, convolution_history: 7 },
    LayerCacheShape { history: 32, key_channels: 128, attention_channels: 192, value_channels: 48, convolution_channels: 256, convolution_history: 7 },
    LayerCacheShape { history: 32, key_channels: 128, attention_channels: 192, value_channels: 48, convolution_channels: 256, convolution_history: 7 },
    LayerCacheShape { history: 64, key_channels: 128, attention_channels: 192, value_channels: 48, convolution_channels: 256, convolution_history: 15 },
    LayerCacheShape { history: 64, key_channels: 128, attention_channels: 192, value_channels: 48, convolution_channels: 256, convolution_history: 15 },
];

pub struct LayerCache {
    pub key: Tensor,
    pub nonlinear_attention: Tensor,
    pub value1: Tensor,
    pub value2: Tensor,
    pub convolution1: Tensor,
    pub convolution2: Tensor,
}

pub struct Zipformer32Cache {
    pub layers: Vec<LayerCache>,
    pub embedding: Tensor,
    pub processed_lengths: Tensor,
}

impl Zipformer32Cache {
    pub fn zeros(batch: usize, device: &Device) -> CandleResult<Self> {
        if batch == 0 || batch > 16 {
            candle_core::bail!("invalid Zipformer batch size {batch}")
        }
        let mut layers = Vec::with_capacity(LAYER_CACHE_SHAPES.len());
        for shape in LAYER_CACHE_SHAPES {
            layers.push(LayerCache {
                key: Tensor::zeros((shape.history, batch, shape.key_channels), DType::F32, device)?,
                nonlinear_attention: Tensor::zeros((1, batch, shape.history, shape.attention_channels), DType::F32, device)?,
                value1: Tensor::zeros((shape.history, batch, shape.value_channels), DType::F32, device)?,
                value2: Tensor::zeros((shape.history, batch, shape.value_channels), DType::F32, device)?,
                convolution1: Tensor::zeros((batch, shape.convolution_channels, shape.convolution_history), DType::F32, device)?,
                convolution2: Tensor::zeros((batch, shape.convolution_channels, shape.convolution_history), DType::F32, device)?,
            });
        }
        Ok(Self {
            layers,
            embedding: Tensor::zeros((batch, 128, 3, 19), DType::F32, device)?,
            processed_lengths: Tensor::zeros((batch,), DType::I64, device)?,
        })
    }
}


impl Zipformer32Cache {
    pub fn replace_from_outputs(&mut self, outputs: &std::collections::BTreeMap<String, Tensor>) -> CandleResult<()> {
        let batch = self.processed_lengths.dims()[0];
        let expected = LAYER_CACHE_SHAPES.len() * 6 + 2;
        if outputs.len() != expected {
            candle_core::bail!("Zipformer cache update requires {expected} outputs, got {}", outputs.len())
        }
        let require = |name: &str, dims: &[usize], dtype: DType| -> CandleResult<Tensor> {
            let value = match outputs.get(name) {
                Some(value) => value,
                None => candle_core::bail!("missing Zipformer cache output {name}"),
            };
            if value.dims() != dims || value.dtype() != dtype {
                candle_core::bail!("Zipformer cache output {name} has incorrect shape or dtype")
            }
            Ok(value.clone())
        };
        let mut next = Vec::with_capacity(LAYER_CACHE_SHAPES.len());
        for (index, shape) in LAYER_CACHE_SHAPES.iter().enumerate() {
            let key = require(&format!("new_cached_key_{index}"), &[shape.history, batch, shape.key_channels], DType::F32)?;
            let nonlinear_attention = require(&format!("new_cached_nonlin_attn_{index}"), &[1, batch, shape.history, shape.attention_channels], DType::F32)?;
            let value1 = require(&format!("new_cached_val1_{index}"), &[shape.history, batch, shape.value_channels], DType::F32)?;
            let value2 = require(&format!("new_cached_val2_{index}"), &[shape.history, batch, shape.value_channels], DType::F32)?;
            let convolution1 = require(&format!("new_cached_conv1_{index}"), &[batch, shape.convolution_channels, shape.convolution_history], DType::F32)?;
            let convolution2 = require(&format!("new_cached_conv2_{index}"), &[batch, shape.convolution_channels, shape.convolution_history], DType::F32)?;
            next.push(LayerCache { key, nonlinear_attention, value1, value2, convolution1, convolution2 });
        }
        let embedding = require("new_embed_states", &[batch, 128, 3, 19], DType::F32)?;
        let processed_lengths = require("new_processed_lens", &[batch], DType::I64)?;
        self.layers = next;
        self.embedding = embedding;
        self.processed_lengths = processed_lengths;
        Ok(())
    }
}

fn expect_port(ports: &[TensorPort], name: &str, dims: &[Option<usize>], dtype: u64) -> Result<(), String> {
    let port = ports.iter().find(|port| port.name == name)
        .ok_or_else(|| format!("missing Zipformer port {name}"))?;
    if port.element_type != Some(dtype) || port.dimensions.len() != dims.len() {
        return Err(format!("Zipformer port {name} type/rank mismatch"));
    }
    for (actual, required) in port.dimensions.iter().zip(dims) {
        if let Some(required) = required {
            if actual != &Dimension::Fixed(*required as u64) {
                return Err(format!("Zipformer port {name} shape mismatch"));
            }
        }
    }
    Ok(())
}

fn expect_weight(report: &OnnxModelReport, name: &str, dims: &[u64]) -> Result<(), String> {
    let weight = report.graph.initializers.iter().find(|weight: &&TensorWeight| weight.name == name)
        .ok_or_else(|| format!("missing Zipformer checkpoint tensor {name}"))?;
    if weight.data_type != 1 || weight.external || weight.dimensions != dims {
        return Err(format!("Zipformer checkpoint tensor {name} has incompatible shape or type"));
    }
    Ok(())
}

pub fn validate_chunk32_checkpoint(
    encoder: &OnnxModelReport,
    decoder: &OnnxModelReport,
    joiner: &OnnxModelReport,
) -> Result<(), String> {
    expect_port(&encoder.graph.inputs, "x", &[None, Some(FEATURES_PER_CHUNK), Some(FEATURE_BINS)], 1)?;
    expect_port(&encoder.graph.inputs, "embed_states", &[None, Some(128), Some(3), Some(19)], 1)?;
    expect_port(&encoder.graph.inputs, "processed_lens", &[None], 7)?;
    expect_port(&encoder.graph.outputs, "encoder_out", &[None, None, Some(ENCODER_DIM)], 1)?;
    for (index, shape) in LAYER_CACHE_SHAPES.iter().enumerate() {
        let ports = &encoder.graph.inputs;
        expect_port(ports, &format!("cached_key_{index}"), &[Some(shape.history), None, Some(shape.key_channels)], 1)?;
        expect_port(ports, &format!("cached_nonlin_attn_{index}"), &[Some(1), None, Some(shape.history), Some(shape.attention_channels)], 1)?;
        for prefix in ["cached_val1", "cached_val2"] {
            expect_port(ports, &format!("{prefix}_{index}"), &[Some(shape.history), None, Some(shape.value_channels)], 1)?;
        }
        for prefix in ["cached_conv1", "cached_conv2"] {
            expect_port(ports, &format!("{prefix}_{index}"), &[None, Some(shape.convolution_channels), Some(shape.convolution_history)], 1)?;
        }
        let outputs = &encoder.graph.outputs;
        expect_port(outputs, &format!("new_cached_key_{index}"), &[None, None, Some(shape.key_channels)], 1)?;
        expect_port(outputs, &format!("new_cached_nonlin_attn_{index}"), &[Some(1), None, None, Some(shape.attention_channels)], 1)?;
        for prefix in ["new_cached_val1", "new_cached_val2"] {
            expect_port(outputs, &format!("{prefix}_{index}"), &[None, None, Some(shape.value_channels)], 1)?;
        }
        for prefix in ["new_cached_conv1", "new_cached_conv2"] {
            expect_port(outputs, &format!("{prefix}_{index}"), &[None, Some(shape.convolution_channels), None], 1)?;
        }
    }
    expect_port(&encoder.graph.outputs, "new_embed_states", &[None, Some(128), None, Some(19)], 1)?;
    expect_port(&encoder.graph.outputs, "new_processed_lens", &[None], 7)?;
    expect_port(&decoder.graph.inputs, "y", &[None, Some(DECODER_CONTEXT)], 7)?;
    expect_port(&decoder.graph.outputs, "decoder_out", &[None, Some(512)], 1)?;
    expect_port(&joiner.graph.inputs, "encoder_out", &[None, Some(512)], 1)?;
    expect_port(&joiner.graph.inputs, "decoder_out", &[None, Some(512)], 1)?;
    expect_port(&joiner.graph.outputs, "logit", &[None, Some(VOCABULARY_SIZE)], 1)?;
    expect_weight(decoder, "decoder.embedding.weight", &[11661, 512])?;
    expect_weight(decoder, "decoder.conv.weight", &[512, 4, 2])?;
    expect_weight(decoder, "decoder_proj.weight", &[512, 512])?;
    expect_weight(decoder, "decoder_proj.bias", &[512])?;
    expect_weight(joiner, "output_linear.weight", &[11661, 512])?;
    expect_weight(joiner, "output_linear.bias", &[11661])?;
    if decoder.graph.initializers.len() != 4 || joiner.graph.initializers.len() != 2 {
        return Err("unexpected decoder or joiner initializer count".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;

    #[test]
    fn cache_shapes_are_derived_from_observed_checkpoint() {
        let cache = Zipformer32Cache::zeros(2, &Device::Cpu).unwrap();
        assert_eq!(cache.layers.len(), 12);
        assert_eq!(cache.layers[0].key.dims(), &[128, 2, 128]);
        assert_eq!(cache.layers[0].nonlinear_attention.dims(), &[1, 2, 128, 144]);
        assert_eq!(cache.layers[6].nonlinear_attention.dims(), &[1, 2, 16, 192]);
        assert_eq!(cache.layers[6].value1.dims(), &[16, 2, 96]);
        assert_eq!(cache.layers[10].convolution2.dims(), &[2, 256, 15]);
        assert_eq!(cache.embedding.dims(), &[2, 128, 3, 19]);
        assert_eq!(cache.processed_lengths.dims(), &[2]);
        assert!(Zipformer32Cache::zeros(0, &Device::Cpu).is_err());
    }

    #[test]
    fn cache_update_is_complete_and_atomic() {
        use std::collections::BTreeMap;
        let device = Device::Cpu;
        let mut cache = Zipformer32Cache::zeros(1, &device).unwrap();
        let mut outputs = BTreeMap::new();
        for (i, shape) in LAYER_CACHE_SHAPES.iter().enumerate() {
            for (name, dims) in [
                ("key", vec![shape.history, 1, shape.key_channels]),
                ("nonlin_attn", vec![1, 1, shape.history, shape.attention_channels]),
                ("val1", vec![shape.history, 1, shape.value_channels]),
                ("val2", vec![shape.history, 1, shape.value_channels]),
                ("conv1", vec![1, shape.convolution_channels, shape.convolution_history]),
                ("conv2", vec![1, shape.convolution_channels, shape.convolution_history]),
            ] {
                outputs.insert(format!("new_cached_{name}_{i}"), Tensor::zeros(dims, DType::F32, &device).unwrap());
            }
        }
        outputs.insert("new_embed_states".into(), Tensor::zeros((1, 128, 3, 19), DType::F32, &device).unwrap());
        outputs.insert("new_processed_lens".into(), Tensor::new(&[32_i64], &device).unwrap());
        cache.replace_from_outputs(&outputs).unwrap();
        assert_eq!(cache.processed_lengths.to_vec1::<i64>().unwrap(), [32]);
        outputs.insert("new_cached_key_0".into(), Tensor::zeros((1, 1, 128), DType::F32, &device).unwrap());
        assert!(cache.replace_from_outputs(&outputs).is_err());
        assert_eq!(cache.processed_lengths.to_vec1::<i64>().unwrap(), [32]);
        outputs.insert("new_cached_key_0".into(), Tensor::zeros((128, 1, 128), DType::I64, &device).unwrap());
        assert!(cache.replace_from_outputs(&outputs).is_err());
        assert_eq!(cache.processed_lengths.to_vec1::<i64>().unwrap(), [32]);
        outputs.remove("new_cached_key_0");
        assert!(cache.replace_from_outputs(&outputs).is_err());
    }

    fn port(name: impl Into<String>, dimensions: &[Option<usize>], dtype: u64) -> TensorPort {
        TensorPort {
            name: name.into(),
            element_type: Some(dtype),
            dimensions: dimensions.iter().map(|dimension| match dimension {
                Some(value) => Dimension::Fixed(*value as u64),
                None => Dimension::Symbol("N".into()),
            }).collect(),
        }
    }

    fn weight(name: &str, dimensions: &[u64]) -> TensorWeight {
        TensorWeight { name: name.into(), dimensions: dimensions.to_vec(), data_type: 1, external: false }
    }

    fn mock_report() -> (OnnxModelReport, OnnxModelReport, OnnxModelReport) {
        use super::super::onnx_inspect::OnnxGraphReport;
        let mut encoder = OnnxModelReport { ir_version: 7, graph: OnnxGraphReport::default() };
        let mut decoder = encoder.clone();
        let mut joiner = encoder.clone();
        encoder.graph.inputs.push(port("x", &[None, Some(77), Some(80)], 1));
        encoder.graph.inputs.push(port("embed_states", &[None, Some(128), Some(3), Some(19)], 1));
        encoder.graph.inputs.push(port("processed_lens", &[None], 7));
        encoder.graph.outputs.push(port("encoder_out", &[None, None, Some(512)], 1));
        encoder.graph.outputs.push(port("new_embed_states", &[None, Some(128), None, Some(19)], 1));
        encoder.graph.outputs.push(port("new_processed_lens", &[None], 7));
        for (index, shape) in LAYER_CACHE_SHAPES.iter().enumerate() {
            for (name, dims) in [
                ("cached_key", vec![Some(shape.history), None, Some(shape.key_channels)]),
                ("cached_nonlin_attn", vec![Some(1), None, Some(shape.history), Some(shape.attention_channels)]),
                ("cached_val1", vec![Some(shape.history), None, Some(shape.value_channels)]),
                ("cached_val2", vec![Some(shape.history), None, Some(shape.value_channels)]),
                ("cached_conv1", vec![None, Some(shape.convolution_channels), Some(shape.convolution_history)]),
                ("cached_conv2", vec![None, Some(shape.convolution_channels), Some(shape.convolution_history)]),
            ] {
                encoder.graph.inputs.push(port(format!("{name}_{index}"), &dims, 1));
                encoder.graph.outputs.push(port(format!("new_{name}_{index}"), &dims, 1));
            }
        }
        decoder.graph.inputs.push(port("y", &[None, Some(2)], 7));
        decoder.graph.outputs.push(port("decoder_out", &[None, Some(512)], 1));
        decoder.graph.initializers = vec![
            weight("decoder.embedding.weight", &[11661, 512]),
            weight("decoder.conv.weight", &[512, 4, 2]),
            weight("decoder_proj.weight", &[512, 512]),
            weight("decoder_proj.bias", &[512]),
        ];
        joiner.graph.inputs.push(port("encoder_out", &[None, Some(512)], 1));
        joiner.graph.inputs.push(port("decoder_out", &[None, Some(512)], 1));
        joiner.graph.outputs.push(port("logit", &[None, Some(11661)], 1));
        joiner.graph.initializers = vec![
            weight("output_linear.weight", &[11661, 512]),
            weight("output_linear.bias", &[11661]),
        ];
        (encoder, decoder, joiner)
    }

    #[test]
    fn checkpoint_contract_rejects_wrong_cache_and_projection() {
        let (mut encoder, mut decoder, joiner) = mock_report();
        validate_chunk32_checkpoint(&encoder, &decoder, &joiner).unwrap();
        encoder.graph.inputs.iter_mut().find(|input| input.name == "cached_key_6")
            .unwrap().dimensions[2] = Dimension::Fixed(128);
        assert!(validate_chunk32_checkpoint(&encoder, &decoder, &joiner).is_err());
        let (encoder, _, joiner) = mock_report();
        decoder.graph.initializers[1].dimensions = vec![512, 1, 2];
        assert!(validate_chunk32_checkpoint(&encoder, &decoder, &joiner).is_err());
        let (mut encoder, decoder, joiner) = mock_report();
        encoder.graph.outputs.iter_mut().find(|output| output.name == "new_cached_nonlin_attn_0")
            .unwrap().element_type = Some(7);
        assert!(validate_chunk32_checkpoint(&encoder, &decoder, &joiner).is_err());
        let (mut encoder, decoder, joiner) = mock_report();
        encoder.graph.outputs.iter_mut().find(|output| output.name == "new_cached_conv1_0")
            .unwrap().dimensions[1] = Dimension::Fixed(999);
        assert!(validate_chunk32_checkpoint(&encoder, &decoder, &joiner).is_err());
    }

}
