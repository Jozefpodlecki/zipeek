use candle_core::{D, Module, Result, Tensor};
use candle_nn::{Linear, RmsNorm, VarBuilder};

use super::config::DecoderConfig;

const NORM_EPS: f64 = 1e-6;
const ROPE_THETA: f64 = 1_000_000.0;

pub struct AttentionCache {
    keys: Tensor,
    values: Tensor,
}

pub struct CausalAttention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    o_proj: Linear,
    q_norm: RmsNorm,
    k_norm: RmsNorm,
    head_dim: usize,
    query_heads: usize,
    kv_heads: usize,
    cache: Option<AttentionCache>,
    rotary_tables: Option<(Tensor, Tensor)>,
}

fn rotary_tables(
    head_dim: usize,
    positions: usize,
    device: &candle_core::Device,
    dtype: candle_core::DType,
) -> Result<(Tensor, Tensor)> {
    if head_dim == 0 || head_dim % 2 != 0 || positions == 0 {
        candle_core::bail!("invalid Qwen3-TTS rotary dimensions")
    }
    let half = head_dim / 2;
    let inverse_frequencies = (0..half)
        .map(|index| ROPE_THETA.powf(-((2 * index) as f64) / head_dim as f64))
        .collect::<Vec<_>>();
    let mut cos = Vec::with_capacity(positions * head_dim);
    let mut sin = Vec::with_capacity(positions * head_dim);
    for position in 0..positions {
        for _ in 0..2 {
            for frequency in &inverse_frequencies {
                let angle = position as f64 * frequency;
                cos.push(angle.cos() as f32);
                sin.push(angle.sin() as f32);
            }
        }
    }
    Ok((
        Tensor::from_vec(cos, (1, positions, head_dim), device)?.to_dtype(dtype)?,
        Tensor::from_vec(sin, (1, positions, head_dim), device)?.to_dtype(dtype)?,
    ))
}

fn apply_rotary(hidden: &Tensor, start: usize, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let (_, length, head_dim) = hidden.dims3()?;
    let half = head_dim / 2;
    let cos = cos.narrow(1, start, length)?;
    let sin = sin.narrow(1, start, length)?;
    let first = hidden.narrow(2, 0, half)?;
    let second = hidden.narrow(2, half, half)?;
    let rotated = Tensor::cat(&[&second.neg()?, &first], 2)?;
    hidden
        .broadcast_mul(&cos)?
        .broadcast_add(&rotated.broadcast_mul(&sin)?)
}

#[cfg(test)]
fn rotary_embedding(hidden: &Tensor, start: usize) -> Result<Tensor> {
    let (_, length, head_dim) = hidden.dims3()?;
    let (cos, sin) = rotary_tables(head_dim, start + length, hidden.device(), hidden.dtype())?;
    apply_rotary(hidden, start, &cos, &sin)
}

impl CausalAttention {
    fn load(config: &DecoderConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let hidden = config.hidden_size;
        let head_dim = config.head_dim;
        Ok(Self {
            q_proj: candle_nn::linear_no_bias(
                hidden,
                config.query_heads * head_dim,
                vb.pp("q_proj"),
            )?,
            k_proj: candle_nn::linear_no_bias(hidden, config.kv_heads * head_dim, vb.pp("k_proj"))?,
            v_proj: candle_nn::linear_no_bias(hidden, config.kv_heads * head_dim, vb.pp("v_proj"))?,
            o_proj: candle_nn::linear_no_bias(
                config.query_heads * head_dim,
                hidden,
                vb.pp("o_proj"),
            )?,
            q_norm: candle_nn::rms_norm(head_dim, NORM_EPS, vb.pp("q_norm"))?,
            k_norm: candle_nn::rms_norm(head_dim, NORM_EPS, vb.pp("k_norm"))?,
            head_dim,
            query_heads: config.query_heads,
            kv_heads: config.kv_heads,
            cache: None,
            rotary_tables: None,
        })
    }

    fn forward(&mut self, hidden: &Tensor, start: usize) -> Result<Tensor> {
        let length = hidden.dim(0)?;
        let q = self
            .q_proj
            .forward(hidden)?
            .reshape((length, self.query_heads, self.head_dim))?
            .transpose(0, 1)?;
        let k = self
            .k_proj
            .forward(hidden)?
            .reshape((length, self.kv_heads, self.head_dim))?
            .transpose(0, 1)?;
        let v = self
            .v_proj
            .forward(hidden)?
            .reshape((length, self.kv_heads, self.head_dim))?
            .transpose(0, 1)?;
        let required_positions = start
            .checked_add(length)
            .ok_or_else(|| candle_core::Error::Msg("rotary position overflow".into()))?;
        let needs_new_tables = match &self.rotary_tables {
            Some((cos, _)) => cos.dim(1)? < required_positions,
            None => true,
        };
        if needs_new_tables {
            let capacity = required_positions.next_power_of_two().max(128);
            self.rotary_tables = Some(rotary_tables(
                self.head_dim,
                capacity,
                hidden.device(),
                hidden.dtype(),
            )?);
        }
        let (cos, sin) = self.rotary_tables.as_ref().expect("rotary tables initialized");
        let q = apply_rotary(&self.q_norm.forward(&q)?, start, cos, sin)?;
        let k = apply_rotary(&self.k_norm.forward(&k)?, start, cos, sin)?;
        let (k, v) = if let Some(cache) = &self.cache {
            (
                Tensor::cat(&[&cache.keys, &k], 1)?,
                Tensor::cat(&[&cache.values, &v], 1)?,
            )
        } else {
            (k, v)
        };
        let sequence = k.dim(1)?;
        self.cache = Some(AttentionCache {
            keys: k.clone(),
            values: v.clone(),
        });
        let groups = self.query_heads / self.kv_heads;
        let keys = k
            .unsqueeze(1)?
            .broadcast_as((self.kv_heads, groups, sequence, self.head_dim))?
            .contiguous()?
            .reshape((self.query_heads, sequence, self.head_dim))?;
        let values = v
            .unsqueeze(1)?
            .broadcast_as((self.kv_heads, groups, sequence, self.head_dim))?
            .contiguous()?
            .reshape((self.query_heads, sequence, self.head_dim))?;
        let logits = q
            .matmul(&keys.transpose(1, 2)?)?
            .affine(1.0 / (self.head_dim as f64).sqrt(), 0.0)?;
        let masked_logits = if length == 1 {
            logits
        } else {
            let mut mask = Vec::with_capacity(length * sequence);
            for query in 0..length {
                for key in 0..sequence {
                    mask.push(if key <= start + query { 0.0f32 } else { -1.0e9 });
                }
            }
            let mask = Tensor::from_vec(mask, (1, length, sequence), hidden.device())?
                .to_dtype(logits.dtype())?;
            logits.broadcast_add(&mask)?
        };
        let weights = candle_nn::ops::softmax(&masked_logits, D::Minus1)?;
        let values = weights
            .matmul(&values)?
            .transpose(0, 1)?
            .contiguous()?
            .reshape((length, self.query_heads * self.head_dim))?;
        self.o_proj.forward(&values)
    }
}

