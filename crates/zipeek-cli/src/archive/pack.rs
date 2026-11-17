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
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for file in files {
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
