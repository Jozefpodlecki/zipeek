use candle_core::{D, Device, Module, Result, Tensor};
use candle_nn::{Conv1d, Embedding, Linear, VarBuilder};

use super::zipformer32::VOCABULARY_SIZE;

pub struct Zipformer32Decoder {
    embedding: Embedding,
    convolution: Conv1d,
    projection: Linear,
}

impl Zipformer32Decoder {
    pub fn load(vb: VarBuilder<'_>) -> Result<Self> {
        let embedding = candle_nn::embedding(VOCABULARY_SIZE, 512, vb.pp("decoder.embedding"))?;
        let convolution = candle_nn::conv1d_no_bias(
            512,
            512,
            2,
            candle_nn::Conv1dConfig { groups: 128, ..Default::default() },
            vb.pp("decoder.conv"),
        )?;
        let projection = candle_nn::linear(512, 512, vb.pp("decoder_proj"))?;
        Ok(Self { embedding, convolution, projection })
    }

    pub fn forward(&self, token_context: &[u32; 2], device: &Device) -> Result<Tensor> {
        if token_context.iter().any(|token| *token as usize >= VOCABULARY_SIZE) {
            candle_core::bail!("streaming decoder context token outside vocabulary")
        }
        let ids = Tensor::from_vec(token_context.to_vec(), (1, 2), device)?;
        let embedding = self.embedding.forward(&ids)?;
        let conv = self.convolution.forward(&embedding.transpose(1, 2)?.contiguous()?)?;
        self.projection.forward(&conv.squeeze(2)?.relu()?)
    }
}

pub struct Zipformer32Joiner {
    output: Linear,
}

impl Zipformer32Joiner {
    pub fn load(vb: VarBuilder<'_>) -> Result<Self> {
        Ok(Self { output: candle_nn::linear(512, VOCABULARY_SIZE, vb.pp("output_linear"))? })
    }

    pub fn logits(&self, encoded: &Tensor, decoded: &Tensor) -> Result<Tensor> {
        if encoded.dims() != decoded.dims() || *encoded.dims().last().unwrap_or(&0) != 512 {
            candle_core::bail!("Zipformer joiner requires matching 512-dimensional inputs")
        }
        self.output.forward(&(encoded + decoded)?.tanh()?)
    }

    pub fn greedy_token(&self, encoded: &Tensor, decoded: &Tensor) -> Result<u32> {
        self.logits(encoded, decoded)?.argmax(D::Minus1)?.flatten_all()?.to_scalar::<u32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{DType, Device};

    #[test]
    fn checkpoint_named_decoder_and_joiner_have_correct_output_shapes() {
        let device = Device::Cpu;
        let vb = VarBuilder::zeros(DType::F32, &device);
        let decoder = Zipformer32Decoder::load(vb.clone()).unwrap();
        let joiner = Zipformer32Joiner::load(vb).unwrap();
        let decoded = decoder.forward(&[0, 2], &device).unwrap();
        assert_eq!(decoded.dims(), &[1, 512]);
        let encoded = Tensor::zeros((1, 512), DType::F32, &device).unwrap();
        assert_eq!(joiner.logits(&encoded, &decoded).unwrap().dims(), &[1, VOCABULARY_SIZE]);
        assert!(decoder.forward(&[VOCABULARY_SIZE as u32, 0], &device).is_err());
    }
}
