use candle_core::{D, Module, Result, Tensor};
use candle_nn::{Conv1d, Embedding, Linear, VarBuilder};

use super::config::TransducerConfig;

pub struct StatelessDecoder {
    embedding: Embedding,
    convolution: Option<Conv1d>,
    context_size: usize,
    blank_id: u32,
}

impl StatelessDecoder {
    pub fn load(config: &TransducerConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let embedding = candle_nn::embedding(
            config.vocabulary_size,
            config.decoder_dim,
            vb.pp("embedding"),
        )?;
        let convolution = if config.context_size == 2 {
            Some(candle_nn::conv1d(
                config.decoder_dim,
                config.decoder_dim,
                config.context_size,
                candle_nn::Conv1dConfig::default(),
                vb.pp("conv"),
            )?)
        } else {
            None
        };
        Ok(Self {
            embedding,
            convolution,
            context_size: config.context_size,
            blank_id: config.blank_id,
        })
    }

    pub fn forward(&self, recent_tokens: &[u32], device: &candle_core::Device) -> Result<Tensor> {
        let mut context = vec![self.blank_id; self.context_size];
        let tail = recent_tokens.len().min(self.context_size);
        context[self.context_size - tail..]
            .copy_from_slice(&recent_tokens[recent_tokens.len() - tail..]);
        let indices = Tensor::from_vec(context, (1, self.context_size), device)?;
        let embeddings = self.embedding.forward(&indices)?;
        match &self.convolution {
            Some(convolution) => convolution
                .forward(&embeddings.transpose(1, 2)?.contiguous()?)?
                .squeeze(2)?
                .squeeze(0),
            None => embeddings.squeeze(0)?.squeeze(0),
        }
    }
}

pub struct Joiner {
    encoder_proj: Linear,
    decoder_proj: Linear,
    output_linear: Linear,
}

impl Joiner {
    pub fn load(config: &TransducerConfig, vb: VarBuilder<'_>) -> Result<Self> {
        Ok(Self {
            encoder_proj: candle_nn::linear(
                config.encoder_dim,
                config.joiner_dim,
                vb.pp("encoder_proj"),
            )?,
            decoder_proj: candle_nn::linear(
                config.decoder_dim,
                config.joiner_dim,
                vb.pp("decoder_proj"),
            )?,
            output_linear: candle_nn::linear(
                config.joiner_dim,
                config.vocabulary_size,
                vb.pp("output_linear"),
            )?,
        })
    }

    pub fn logits(&self, encoder_frame: &Tensor, decoder_state: &Tensor) -> Result<Tensor> {
        let encoder = self.encoder_proj.forward(encoder_frame)?;
        let decoder = self.decoder_proj.forward(decoder_state)?;
        let joined = (&encoder + &decoder)?.tanh()?;
        self.output_linear.forward(&joined)
    }
}

pub struct CandleTransducerHead {
    decoder: StatelessDecoder,
    joiner: Joiner,
}

impl CandleTransducerHead {
    pub fn load(config: &TransducerConfig, vb: VarBuilder<'_>) -> Result<Self> {
        Ok(Self {
            decoder: StatelessDecoder::load(config, vb.pp("decoder"))?,
            joiner: Joiner::load(config, vb.pp("joiner"))?,
        })
    }

    pub fn next_token(
        &self,
        encoder_frame: &Tensor,
        recent_tokens: &[u32],
    ) -> Result<u32> {
        let decoder_state = self
            .decoder
            .forward(recent_tokens, encoder_frame.device())?;
        let logits = self.joiner.logits(encoder_frame, &decoder_state)?;
        Ok(logits.argmax(D::Minus1)?.to_scalar::<u32>()?)
    }
}
