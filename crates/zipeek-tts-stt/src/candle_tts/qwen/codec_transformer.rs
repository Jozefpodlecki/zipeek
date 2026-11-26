use candle_core::{D, Module, Result, Tensor};
use candle_nn::{Linear, RmsNorm, VarBuilder};

use super::codec::SpeechDecoderConfig;

struct AudioAttention {
    q: Linear,
    k: Linear,
    v: Linear,
    o: Linear,
    heads: usize,
    head_dim: usize,
}

fn rotate(hidden: &Tensor, theta: f64) -> Result<Tensor> {
    let (_, length, dim) = hidden.dims3()?;
    let half = dim / 2;
    let mut cosine = Vec::with_capacity(length * dim);
    let mut sine = Vec::with_capacity(length * dim);
    for position in 0..length {
        let angles = (0..half)
            .map(|index| {
                let angle = position as f64 / theta.powf(2.0 * index as f64 / dim as f64);
                (angle.cos() as f32, angle.sin() as f32)
            })
            .collect::<Vec<_>>();
        for _ in 0..2 {
            cosine.extend(angles.iter().map(|angle| angle.0));
            sine.extend(angles.iter().map(|angle| angle.1));
        }
    }
    let cosine =
        Tensor::from_vec(cosine, (1, length, dim), hidden.device())?.to_dtype(hidden.dtype())?;
    let sine =
        Tensor::from_vec(sine, (1, length, dim), hidden.device())?.to_dtype(hidden.dtype())?;
    let left = hidden.narrow(D::Minus1, 0, half)?;
    let right = hidden.narrow(D::Minus1, half, half)?;
    let rotated = Tensor::cat(&[&right.neg()?, &left], D::Minus1)?;
    hidden
        .broadcast_mul(&cosine)?
        .broadcast_add(&rotated.broadcast_mul(&sine)?)
}

fn audio_causal_mask(length: usize, window: usize, hidden: &Tensor) -> Result<Tensor> {
    if length == 0 || window == 0 {
        candle_core::bail!("invalid speech codec attention window")
    }
    let mut values = Vec::with_capacity(length * length);
    for query in 0..length {
        for key in 0..length {
            values.push(if key <= query && query - key < window {
                0.0f32
            } else {
                -1.0e9f32
            });
        }
    }
    Tensor::from_vec(values, (1, length, length), hidden.device())?.to_dtype(hidden.dtype())
}

impl AudioAttention {
    fn load(config: &SpeechDecoderConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let projection = config.heads * config.head_dim;
        Ok(Self {
            q: candle_nn::linear_no_bias(config.hidden_size, projection, vb.pp("q_proj"))?,
            k: candle_nn::linear_no_bias(config.hidden_size, projection, vb.pp("k_proj"))?,
            v: candle_nn::linear_no_bias(config.hidden_size, projection, vb.pp("v_proj"))?,
            o: candle_nn::linear_no_bias(projection, config.hidden_size, vb.pp("o_proj"))?,
            heads: config.heads,
            head_dim: config.head_dim,
        })
    }

    fn forward(&self, inputs: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let length = inputs.dim(0)?;
        let shape = (length, self.heads, self.head_dim);
        let q = rotate(
            &self.q.forward(inputs)?.reshape(shape)?.transpose(0, 1)?,
            10_000.0,
        )?;
        let k = rotate(
            &self.k.forward(inputs)?.reshape(shape)?.transpose(0, 1)?,
            10_000.0,
        )?;
        let v = self.v.forward(inputs)?.reshape(shape)?.transpose(0, 1)?;
        let keys = k.transpose(1, 2)?.contiguous()?;
        let attention = q
            .contiguous()?
            .matmul(&keys)?
            .affine((self.head_dim as f64).powf(-0.5), 0.0)?;
        let probabilities = candle_nn::ops::softmax(&attention.broadcast_add(mask)?, D::Minus1)?;
        let output = probabilities
            .contiguous()?
            .matmul(&v.contiguous()?)?
            .transpose(0, 1)?
            .contiguous()?
            .reshape((length, self.heads * self.head_dim))?;
        self.o.forward(&output)
    }
}

struct AudioDecoderBlock {
    attention: AudioAttention,
    input_norm: RmsNorm,
    output_norm: RmsNorm,
    attention_scale: Tensor,
    mlp_scale: Tensor,
    gate: Linear,
    up: Linear,
    down: Linear,
}

