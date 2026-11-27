use candle_core::{D, DType, Device, Module, Result, Tensor};
use candle_nn::{Embedding, Linear, RmsNorm, VarBuilder};

pub const TEXT_LAYERS: usize = 28;
pub const TEXT_WIDTH: usize = 1024;
pub const TEXT_HEADS: usize = 16;
pub const TEXT_KV_HEADS: usize = 8;
pub const TEXT_HEAD_WIDTH: usize = 128;
pub const TEXT_FFN_WIDTH: usize = 3072;
pub const TEXT_VOCAB: usize = 151_936;
const NORM_EPS: f64 = 1e-6;
const ROPE_THETA: f64 = 1_000_000.0;

struct KvCache {
    keys: Tensor,
    values: Tensor,
}

struct TextAttention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    o_proj: Linear,
    q_norm: RmsNorm,
    k_norm: RmsNorm,
    cache: Option<KvCache>,
}

fn rotary(hidden: &Tensor, start: usize) -> Result<Tensor> {
    let (heads, tokens, width) = hidden.dims3()?;
    if width != TEXT_HEAD_WIDTH {
        candle_core::bail!("invalid Qwen3-ASR rotary head width")
    }
    let half = width / 2;
    let mut cos = Vec::with_capacity(tokens * width);
    let mut sin = Vec::with_capacity(tokens * width);
    for token in start..start + tokens {
        let angles = (0..half)
            .map(|i| token as f64 / ROPE_THETA.powf((2 * i) as f64 / width as f64))
            .collect::<Vec<_>>();
        for _ in 0..2 {
            cos.extend(angles.iter().map(|angle| angle.cos() as f32));
            sin.extend(angles.iter().map(|angle| angle.sin() as f32));
        }
    }
    let cos =
        Tensor::from_vec(cos, (1, tokens, width), hidden.device())?.to_dtype(hidden.dtype())?;
    let sin =
        Tensor::from_vec(sin, (1, tokens, width), hidden.device())?.to_dtype(hidden.dtype())?;
    let a = hidden.narrow(2, 0, half)?;
    let b = hidden.narrow(2, half, half)?;
    let rotated = Tensor::cat(&[&b.neg()?, &a], 2)?;
    let _ = heads;
    hidden
        .broadcast_mul(&cos)?
        .broadcast_add(&rotated.broadcast_mul(&sin)?)
}

impl TextAttention {
    fn load(vb: VarBuilder<'_>) -> Result<Self> {
        Ok(Self {
            q_proj: candle_nn::linear_no_bias(
                TEXT_WIDTH,
                TEXT_HEADS * TEXT_HEAD_WIDTH,
                vb.pp("q_proj"),
            )?,
            k_proj: candle_nn::linear_no_bias(
                TEXT_WIDTH,
                TEXT_KV_HEADS * TEXT_HEAD_WIDTH,
                vb.pp("k_proj"),
            )?,
            v_proj: candle_nn::linear_no_bias(
                TEXT_WIDTH,
                TEXT_KV_HEADS * TEXT_HEAD_WIDTH,
                vb.pp("v_proj"),
            )?,
            o_proj: candle_nn::linear_no_bias(
                TEXT_HEADS * TEXT_HEAD_WIDTH,
                TEXT_WIDTH,
                vb.pp("o_proj"),
            )?,
            q_norm: candle_nn::rms_norm(TEXT_HEAD_WIDTH, NORM_EPS, vb.pp("q_norm"))?,
            k_norm: candle_nn::rms_norm(TEXT_HEAD_WIDTH, NORM_EPS, vb.pp("k_norm"))?,
            cache: None,
        })
    }

    fn reset(&mut self) {
        self.cache = None;
    }

