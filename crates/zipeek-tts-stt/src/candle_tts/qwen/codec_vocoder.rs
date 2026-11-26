use candle_core::{D, Module, Result, Tensor};
use candle_nn::{
    Conv1d, Conv1dConfig, ConvTranspose1d, ConvTranspose1dConfig, LayerNorm, Linear, VarBuilder,
};

use super::codec::SpeechDecoderConfig;

pub struct CausalConv {
    convolution: Conv1d,
    left_padding: usize,
    stride: usize,
    dilation: usize,
    kernel: usize,
}

impl CausalConv {
    pub fn load(
        input_channels: usize,
        output_channels: usize,
        kernel: usize,
        dilation: usize,
        groups: usize,
        vb: VarBuilder<'_>,
    ) -> Result<Self> {
        let config = Conv1dConfig {
            dilation,
            groups,
            ..Conv1dConfig::default()
        };
        Ok(Self {
            convolution: candle_nn::conv1d(
                input_channels,
                output_channels,
                kernel,
                config,
                vb.pp("conv"),
            )?,
            left_padding: (kernel - 1) * dilation,
            stride: 1,
            dilation,
            kernel,
        })
    }

    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let (_, _, length) = input.dims3()?;
        let effective_kernel = (self.kernel - 1) * self.dilation + 1;
        let unpadded_frames = length + self.left_padding - effective_kernel;
        let extra_padding = (self.stride - unpadded_frames % self.stride) % self.stride;
        let padded = input.pad_with_zeros(2, self.left_padding, extra_padding)?;
        self.convolution.forward(&padded)
    }
}

pub struct CausalTranspose {
    convolution: ConvTranspose1d,
    trim: usize,
}

