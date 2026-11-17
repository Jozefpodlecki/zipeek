use std::io::Write;

pub fn encode_rgb8(width: u32, height: u32, rgb: &[u8]) -> Result<Vec<u8>, String> {
    let row = usize::try_from(width)
        .map_err(|e| e.to_string())?
        .checked_mul(3)
        .ok_or("image width overflow")?;
    let expected = row
        .checked_mul(height as usize)
        .ok_or("image size overflow")?;
    if width == 0 || height == 0 || rgb.len() != expected {
        return Err("invalid RGB dimensions or pixel byte count".into());
    }

    let mut scanlines = Vec::with_capacity(expected + height as usize);
    for row_bytes in rgb.chunks_exact(row) {
        scanlines.push(0);
        scanlines.extend_from_slice(row_bytes);
    }

    let mut zlib = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    zlib.write_all(&scanlines).map_err(|e| e.to_string())?;
    let compressed = zlib.finish().map_err(|e| e.to_string())?;

    let mut png = Vec::with_capacity(compressed.len() + 64);
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr)?;
    chunk(&mut png, b"IDAT", &compressed)?;
    chunk(&mut png, b"IEND", &[])?;
    Ok(png)
}

fn chunk(target: &mut Vec<u8>, kind: &[u8; 4], payload: &[u8]) -> Result<(), String> {
    let len = u32::try_from(payload.len()).map_err(|_| "PNG chunk too large")?;
    target.extend_from_slice(&len.to_be_bytes());
    target.extend_from_slice(kind);
    target.extend_from_slice(payload);
    let mut crc = crc32fast::Hasher::new();
    crc.update(kind);
    crc.update(payload);
    target.extend_from_slice(&crc.finalize().to_be_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn png_roundtrip_rgb() {
        let bytes = encode_rgb8(2, 1, &[255, 0, 0, 0, 255, 0]).unwrap();
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(&bytes[12..16], b"IHDR");
        assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 2);
        assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 1);
        assert_eq!(&bytes[bytes.len() - 8..bytes.len() - 4], b"IEND");
        let idat_pos = 33;
        let idat_len =
            u32::from_be_bytes(bytes[idat_pos..idat_pos + 4].try_into().unwrap()) as usize;
        assert_eq!(&bytes[idat_pos + 4..idat_pos + 8], b"IDAT");
        let mut decoded = Vec::new();
        flate2::read::ZlibDecoder::new(&bytes[idat_pos + 8..idat_pos + 8 + idat_len])
            .read_to_end(&mut decoded)
            .unwrap();
        assert_eq!(decoded, vec![0, 255, 0, 0, 0, 255, 0]);
    }

    #[test]
    fn incorrect_pixel_count_rejected() {
        assert!(encode_rgb8(512, 512, &[0, 1, 2]).is_err());
    }
}