    fn forward(&mut self, hidden: &Tensor, start: usize) -> Result<Tensor> {
        let tokens = hidden.dim(0)?;
        let q = self
            .q_proj
            .forward(hidden)?
            .reshape((tokens, TEXT_HEADS, TEXT_HEAD_WIDTH))?
            .transpose(0, 1)?;
        let k = self
            .k_proj
            .forward(hidden)?
            .reshape((tokens, TEXT_KV_HEADS, TEXT_HEAD_WIDTH))?
            .transpose(0, 1)?;
        let v = self
            .v_proj
            .forward(hidden)?
            .reshape((tokens, TEXT_KV_HEADS, TEXT_HEAD_WIDTH))?
            .transpose(0, 1)?;
        let q = rotary(&self.q_norm.forward(&q)?, start)?;
        let k = rotary(&self.k_norm.forward(&k)?, start)?;
        let (k, v) = if let Some(cache) = &self.cache {
            (
                Tensor::cat(&[&cache.keys, &k], 1)?,
                Tensor::cat(&[&cache.values, &v], 1)?,
            )
        } else {
            (k, v)
        };
        let positions = k.dim(1)?;
        self.cache = Some(KvCache {
            keys: k.clone(),
            values: v.clone(),
        });
        let keys = k
            .unsqueeze(1)?
            .broadcast_as((
                TEXT_KV_HEADS,
                TEXT_HEADS / TEXT_KV_HEADS,
                positions,
                TEXT_HEAD_WIDTH,
            ))?
            .contiguous()?
            .reshape((TEXT_HEADS, positions, TEXT_HEAD_WIDTH))?;
        let values = v
            .unsqueeze(1)?
            .broadcast_as((
                TEXT_KV_HEADS,
                TEXT_HEADS / TEXT_KV_HEADS,
                positions,
                TEXT_HEAD_WIDTH,
            ))?
            .contiguous()?
            .reshape((TEXT_HEADS, positions, TEXT_HEAD_WIDTH))?;
        let logits = q
            .contiguous()?
            .matmul(&keys.transpose(1, 2)?.contiguous()?)?
            .affine(1.0 / (TEXT_HEAD_WIDTH as f64).sqrt(), 0.)?;
        let mut mask = Vec::with_capacity(tokens * positions);
        for query in 0..tokens {
            for key in 0..positions {
                mask.push(if key <= start + query { 0.0f32 } else { -1.0e9 });
            }
        }
        let mask = Tensor::from_vec(mask, (1, tokens, positions), hidden.device())?
            .to_dtype(logits.dtype())?;
        let weights = candle_nn::ops::softmax(&logits.broadcast_add(&mask)?, D::Minus1)?;
        let attended = weights
            .contiguous()?
            .matmul(&values.contiguous()?)?
            .transpose(0, 1)?
            .contiguous()?
            .reshape((tokens, TEXT_HEADS * TEXT_HEAD_WIDTH))?;
        self.o_proj.forward(&attended)
    }
}

struct TextLayer {
    attention: TextAttention,
    input_norm: RmsNorm,
    post_norm: RmsNorm,
    gate: Linear,
    up: Linear,
    down: Linear,
}

impl TextLayer {
    fn load(vb: VarBuilder<'_>) -> Result<Self> {
        let attention = TextAttention::load(vb.pp("self_attn"))?;
        let mlp = vb.pp("mlp");
        Ok(Self {
            attention,
            input_norm: candle_nn::rms_norm(TEXT_WIDTH, NORM_EPS, vb.pp("input_layernorm"))?,
            post_norm: candle_nn::rms_norm(
                TEXT_WIDTH,
                NORM_EPS,
                vb.pp("post_attention_layernorm"),
            )?,
            gate: candle_nn::linear_no_bias(TEXT_WIDTH, TEXT_FFN_WIDTH, mlp.pp("gate_proj"))?,
            up: candle_nn::linear_no_bias(TEXT_WIDTH, TEXT_FFN_WIDTH, mlp.pp("up_proj"))?,
            down: candle_nn::linear_no_bias(TEXT_FFN_WIDTH, TEXT_WIDTH, mlp.pp("down_proj"))?,
        })
    }
    fn reset(&mut self) {
        self.attention.reset();
    }

    fn forward(&mut self, hidden: &Tensor, start: usize) -> Result<Tensor> {
        let attention = self
            .attention
            .forward(&self.input_norm.forward(hidden)?, start)?;
        let hidden = hidden.broadcast_add(&attention)?;
        let normalized = self.post_norm.forward(&hidden)?;
        let gate = candle_nn::Activation::Silu.forward(&self.gate.forward(&normalized)?)?;
        let up = self.up.forward(&normalized)?;
        hidden.broadcast_add(&self.down.forward(&gate.mul(&up)?)?)
    }
}