pub struct DecoderLayer {
    attention: CausalAttention,
    input_norm: RmsNorm,
    post_norm: RmsNorm,
    gate: Linear,
    up: Linear,
    down: Linear,
}

impl DecoderLayer {
    fn load(config: &DecoderConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let mlp = vb.pp("mlp");
        Ok(Self {
            attention: CausalAttention::load(config, vb.pp("self_attn"))?,
            input_norm: candle_nn::rms_norm(
                config.hidden_size,
                NORM_EPS,
                vb.pp("input_layernorm"),
            )?,
            post_norm: candle_nn::rms_norm(
                config.hidden_size,
                NORM_EPS,
                vb.pp("post_attention_layernorm"),
            )?,
            gate: candle_nn::linear_no_bias(
                config.hidden_size,
                config.intermediate_size,
                mlp.pp("gate_proj"),
            )?,
            up: candle_nn::linear_no_bias(
                config.hidden_size,
                config.intermediate_size,
                mlp.pp("up_proj"),
            )?,
            down: candle_nn::linear_no_bias(
                config.intermediate_size,
                config.hidden_size,
                mlp.pp("down_proj"),
            )?,
        })
    }

    fn forward(&mut self, hidden: &Tensor, position: usize) -> Result<Tensor> {
        let attended = self
            .attention
            .forward(&self.input_norm.forward(hidden)?, position)?;
        let hidden = hidden.broadcast_add(&attended)?;
        let normalized = self.post_norm.forward(&hidden)?;
        let gate = candle_nn::Activation::Silu.forward(&self.gate.forward(&normalized)?)?;
        let up = self.up.forward(&normalized)?;
        hidden.broadcast_add(&self.down.forward(&gate.mul(&up)?)?)
    }
}

pub struct CausalTransformer {
    layers: Vec<DecoderLayer>,
    norm: RmsNorm,
    hidden_size: usize,
    position: usize,
}

impl CausalTransformer {
    pub fn load(config: &DecoderConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let layers = (0..config.layers)
            .map(|index| DecoderLayer::load(config, vb.pp(format!("layers.{index}"))))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            layers,
            norm: candle_nn::rms_norm(config.hidden_size, NORM_EPS, vb.pp("norm"))?,
            hidden_size: config.hidden_size,
            position: 0,
        })
    }

    pub fn forward(&mut self, input: &Tensor) -> Result<Tensor> {
        let (length, width) = input.dims2()?;
        if length == 0 || width != self.hidden_size {
            candle_core::bail!("Qwen3-TTS decoder expects nonempty [tokens, hidden_size]")
        }
        let mut hidden = input.clone();
        for layer in &mut self.layers {
            hidden = layer.forward(&hidden, self.position)?;
        }
        self.position += length;
        self.norm.forward(&hidden)
    }

    pub fn reset(&mut self) {
        self.position = 0;
        for layer in &mut self.layers {
            layer.attention.cache = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotary_cached_slice_matches_fresh_computation() {
        let device = candle_core::Device::Cpu;
        let hidden = Tensor::new(&[[0.5f32, -1.0, 0.25, 2.0]], &device)
            .unwrap()
            .reshape((1, 1, 4))
            .unwrap();
        let cached = rotary_tables(4, 128, &device, candle_core::DType::F32).unwrap();
        for position in [0, 1, 17, 127] {
            let result = apply_rotary(&hidden, position, &cached.0, &cached.1).unwrap();
            let reference = rotary_embedding(&hidden, position).unwrap();
            assert_eq!(result.to_vec3::<f32>().unwrap(), reference.to_vec3::<f32>().unwrap());
        }
    }

    #[test]
    fn rotary_preserves_zero_position() {
        let device = candle_core::Device::Cpu;
        let original = Tensor::new(&[[1.0f32, 2.0, 3.0, 4.0]], &device)
            .unwrap()
            .reshape((1, 1, 4))
            .unwrap();
        let rotated = rotary_embedding(&original, 0).unwrap();
        assert_eq!(
            rotated.to_vec3::<f32>().unwrap(),
            original.to_vec3::<f32>().unwrap()
        );
    }
}
