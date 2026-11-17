use std::{net::SocketAddr, path::PathBuf};

pub enum BackendKind {
    Placeholder,
    Sdxl,
}

pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub max_pending: usize,
    pub backend: BackendKind,
    pub model_dir: PathBuf,
    pub steps: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let bind = std::env::var("ZIPEEK_IMAGE_BIND")
            .unwrap_or_else(|_| "127.0.0.1:1431".into())
            .parse::<SocketAddr>()?;
        if !bind.ip().is_loopback() {
            return Err("ZIPEEK_IMAGE_BIND must use a loopback address (no public authentication configured)".into());
        }
        let data_dir = std::env::var_os("ZIPEEK_IMAGE_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/zipeek-images"));
        let backend = match std::env::var("ZIPEEK_IMAGE_BACKEND")
            .unwrap_or_else(|_| "placeholder".into())
            .as_str()
        {
            "placeholder" => BackendKind::Placeholder,
            "sdxl" => BackendKind::Sdxl,
            _ => return Err("ZIPEEK_IMAGE_BACKEND must be 'placeholder' or 'sdxl'".into()),
        };
        let model_dir = std::env::var_os("ZIPEEK_IMAGE_MODEL_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .join("models")
                    .join("sdxl")
            });
        let steps = std::env::var("ZIPEEK_IMAGE_STEPS")
            .unwrap_or_else(|_| "20".into())
            .parse::<usize>()?;
        if !(1..=100).contains(&steps) {
            return Err("ZIPEEK_IMAGE_STEPS must be between 1 and 100".into());
        }
        Ok(Self {
            bind,
            data_dir,
            max_pending: 4,
            backend,
            model_dir,
            steps,
        })
    }
}