pub struct QwenAsrTextDecoder {
    tokens: Embedding,
    layers: Vec<TextLayer>,
    norm: RmsNorm,
    head: Linear,
    seen: usize,
}

impl QwenAsrTextDecoder {
    pub fn load(vb: VarBuilder<'_>) -> Result<Self> {
        let model = vb.pp("thinker.model");
        let tokens = candle_nn::embedding(TEXT_VOCAB, TEXT_WIDTH, model.pp("embed_tokens"))?;
        let layers = (0..TEXT_LAYERS)
            .map(|i| TextLayer::load(model.pp(format!("layers.{i}"))))
            .collect::<Result<Vec<_>>>()?;
        let head = candle_nn::linear_no_bias(TEXT_WIDTH, TEXT_VOCAB, vb.pp("thinker.lm_head"))?;
        Ok(Self {
            tokens,
            layers,
            norm: candle_nn::rms_norm(TEXT_WIDTH, NORM_EPS, model.pp("norm"))?,
            head,
            seen: 0,
        })
    }

    pub fn reset(&mut self) {
        self.seen = 0;
        for layer in &mut self.layers {
            layer.reset();
        }
    }

    pub fn embed(&self, ids: &[u32], device: &Device) -> Result<Tensor> {
        let ids = Tensor::new(ids, device)?;
        self.tokens.forward(&ids)
    }

    pub fn forward_embeddings(&mut self, inputs: &Tensor) -> Result<Tensor> {
        let (length, width) = inputs.dims2()?;
        if length == 0 || width != TEXT_WIDTH {
            candle_core::bail!("invalid Qwen3-ASR text decoder input dimensions")
        }
        let mut hidden = inputs.clone();
        for layer in &mut self.layers {
            hidden = layer.forward(&hidden, self.seen)?;
        }
        self.seen += length;
        let final_token = hidden.narrow(0, length - 1, 1)?;
        self.head.forward(&self.norm.forward(&final_token)?)
    }

    pub fn next_token(&mut self, inputs: &Tensor) -> Result<u32> {
        let logits = self.forward_embeddings(inputs)?;
        let logits = logits.to_dtype(DType::F32)?;
        let token = logits.argmax(D::Minus1)?.to_vec1::<u32>()?[0];
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transposed_text_attention_operands_support_batched_matmul() {
        let device = Device::Cpu;
        let tokens = 2;
        let positions = 3;
        let q = Tensor::ones((tokens, TEXT_HEADS, TEXT_HEAD_WIDTH), DType::F32, &device)
            .unwrap()
            .transpose(0, 1)
            .unwrap();
        let keys = Tensor::ones(
            (positions, TEXT_HEADS, TEXT_HEAD_WIDTH),
            DType::F32,
            &device,
        )
        .unwrap()
        .transpose(0, 1)
        .unwrap();
        let scores = q
            .contiguous()
            .unwrap()
            .matmul(&keys.transpose(1, 2).unwrap().contiguous().unwrap())
            .unwrap();
        assert_eq!(scores.dims(), &[TEXT_HEADS, tokens, positions]);
        let values = keys;
        let attended = scores
            .contiguous()
            .unwrap()
            .matmul(&values.contiguous().unwrap())
            .unwrap();
        assert_eq!(attended.dims(), &[TEXT_HEADS, tokens, TEXT_HEAD_WIDTH]);
    }

    #[test]
    fn qwen3_decoder_dimensions_match_checkpoint() {
        assert_eq!(TEXT_LAYERS, 28);
        assert_eq!(TEXT_HEADS * TEXT_HEAD_WIDTH, 2048);
        assert_eq!(TEXT_KV_HEADS * TEXT_HEAD_WIDTH, 1024);
        assert_eq!(TEXT_VOCAB, 151_936);
    }
    #[test]
    fn rotary_preserves_zero_position() {
        let input = Tensor::arange(0f32, 128f32, &Device::Cpu)
            .unwrap()
            .reshape((1, 1, 128))
            .unwrap();
        let output = rotary(&input, 0).unwrap();
        assert_eq!(
            input.to_vec3::<f32>().unwrap(),
            output.to_vec3::<f32>().unwrap()
        );
    }
}
