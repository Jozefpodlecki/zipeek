use super::{Result, paths::collect};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

pub async fn pack(root: &Path, archive: &Path) -> Result<()> {
    if let Some(parent) = archive
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        tokio::fs::create_dir_all(parent).await?;
    }
    let root = tokio::fs::canonicalize(root).await?;
    let archive = absolute_destination(archive).await?;
    let mut files = collect(&root, &root).await?;
    files.retain(|path| path != &archive);
    let count = files.len();
    let destination = archive.clone();
    tokio::task::spawn_blocking(move || write_zip(&root, &archive, &files)).await??;
    println!("Packed {count} source files: {}", destination.display());
    Ok(())
}

async fn absolute_destination(archive: &Path) -> Result<PathBuf> {
    let parent = archive
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = tokio::fs::canonicalize(parent).await?;
    let name = archive.file_name().ok_or("Archive path must name a file")?;
    Ok(parent.join(name))
}

fn write_zip(root: &Path, archive: &Path, files: &[PathBuf]) -> Result<()> {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temp = archive.with_extension(format!("zip.part-{}-{suffix}", std::process::id()));
    let result = (|| {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        let output = BufWriter::with_capacity(256 * 1024, file);
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
                .to_str()
                .ok_or("Source paths must be valid UTF-8 to be packed")?
                .replace('\\', "/");
            zip.start_file(name, options)?;
            io::copy(
                &mut BufReader::with_capacity(256 * 1024, File::open(file)?),
                &mut zip,
            )?;
        }
        let mut output = zip.finish()?;
        output.flush()?;
        output.get_ref().sync_all()?;
        drop(output);
        replace_archive(&temp, archive, suffix)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn replace_archive(temp: &Path, archive: &Path, suffix: u128) -> Result<()> {
    if !archive.exists() {
        fs::rename(temp, archive)?;
        return Ok(());
    }
    let previous = archive.with_extension(format!("zip.previous-{}-{suffix}", std::process::id()));
    fs::rename(archive, &previous)?;
    match fs::rename(temp, archive) {
        Ok(()) => {
            fs::remove_file(previous)?;
            Ok(())
        }
        Err(error) => {
            if let Err(recovery) = fs::rename(&previous, archive) {
                return Err(format!(
                    "Replacing the archive failed: {error}; restoring the previous archive failed: {recovery}; original retained at {}",
                    previous.display()
                )
                .into());
            }
            Err(error.into())
        }
    }
}

fn compression_method(file: &Path) -> CompressionMethod {
    let extension = file
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "avif" | "gif" | "gz" | "zst" | "br" | "zip" | "pdf"
        | "mp3" | "mp4" | "ogg" | "opus" | "woff2" | "wasm" => CompressionMethod::Stored,
        _ => CompressionMethod::Deflated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_root() -> Result<PathBuf> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-pack-test-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir_all(root.join("nested"))?;
        fs::write(root.join("Cargo.toml"), "[workspace]\n")?;
        fs::write(root.join("nested/src.rs"), "fn main() {}\n")?;
        Ok(root)
    }

    #[test]
    fn already_compressed_assets_are_stored_without_recompression() {
        for name in [
            "image.png",
            "image.webp",
            "IMAGE.PNG",
            "archive.zip",
            "dictionary.gz",
            "font.woff2",
            "audio.ogg",
            "picture.avif",
            "code.wasm",
        ] {
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

    #[tokio::test]
    async fn nested_output_archive_is_excluded_from_input() -> Result<()> {
        let root = fixture_root()?;
        let archive = root.join("nested/source.zip");
        fs::write(&archive, "previous package")?;
        pack(&root, &archive).await?;
        let mut zip = zip::ZipArchive::new(File::open(&archive)?)?;
        assert_eq!(zip.len(), 2);
        assert!(zip.by_name("Cargo.toml").is_ok());
        assert!(zip.by_name("nested/src.rs").is_ok());
        assert!(zip.by_name("nested/source.zip").is_err());
        assert!(!root.join("nested/source.zip.part").exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn failed_archive_write_keeps_existing_output_and_cleans_staging() -> Result<()> {
        let root = fixture_root()?;
        let archive = root.join("source.zip");
        fs::write(&archive, "original")?;
        let nonexistent = root.join("missing.rs");
        assert!(write_zip(&root, &archive, &[nonexistent]).is_err());
        assert_eq!(fs::read(&archive)?, b"original");
        let temporary_files = fs::read_dir(&root)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains("zip.part-"))
            .count();
        assert_eq!(temporary_files, 0);
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
