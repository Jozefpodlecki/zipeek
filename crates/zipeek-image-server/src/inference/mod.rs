mod placeholder;
mod png;
mod sdxl;

pub use placeholder::PlaceholderBackend;
pub use sdxl::SdxlBackend;

use std::{future::Future, pin::Pin};

pub struct GeneratedImage {
    pub bytes: Vec<u8>,
    pub media_type: &'static str,
}

pub trait ImageBackend: Send + Sync + 'static {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
        width: u32,
        height: u32,
    ) -> Pin<Box<dyn Future<Output = Result<GeneratedImage, String>> + Send + 'a>>;
}