impl AudioDecoderBlock {
    fn load(config: &SpeechDecoderConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let mlp = vb.pp("mlp");
        Ok(Self {
            attention: AudioAttention::load(config, vb.pp("self_attn"))?,
            input_norm: candle_nn::rms_norm(config.hidden_size, 1e-5, vb.pp("input_layernorm"))?,
            output_norm: candle_nn::rms_norm(
                config.hidden_size,
                1e-5,
                vb.pp("post_attention_layernorm"),
            )?,
            attention_scale: vb
                .pp("self_attn_layer_scale")
                .get(config.hidden_size, "scale")?,
            mlp_scale: vb.pp("mlp_layer_scale").get(config.hidden_size, "scale")?,
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

    fn forward(&self, input: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let attended = self
            .attention
            .forward(&self.input_norm.forward(input)?, mask)?;
        let input = input.broadcast_add(&attended.broadcast_mul(&self.attention_scale)?)?;
        let normalized = self.output_norm.forward(&input)?;
        let gated = candle_nn::Activation::Silu
            .forward(&self.gate.forward(&normalized)?)?
            .broadcast_mul(&self.up.forward(&normalized)?)?;
        input.broadcast_add(&self.down.forward(&gated)?.broadcast_mul(&self.mlp_scale)?)
    }
}

pub struct AudioDecoderTransformer {
    window: usize,
    input: Linear,
    layers: Vec<AudioDecoderBlock>,
    norm: RmsNorm,
    output: Linear,
}

impl AudioDecoderTransformer {
    pub fn load(config: &SpeechDecoderConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let layers = (0..config.layers)
            .map(|index| AudioDecoderBlock::load(config, vb.pp(format!("layers.{index}"))))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            window: config.sliding_window,
            input: candle_nn::linear(config.latent_dim, config.hidden_size, vb.pp("input_proj"))?,
            layers,
            norm: candle_nn::rms_norm(config.hidden_size, 1e-5, vb.pp("norm"))?,
            output: candle_nn::linear(config.hidden_size, config.latent_dim, vb.pp("output_proj"))?,
        })
    }

    pub fn forward(&self, inputs: &Tensor) -> Result<Tensor> {
        let mut hidden = self.input.forward(inputs)?;
        let mask = audio_causal_mask(hidden.dim(0)?, self.window, &hidden)?;
        for layer in &self.layers {
            hidden = layer.forward(&hidden, &mask)?;
        }
        self.output.forward(&self.norm.forward(&hidden)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_causal_mask_excludes_future_and_outside_window() {
        let device = candle_core::Device::Cpu;
        let input = Tensor::zeros((4, 8), candle_core::DType::F32, &device).unwrap();
        let mask = audio_causal_mask(4, 2, &input).unwrap();
        let values = mask.to_vec3::<f32>().unwrap();
        assert_eq!(values[0][2][2], 0.0);
        assert_eq!(values[0][2][1], 0.0);
        assert!(values[0][2][0] < -1e8);
        assert!(values[0][2][3] < -1e8);
        assert!(audio_causal_mask(4, 0, &input).is_err());
    }

    #[test]
    fn attention_matmul_accepts_transposed_contiguous_keys_and_values() {
        let device = candle_core::Device::Cpu;
        let queries = Tensor::zeros((2, 3, 4), candle_core::DType::F32, &device).unwrap();
        let keys = Tensor::zeros((2, 3, 4), candle_core::DType::F32, &device).unwrap();
        let values = Tensor::ones((2, 3, 4), candle_core::DType::F32, &device).unwrap();
        let scores = queries
            .matmul(&keys.transpose(1, 2).unwrap().contiguous().unwrap())
            .unwrap();
        let output = scores.matmul(&values.contiguous().unwrap()).unwrap();
        assert_eq!(scores.dims(), &[2, 3, 3]);
        assert_eq!(output.dims(), &[2, 3, 4]);
    }

    #[test]
    fn rotary_is_identity_at_position_zero() {
        let device = candle_core::Device::Cpu;
        let tensor = Tensor::new(&[[[1f32, 2., 3., 4.]]], &device).unwrap();
        assert_eq!(
            rotate(&tensor, 10000.0).unwrap().to_vec3::<f32>().unwrap(),
            tensor.to_vec3::<f32>().unwrap()
        );
    }
}
