use candle_core::{D, Module, Result, Tensor};
use candle_nn::{Linear, VarBuilder};

pub struct CausalAttentionCache {
    keys: Tensor,
    values: Tensor,
}

pub struct ChunkCausalAttention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    output_proj: Linear,
    heads: usize,
    head_dim: usize,
    max_left_frames: usize,
    cache: Option<CausalAttentionCache>,
}

impl ChunkCausalAttention {
    pub fn load(
        width: usize,
        heads: usize,
        max_left_frames: usize,
        vb: VarBuilder<'_>,
    ) -> Result<Self> {
        if width == 0 || heads == 0 || !width.is_multiple_of(heads) || max_left_frames == 0 {
            candle_core::bail!("invalid causal streaming attention dimensions")
        }
        Ok(Self {
            q_proj: candle_nn::linear(width, width, vb.pp("q_proj"))?,
            k_proj: candle_nn::linear(width, width, vb.pp("k_proj"))?,
            v_proj: candle_nn::linear(width, width, vb.pp("v_proj"))?,
            output_proj: candle_nn::linear(width, width, vb.pp("output_proj"))?,
            heads,
            head_dim: width / heads,
            max_left_frames,
            cache: None,
        })
    }

    pub fn reset(&mut self) {
        self.cache = None;
    }

    pub fn cached_frames(&self) -> usize {
        self.cache.as_ref().map_or(0, |cache| cache.keys.dims()[1])
    }

    pub fn forward_chunk(&mut self, new_frames: &Tensor) -> Result<Tensor> {
        let (frames, width) = new_frames.dims2()?;
        if frames == 0 || width != self.heads * self.head_dim {
            candle_core::bail!("invalid input shape for chunk causal attention")
        }
        let reshape_heads = |tensor: Tensor| -> Result<Tensor> {
            tensor
                .reshape((frames, self.heads, self.head_dim))?
                .transpose(0, 1)?
                .contiguous()
        };
        let queries = reshape_heads(self.q_proj.forward(new_frames)?)?;
        let new_keys = reshape_heads(self.k_proj.forward(new_frames)?)?;
        let new_values = reshape_heads(self.v_proj.forward(new_frames)?)?;
        let (keys, values, previous) = match &self.cache {
            Some(cache) => (
                Tensor::cat(&[&cache.keys, &new_keys], 1)?,
                Tensor::cat(&[&cache.values, &new_values], 1)?,
                cache.keys.dim(1)?,
            ),
            None => (new_keys, new_values, 0),
        };
        let total = previous + frames;
        let mut mask = Vec::with_capacity(frames * total);
        for query in 0..frames {
            for key in 0..total {
                mask.push(if key <= previous + query { 0.0f32 } else { -1e9f32 });
            }
        }
        let causal = Tensor::from_vec(mask, (frames, total), new_frames.device())?
            .to_dtype(new_frames.dtype())?;
        let logits = queries
            .matmul(&keys.transpose(1, 2)?.contiguous()?)?
            .affine(1.0 / (self.head_dim as f64).sqrt(), 0.0)?
            .broadcast_add(&causal)?;
        let probabilities = candle_nn::ops::softmax(&logits, D::Minus1)?;
        let attended = probabilities
            .contiguous()?
            .matmul(&values.contiguous()?)?
            .transpose(0, 1)?
            .contiguous()?
            .reshape((frames, width))?;
        let start = total.saturating_sub(self.max_left_frames);
        self.cache = Some(CausalAttentionCache {
            keys: keys.narrow(1, start, total - start)?.contiguous()?,
            values: values.narrow(1, start, total - start)?.contiguous()?,
        });
        self.output_proj.forward(&attended)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{DType, Device};

    #[test]
    fn causal_attention_cache_is_bounded_across_chunks() {
        let device = Device::Cpu;
        let vb = VarBuilder::zeros(DType::F32, &device);
        let mut attention = ChunkCausalAttention::load(8, 2, 3, vb).unwrap();
        let chunk = Tensor::zeros((2, 8), DType::F32, &device).unwrap();
        assert_eq!(attention.forward_chunk(&chunk).unwrap().dims(), &[2, 8]);
        assert_eq!(attention.cached_frames(), 2);
        assert_eq!(attention.forward_chunk(&chunk).unwrap().dims(), &[2, 8]);
        assert_eq!(attention.cached_frames(), 3);
        attention.reset();
        assert_eq!(attention.cached_frames(), 0);
    }
}