impl CausalTranspose {
    fn load(
        input: usize,
        output: usize,
        kernel: usize,
        stride: usize,
        vb: VarBuilder<'_>,
    ) -> Result<Self> {
        let config = ConvTranspose1dConfig {
            stride,
            ..ConvTranspose1dConfig::default()
        };
        Ok(Self {
            convolution: candle_nn::conv_transpose1d(input, output, kernel, config, vb.pp("conv"))?,
            trim: kernel - stride,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let output = self.convolution.forward(input)?;
        output.narrow(D::Minus1, 0, output.dim(D::Minus1)? - self.trim)
    }
}

struct SnakeBeta {
    alpha: Tensor,
    beta: Tensor,
}

impl SnakeBeta {
    fn load(channels: usize, vb: VarBuilder<'_>) -> Result<Self> {
        Ok(Self {
            alpha: vb.get(channels, "alpha")?.reshape((1, channels, 1))?,
            beta: vb.get(channels, "beta")?.reshape((1, channels, 1))?,
        })
    }

    fn forward(&self, inputs: &Tensor) -> Result<Tensor> {
        let alpha = self.alpha.exp()?;
        let beta = self.beta.exp()?.affine(1.0, 1e-9)?.recip()?;
        let periodic = inputs.broadcast_mul(&alpha)?.sin()?.sqr()?;
        inputs.broadcast_add(&periodic.broadcast_mul(&beta)?)
    }
}

struct ConvNeXt {
    depthwise: CausalConv,
    norm: LayerNorm,
    up: Linear,
    down: Linear,
    gamma: Tensor,
}

impl ConvNeXt {
    fn load(channels: usize, vb: VarBuilder<'_>) -> Result<Self> {
        Ok(Self {
            depthwise: CausalConv::load(channels, channels, 7, 1, channels, vb.pp("dwconv"))?,
            norm: candle_nn::layer_norm(channels, 1e-6, vb.pp("norm"))?,
            up: candle_nn::linear(channels, channels * 4, vb.pp("pwconv1"))?,
            down: candle_nn::linear(channels * 4, channels, vb.pp("pwconv2"))?,
            gamma: vb.get(channels, "gamma")?,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let hidden = self.depthwise.forward(input)?.transpose(1, 2)?;
        let hidden = self.norm.forward(&hidden)?;
        let hidden = self.up.forward(&hidden)?.gelu()?;
        let hidden = self
            .down
            .forward(&hidden)?
            .broadcast_mul(&self.gamma)?
            .transpose(1, 2)?;
        input.broadcast_add(&hidden)
    }
}

struct ResidualUnit {
    activation_a: SnakeBeta,
    conv_a: CausalConv,
    activation_b: SnakeBeta,
    conv_b: CausalConv,
}

impl ResidualUnit {
    fn load(channels: usize, dilation: usize, vb: VarBuilder<'_>) -> Result<Self> {
        Ok(Self {
            activation_a: SnakeBeta::load(channels, vb.pp("act1"))?,
            conv_a: CausalConv::load(channels, channels, 7, dilation, 1, vb.pp("conv1"))?,
            activation_b: SnakeBeta::load(channels, vb.pp("act2"))?,
            conv_b: CausalConv::load(channels, channels, 1, 1, 1, vb.pp("conv2"))?,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let x = self.conv_a.forward(&self.activation_a.forward(input)?)?;
        let x = self.conv_b.forward(&self.activation_b.forward(&x)?)?;
        input.broadcast_add(&x)
    }
}

struct Upsampling {
    transpose: CausalTranspose,
    refinement: ConvNeXt,
}

impl Upsampling {
    fn load(channels: usize, factor: usize, vb: VarBuilder<'_>) -> Result<Self> {
        Ok(Self {
            transpose: CausalTranspose::load(channels, channels, factor, factor, vb.pp("0"))?,
            refinement: ConvNeXt::load(channels, vb.pp("1"))?,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        self.refinement.forward(&self.transpose.forward(input)?)
    }
}

struct DecoderUpsample {
    activation: SnakeBeta,
    transpose: CausalTranspose,
    residuals: Vec<ResidualUnit>,
}

impl DecoderUpsample {
    fn load(input: usize, factor: usize, vb: VarBuilder<'_>) -> Result<Self> {
        let output = input / 2;
        let block = vb.pp("block");
        let residuals = [1, 3, 9]
            .into_iter()
            .enumerate()
            .map(|(index, dilation)| {
                ResidualUnit::load(output, dilation, block.pp(format!("{}", index + 2)))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            activation: SnakeBeta::load(input, block.pp("0"))?,
            transpose: CausalTranspose::load(input, output, 2 * factor, factor, block.pp("1"))?,
            residuals,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut hidden = self.transpose.forward(&self.activation.forward(input)?)?;
        for residual in &self.residuals {
            hidden = residual.forward(&hidden)?;
        }
        Ok(hidden)
    }
}

pub struct SpeechVocoder {
    upsampling: Vec<Upsampling>,
    initial: CausalConv,
    stages: Vec<DecoderUpsample>,
    activation: SnakeBeta,
    final_conv: CausalConv,
}

impl SpeechVocoder {
    pub fn load(config: &SpeechDecoderConfig, vb: VarBuilder<'_>) -> Result<Self> {
        let upsampling = config
            .upsampling_ratios
            .iter()
            .enumerate()
            .map(|(index, factor)| {
                Upsampling::load(
                    config.latent_dim,
                    *factor,
                    vb.pp(format!("upsample.{index}")),
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let sub = vb.pp("decoder");
        let initial =
            CausalConv::load(config.latent_dim, config.decoder_dim, 7, 1, 1, sub.pp("0"))?;
        let mut channels = config.decoder_dim;
        let mut stages = Vec::new();
        for (index, factor) in config.upsample_rates.iter().enumerate() {
            stages.push(DecoderUpsample::load(
                channels,
                *factor,
                sub.pp(format!("{}", index + 1)),
            )?);
            channels /= 2;
        }
        let last = config.upsample_rates.len() + 1;
        Ok(Self {
            upsampling,
            initial,
            stages,
            activation: SnakeBeta::load(channels, sub.pp(format!("{last}")))?,
            final_conv: CausalConv::load(channels, 1, 7, 1, 1, sub.pp(format!("{}", last + 1)))?,
        })
    }

    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut hidden = input.clone();
        for block in &self.upsampling {
            hidden = block.forward(&hidden)?;
        }
        hidden = self.initial.forward(&hidden)?;
        for block in &self.stages {
            hidden = block.forward(&hidden)?;
        }
        self.final_conv
            .forward(&self.activation.forward(&hidden)?)?
            .clamp(-1.0, 1.0)
    }
}
