use candle_core::{DType, Device, Tensor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePreference {
    Cpu,
    Cuda(usize),
}

impl DevicePreference {
    pub fn from_env() -> Result<Self, &'static str> {
        match std::env::var("ZIPEEK_CANDLE_DEVICE").as_deref() {
            Ok("cpu") => Ok(Self::Cpu),
            Ok("cuda") | Err(std::env::VarError::NotPresent) => Ok(Self::Cuda(0)),
            _ => Err("ZIPEEK_CANDLE_DEVICE must be cpu or cuda"),
        }
    }

    pub fn initialize(self) -> Result<Device, candle_core::Error> {
        let device = match self {
            Self::Cpu => Device::Cpu,
            Self::Cuda(index) => Device::new_cuda(index)?,
        };
        let _ = Tensor::zeros((1, 1), DType::F32, &device)?;
        Ok(device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_runtime_executes_real_candle_tensor_operations() {
        let device = DevicePreference::Cpu.initialize().unwrap();
        let tensor = Tensor::ones((2, 3), DType::F32, &device).unwrap();
        let result = tensor.sum_all().unwrap().to_scalar::<f32>().unwrap();
        assert_eq!(result, 6.0);
    }
}
