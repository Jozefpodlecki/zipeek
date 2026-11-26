use super::{Result, paths::collect};
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

pub async fn pack(root: &Path, archive: &Path) -> Result<()> {
    if let Some(parent) = archive.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let files = collect(root, root).await?;
    let root = root.to_owned();
    let archive = archive.to_owned();
    let count = files.len();
    let destination = archive.clone();
    tokio::task::spawn_blocking(move || write_zip(&root, &archive, &files)).await??;
    println!("Packed {count} source files: {}", destination.display());
    Ok(())
}

fn write_zip(root: &Path, archive: &Path, files: &[PathBuf]) -> Result<()> {
    let temp = archive.with_extension("zip.part");
    let output = File::create(&temp)?;
    let mut zip = ZipWriter::new(output);
    for file in files {
        let method = compression_method(file);
        let options = SimpleFileOptions::default()
            .compression_method(method)
            .compression_level(if method == CompressionMethod::Deflated {
                Some(3)
            } else {
                None
            });
        let name = file
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        zip.start_file(name, options)?;
        io::copy(&mut File::open(file)?, &mut zip)?;
    }
    zip.finish()?;
    if archive.exists() {
        std::fs::remove_file(archive)?;
    }
    std::fs::rename(&temp, archive)?;
    Ok(())
}

fn compression_method(file: &Path) -> CompressionMethod {
    match file.extension().and_then(|extension| extension.to_str()) {
        Some(
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "gz" | "zst" | "zip" | "pdf" | "mp3" | "mp4"
            | "wasm",
        ) => CompressionMethod::Stored,
        _ => CompressionMethod::Deflated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_compressed_assets_are_stored_without_recompression() {
        for name in ["image.png", "image.webp", "archive.zip", "dictionary.gz"] {
            assert_eq!(
                compression_method(Path::new(name)),
                CompressionMethod::Stored
            );
        }
        for name in ["source.rs", "characters.json", "README.md"] {
            assert_eq!(
                compression_method(Path::new(name)),
                CompressionMethod::Deflated
            );
        }
    }
}
