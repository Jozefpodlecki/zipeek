use super::{GeneratedImage, ImageBackend};
use std::{future::Future, pin::Pin};

pub struct PlaceholderBackend;

fn escape_svg(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '&' => "&amp;".to_owned(),
            '<' => "&lt;".to_owned(),
            '>' => "&gt;".to_owned(),
            '"' => "&quot;".to_owned(),
            '\'' => "&apos;".to_owned(),
            c if c.is_control() => " ".to_owned(),
            c => c.to_string(),
        })
        .collect()
}

impl ImageBackend for PlaceholderBackend {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
        width: u32,
        height: u32,
    ) -> Pin<Box<dyn Future<Output = Result<GeneratedImage, String>> + Send + 'a>> {
        Box::pin(async move {
            let title = escape_svg(prompt);
            let svg = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}"><rect width="100%" height="100%" fill="#31275e"/><rect x="24" y="24" width="{inner_width}" height="{inner_height}" rx="18" fill="#43358b" stroke="#9682d9"/><text x="50%" y="42%" text-anchor="middle" fill="#fff" font-family="sans-serif" font-size="24">Image preview only</text><text x="50%" y="55%" text-anchor="middle" fill="#e8ddff" font-family="sans-serif" font-size="16">{title}</text></svg>"##,
                inner_width = width - 48,
                inner_height = height - 48
            );
            Ok(GeneratedImage {
                bytes: svg.into_bytes(),
                media_type: "image/svg+xml",
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn placeholder_escapes_untrusted_prompt() {
        let image = PlaceholderBackend
            .generate("<script>&", 512, 512)
            .await
            .unwrap();
        let svg = String::from_utf8(image.bytes).unwrap();
        assert!(svg.contains("&lt;script&gt;&amp;"));
        assert!(!svg.contains("<script>"));
    }
}
