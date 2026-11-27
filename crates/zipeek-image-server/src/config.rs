use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        let backend = BackendKind::parse(
            &std::env::var("ZIPEEK_IMAGE_BACKEND").unwrap_or_else(|_| "sdxl".into()),
        )?;
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

impl BackendKind {
    fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match value {
            "sdxl" => Ok(Self::Sdxl),
            "placeholder" => Ok(Self::Placeholder),
            _ => Err("ZIPEEK_IMAGE_BACKEND must be 'sdxl' or 'placeholder'".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BackendKind;

    #[test]
    fn explicit_backend_modes_do_not_confuse_preview_with_inference() {
        assert_eq!(BackendKind::parse("sdxl").unwrap(), BackendKind::Sdxl);
        assert_eq!(
            BackendKind::parse("placeholder").unwrap(),
            BackendKind::Placeholder
        );
        assert!(BackendKind::parse("unknown").is_err());
    }
}
