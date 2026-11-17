use crate::archive::Result;
use std::{fs, path::Path};

const PRESETS: [&str; 6] = ["lavender", "jade", "sunset", "midnight", "ink", "maple"];
const MAX_WEBP_BYTES: usize = 800_000;
const IMAGE_PREFIX: &str = "data:image/webp;base64,";
const IMAGE_MARKER: &str = "id=\"app-background\" class=\"background\" src=\"";

fn validated_backgrounds(web: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let directory = web.join("public/backgrounds");
    PRESETS
        .iter()
        .map(|preset| {
            let file = directory.join(format!("{preset}.webp"));
            let data = fs::read(&file).map_err(|error| {
                format!("Missing bundled background {}: {error}. Restore public/backgrounds from the source release", file.display())
            })?;
            if data.len() < 20
                || data.len() > MAX_WEBP_BYTES
                || &data[..4] != b"RIFF"
                || &data[8..12] != b"WEBP"
                || u32::from_le_bytes(data[4..8].try_into()?) as usize + 8 != data.len()
            {
                return Err(format!("Invalid optimized WebP background: {}", file.display()).into());
            }
            Ok(((*preset).to_owned(), data))
        })
        .collect()
}

fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0] as usize;
        let second = chunk.get(1).copied().unwrap_or(0) as usize;
        let third = chunk.get(2).copied().unwrap_or(0) as usize;
        encoded.push(ALPHABET[first >> 2] as char);
        encoded.push(ALPHABET[((first & 3) << 4) | (second >> 4)] as char);
        encoded.push(if chunk.len() > 1 {
            ALPHABET[((second & 15) << 2) | (third >> 6)] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            ALPHABET[third & 63] as char
        } else {
            '='
        });
    }
    encoded
}

fn embedded_image_range(html: &str) -> Result<std::ops::Range<usize>> {
    let marker = html
        .find(IMAGE_MARKER)
        .ok_or("Missing app-background image in index.html")?;
    let start = marker + IMAGE_MARKER.len();
    let end = start
        + html[start..]
            .find('"')
            .ok_or("Unterminated app-background src")?;
    if !html[start..end].starts_with(IMAGE_PREFIX) {
        return Err("app-background src must be an embedded WebP data URL".into());
    }
    Ok(start..end)
}

pub fn prepare(root: &Path) -> Result<()> {
    let web = root.join("crates/zipeek-web");
    let backgrounds = validated_backgrounds(&web)?;
    let lavender = &backgrounds[0].1;
    let html_path = web.join("index.html");
    let mut html = fs::read_to_string(&html_path)?;
    let range = embedded_image_range(&html)?;
    let data_url = format!("{IMAGE_PREFIX}{}", base64_encode(lavender));
    if html[range.clone()] == data_url {
        println!("Background assets already current (6 presets)");
        return Ok(());
    }
    html.replace_range(range, &data_url);
    fs::write(&html_path, html)?;
    println!("Embedded Lavender first-paint background; verified 6 optimized WebPs");
    Ok(())
}

pub fn verify(root: &Path) -> Result<()> {
    let web = root.join("crates/zipeek-web");
    let backgrounds = validated_backgrounds(&web)?;
    let html = fs::read_to_string(web.join("index.html"))?;
    let range = embedded_image_range(&html)?;
    let expected = format!("{IMAGE_PREFIX}{}", base64_encode(&backgrounds[0].1));
    if html[range] != expected {
        return Err("Embedded Lavender image is stale; run cargo make prepare-backgrounds".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encodes_padding_and_three_byte_chunks() {
        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(input.as_bytes()), expected);
        }
    }

    #[test]
    fn only_the_background_image_src_is_replaced() -> Result<()> {
        let html = "<img id=\"app-background\" class=\"background\" src=\"data:image/webp;base64,old\"><img src=\"elsewhere\">";
        let mut updated = html.to_owned();
        updated.replace_range(embedded_image_range(html)?, "data:image/webp;base64,new");
        assert!(updated.contains("base64,new\""));
        assert!(updated.contains("src=\"elsewhere\""));
        assert!(embedded_image_range("<img src=\"elsewhere\">").is_err());
        Ok(())
    }

    #[test]
    fn preparation_is_idempotent_and_rejects_corrupt_files() -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zipeek-backgrounds-{}-{unique}",
            std::process::id()
        ));
        let web = root.join("crates/zipeek-web");
        let images = web.join("public/backgrounds");
        fs::create_dir_all(&images)?;
        let result = (|| -> Result<()> {
            let mut sample = b"RIFF".to_vec();
            sample.extend_from_slice(&12_u32.to_le_bytes());
            sample.extend_from_slice(b"WEBPVP8 ");
            sample.extend_from_slice(&[0_u8; 4]);
            for preset in PRESETS {
                fs::write(images.join(format!("{preset}.webp")), &sample)?;
            }
            let html = "<img id=\"app-background\" class=\"background\" src=\"data:image/webp;base64,old\">";
            fs::write(web.join("index.html"), html)?;
            prepare(&root)?;
            let first = fs::read(web.join("index.html"))?;
            prepare(&root)?;
            assert_eq!(fs::read(web.join("index.html"))?, first);
            verify(&root)?;
            fs::write(images.join("jade.webp"), b"broken")?;
            assert!(prepare(&root).is_err());
            Ok(())
        })();
        fs::remove_dir_all(root)?;
        result
    }
}
