use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
};

const SERVERS: [&str; 3] = ["zipeek-ai-server", "zipeek-image-server", "zipeek-tts-stt"];

fn host_triple(output: &str) -> io::Result<&str> {
    output
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::trim))
        .filter(|triple| {
            !triple.is_empty()
                && triple
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "rustc -vV did not report a valid host triple",
            )
        })
}

fn stage_paths(root: &Path, release: bool, triple: &str) -> [(PathBuf, PathBuf); 3] {
    let profile = if release { "release" } else { "debug" };
    let exe = std::env::consts::EXE_SUFFIX;
    let source_dir = root.join("target").join(profile);
    let destination_dir = root.join("crates/zipeek-app/binaries");
    SERVERS.map(|server| {
        (
            source_dir.join(format!("{server}{exe}")),
            destination_dir.join(format!("{server}-{triple}{exe}")),
        )
    })
}

fn files_match(source: &Path, destination: &Path) -> io::Result<bool> {
    let source_metadata = fs::metadata(source)?;
    let destination_metadata = match fs::metadata(destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !source_metadata.is_file()
        || !destination_metadata.is_file()
        || source_metadata.len() != destination_metadata.len()
    {
        return Ok(false);
    }
    let mut source_reader = io::BufReader::new(fs::File::open(source)?);
    let mut destination_reader = io::BufReader::new(fs::File::open(destination)?);
    let mut source_chunk = [0_u8; 64 * 1024];
    let mut destination_chunk = [0_u8; 64 * 1024];
    loop {
        let count = source_reader.read(&mut source_chunk)?;
        if count == 0 {
            return Ok(destination_reader.read(&mut destination_chunk[..1])? == 0);
        }
        destination_reader.read_exact(&mut destination_chunk[..count])?;
        if source_chunk[..count] != destination_chunk[..count] {
            return Ok(false);
        }
    }
}

fn stage_file(source: &Path, destination: &Path) -> io::Result<bool> {
    if files_match(source, destination)? {
        return Ok(false);
    }
    fs::copy(source, destination).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "Cannot stage {} to {}: {error}. On Windows, close running Zipeek processes before rebuilding sidecars.",
                source.display(),
                destination.display(),
            ),
        )
    })?;
    Ok(true)
}

pub fn stage(root: &Path, release: bool) -> io::Result<()> {
    let rustc = Command::new("rustc").arg("-vV").output()?;
    if !rustc.status.success() {
        return Err(io::Error::other(format!(
            "rustc -vV failed: {}",
            String::from_utf8_lossy(&rustc.stderr)
        )));
    }
    let output = String::from_utf8(rustc.stdout)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let triple = host_triple(&output)?;
    let paths = stage_paths(root, release, triple);
    let dictionary = crate::database::installed_native_dictionary(root)
        .map_err(|error| io::Error::other(error.to_string()))?;
    for (source, _) in &paths {
        let metadata = fs::metadata(source).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "{}: {error}. Run cargo make desktop-sidecars first.",
                    source.display()
                ),
            )
        })?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Sidecar binary is missing or empty: {}", source.display()),
            ));
        }
    }
    fs::create_dir_all(root.join("crates/zipeek-app/binaries"))?;
    let dictionary_dir = root.join("crates/zipeek-app/binaries/dictionary");
    fs::create_dir_all(&dictionary_dir)?;
    let installed_dictionary = dictionary_dir.join("hanzi.duckdb");
    if !files_match(&dictionary.path, &installed_dictionary)? {
        let staged = dictionary_dir.join("hanzi.duckdb.staging");
        fs::copy(&dictionary.path, &staged)?;
        let staged_hash = crate::database::hash_file(&staged)
            .map_err(|error| io::Error::other(error.to_string()))?;
        if staged_hash.0 != dictionary.size_bytes || staged_hash.1 != dictionary.sha256 {
            return Err(io::Error::other(
                "Staged native dictionary checksum mismatch",
            ));
        }
        fs::rename(&staged, &installed_dictionary)?;
    }
    for (source, destination) in paths {
        if stage_file(&source, &destination)? {
            println!("Staged {} -> {}", source.display(), destination.display());
        }
    }
    if cfg!(windows) {
        let dll_dir = std::env::var_os("DUCKDB_LIB_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.join("target/duckdb-download")
                    .join(triple)
                    .join("v2.0.0")
            });
        let dll = dll_dir.join("duckdb.dll");
        if !dll.is_file() || fs::metadata(&dll)?.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Native DuckDB runtime DLL missing: {}. Run the duckdb-env setup before desktop packaging.",
                    dll.display()
                ),
            ));
        }
        let output = root.join("crates/zipeek-app/binaries/duckdb.dll");
        if stage_file(&dll, &output)? {
            println!("Staged {} -> {}", dll.display(), output.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_sidecar_is_not_rewritten() {
        let directory = std::env::temp_dir().join(format!(
            "zipeek-sidecar-copy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        let source = directory.join("source.exe");
        let destination = directory.join("destination.exe");
        fs::write(&source, b"same executable").unwrap();
        assert!(stage_file(&source, &destination).unwrap());
        assert!(!stage_file(&source, &destination).unwrap());
        fs::write(&source, b"updated executable").unwrap();
        assert!(stage_file(&source, &destination).unwrap());
        assert_eq!(fs::read(destination).unwrap(), b"updated executable");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reads_host_triple() {
        assert_eq!(
            host_triple("rustc 1.90.0\nhost: x86_64-pc-windows-msvc\n").unwrap(),
            "x86_64-pc-windows-msvc"
        );
        assert!(host_triple("rustc 1.90.0\n").is_err());
        assert!(host_triple("host: wrong/path\n").is_err());
    }

    #[test]
    fn stages_expected_names_for_release() {
        let paths = stage_paths(Path::new("workspace"), true, "x86_64-pc-windows-msvc");
        assert!(paths[0].0.to_string_lossy().contains("target"));
        assert!(
            paths[0]
                .1
                .to_string_lossy()
                .contains("zipeek-ai-server-x86_64-pc-windows-msvc")
        );
        assert!(
            paths[2]
                .1
                .to_string_lossy()
                .contains("zipeek-tts-stt-x86_64-pc-windows-msvc")
        );
        assert!(
            paths[1]
                .1
                .to_string_lossy()
                .contains("zipeek-image-server-x86_64-pc-windows-msvc")
        );
    }
}
