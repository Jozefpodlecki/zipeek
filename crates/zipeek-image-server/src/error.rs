use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub &'static str);

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(ErrorBody { error: self.1 })).into_response()
    }
}

#[derive(Debug)]
pub enum SdxlSetupError {
    InvalidSteps(usize),
    MissingFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    NotAFile(std::path::PathBuf),
    EmptyFile(std::path::PathBuf),
}

impl std::fmt::Display for SdxlSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSteps(steps) => write!(
                f,
                "ZIPEEK_IMAGE_STEPS must be between 1 and 100 (got {steps})"
            ),
            Self::MissingFile { path, source } => write!(
                f,
                "required SDXL model file {} is unavailable: {source}",
                path.display()
            ),
            Self::NotAFile(path) => write!(f, "SDXL model path is not a file: {}", path.display()),
            Self::EmptyFile(path) => write!(f, "SDXL model file is empty: {}", path.display()),
        }
    }
}

impl std::error::Error for SdxlSetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingFile { source, .. } => Some(source),
            _ => None,
        }
    }
}
