use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub docs_count: usize,
}

pub fn verify(root: &Path, require_worker: bool) -> io::Result<Summary> {
    let source = root.join("docs");
    let published = root.join("crates/zipeek-web/static/docs");
    let index_path = published.join("index.json");
    let index: serde_json::Value = serde_json::from_slice(&fs::read(&index_path)?)
        .map_err(|e| invalid(format!("{}: {e}", index_path.display())))?;
    let entries = index
        .as_array()
        .ok_or_else(|| invalid("Docs index must be an array"))?;
    let mut indexed = BTreeSet::new();
    for entry in entries {
        let path = entry
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| invalid("Docs index entry missing path"))?;
        let relative = safe_relative(path)?;
        if !indexed.insert(path.to_owned()) {
            return Err(invalid(format!("duplicate Docs path: {path}")));
        }
        let source_file = source.join(&relative);
        require_regular_file(&source_file)?;
        let markdown = source_file
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("md"));
        if entry.get("markdown").and_then(serde_json::Value::as_bool) != Some(markdown) {
            return Err(invalid(format!("Docs index markdown flag differs: {path}")));
        }
        if markdown {
            let current_text = fs::read_to_string(&source_file)?;
            if entry.get("search_text").and_then(serde_json::Value::as_str)
                != Some(current_text.as_str())
            {
                return Err(invalid(format!(
                    "stale Docs search index: {path}; rebuild Docs"
                )));
            }
        }
    }
    let mut source_paths = BTreeSet::new();
    collect_files(&source, &source, &mut source_paths)?;
    if source_paths != indexed {
        let missing: Vec<_> = source_paths.difference(&indexed).take(3).collect();
        let extra: Vec<_> = indexed.difference(&source_paths).take(3).collect();
        return Err(invalid(format!(
            "Docs index differs from source: missing {missing:?}, extra {extra:?}; rebuild Docs"
        )));
    }
    let mut published_paths = BTreeSet::new();
    collect_files(&published, &published, &mut published_paths)?;
    published_paths.remove("index.json");
    if !published_paths.is_empty() {
        return Err(invalid(
            "Docs source files must be copied by Trunk, not duplicated under static/docs",
        ));
    }

    for name in [
        "grammar-autumn-bridge.png",
        "vocabulary-peony-garden.png",
        "pinyin-landscape.png",
    ] {
        let illustration = root.join("crates/zipeek-web/static/courses").join(name);
        require_regular_file(&illustration)?;
        let png = fs::read(&illustration)?;
        if png.len() < 24
            || &png[..8] != b"\x89PNG\r\n\x1a\n"
            || &png[12..16] != b"IHDR"
            || u32::from_be_bytes(png[16..20].try_into().unwrap()) == 0
            || u32::from_be_bytes(png[20..24].try_into().unwrap()) == 0
        {
            return Err(invalid(format!(
                "invalid Courses artwork: {}",
                illustration.display()
            )));
        }
    }

    for name in [
        "dragon.webp",
        "qilin.webp",
        "phoenix.webp",
        "white-tiger.webp",
        "crane.webp",
        "deer.webp",
    ] {
        let artwork = root.join("crates/zipeek-web/static/practice").join(name);
        require_regular_file(&artwork)?;
        let webp = fs::read(&artwork)?;
        let (width, height) = practice_webp_dimensions(&webp).ok_or_else(|| {
            invalid(format!(
                "invalid Practice creature artwork: {}",
                artwork.display()
            ))
        })?;
        let horizontal = matches!(name, "white-tiger.webp" | "deer.webp");
        if (horizontal && width < height.saturating_mul(2))
            || (!horizontal && height < width.saturating_mul(2))
        {
            return Err(invalid(format!(
                "Practice creature artwork has incorrect orientation: {}",
                artwork.display()
            )));
        }
    }

    let worker_dir = root.join("crates/zipeek-web/static/pedagogy-worker");
    let bootstrap = worker_dir.join("worker.mjs");
    let generated = [
        "pedagogy_wasm.js",
        "pedagogy_wasm_bg.wasm",
        "pedagogy_wasm.d.ts",
        "pedagogy_wasm_bg.wasm.d.ts",
    ];
    let present: Vec<bool> = generated
        .iter()
        .map(|p| worker_dir.join(p).exists())
        .collect();
    let bootstrap_present = bootstrap.exists();
    if bootstrap_present || require_worker || present.iter().any(|value| *value) {
        require_regular_file(&bootstrap)?;
        let text = fs::read_to_string(&bootstrap)?;
        if !text.contains("./pedagogy_wasm.js") || !text.contains("start_worker") {
            return Err(invalid(
                "worker.mjs is incompatible with Rust start_worker; regenerate matching worker assets",
            ));
        }
    }
    if present.iter().any(|v| *v) && !present.iter().all(|v| *v) {
        return Err(invalid(
            "partial pedagogy WASM bindings: run `zipeek-cli build pedagogy-worker`",
        ));
    }
    if require_worker && !present.iter().all(|v| *v) {
        return Err(invalid(
            "missing pedagogy WASM bindings: run `zipeek-cli build pedagogy-worker`",
        ));
    }
    if present.iter().all(|v| *v) {
        for name in generated {
            require_regular_file(&worker_dir.join(name))?;
        }
        let wasm = fs::read(worker_dir.join("pedagogy_wasm_bg.wasm"))?;
        if !wasm.starts_with(b"\0asm") {
            return Err(invalid("pedagogy WASM binding has invalid magic bytes"));
        }
        let js = fs::read_to_string(worker_dir.join("pedagogy_wasm.js"))?;
        if !js.contains("start_worker") {
            return Err(invalid("pedagogy bindings lack start_worker export"));
        }
    }
    Ok(Summary {
        docs_count: source_paths.len(),
    })
}

fn practice_webp_dimensions(webp: &[u8]) -> Option<(u32, u32)> {
    if webp.len() < 30
        || &webp[..4] != b"RIFF"
        || &webp[8..16] != b"WEBPVP8X"
        || webp[20] & 0x10 == 0
    {
        return None;
    }
    let width = u32::from(webp[24]) | (u32::from(webp[25]) << 8) | (u32::from(webp[26]) << 16);
    let height = u32::from(webp[27]) | (u32::from(webp[28]) << 8) | (u32::from(webp[29]) << 16);
    Some((width + 1, height + 1))
}

fn safe_relative(input: &str) -> io::Result<PathBuf> {
    if input.is_empty() || input.contains('\\') || input.contains(':') || input.contains('\0') {
        return Err(invalid(format!("unsafe Docs path: {input:?}")));
    }
    let path = Path::new(input);
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(invalid(format!("unsafe Docs path: {input:?}")));
    }
    Ok(path.to_owned())
}

fn collect_files(root: &Path, dir: &Path, files: &mut BTreeSet<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            return Err(invalid(format!(
                "symlink in published Docs: {}",
                path.display()
            )));
        }
        if meta.is_dir() {
            collect_files(root, &path, files)?;
        } else if meta.is_file() {
            let rel = path.strip_prefix(root).map_err(io::Error::other)?;
            let name = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            files.insert(name);
        } else {
            return Err(invalid(format!("non-file in Docs: {}", path.display())));
        }
    }
    Ok(())
}
fn require_regular_file(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|e| invalid(format!("missing asset {}: {e}", path.display())))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(invalid(format!(
            "empty or non-file asset: {}",
            path.display()
        )));
    }
    Ok(())
}
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mock_creature_webp(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&22_u32.to_le_bytes());
        bytes.extend_from_slice(b"WEBPVP8X");
        bytes.extend_from_slice(&10_u32.to_le_bytes());
        bytes.push(0x10);
        bytes.extend_from_slice(&[0, 0, 0]);
        for dimension in [width - 1, height - 1] {
            bytes.extend_from_slice(&dimension.to_le_bytes()[..3]);
        }
        bytes
    }

    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "zipeek-assets-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("docs/nested")).unwrap();
        fs::write(root.join("docs/nested/a.md"), "# Example\nbody").unwrap();
        fs::create_dir_all(root.join("crates/zipeek-web/static/courses")).unwrap();
        fs::write(
            root.join("crates/zipeek-web/static/courses/grammar-autumn-bridge.png"),
            [
                137u8, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0,
                0, 1,
            ],
        )
        .unwrap();
        fs::write(
            root.join("crates/zipeek-web/static/courses/pinyin-landscape.png"),
            [
                137u8, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0,
                0, 1,
            ],
        )
        .unwrap();
        fs::write(
            root.join("crates/zipeek-web/static/courses/vocabulary-peony-garden.png"),
            [
                137u8, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0,
                0, 1,
            ],
        )
        .unwrap();
        let practice = root.join("crates/zipeek-web/static/practice");
        fs::create_dir_all(&practice).unwrap();
        for name in [
            "dragon.webp",
            "qilin.webp",
            "phoenix.webp",
            "white-tiger.webp",
            "crane.webp",
            "deer.webp",
        ] {
            let (width, height) = if matches!(name, "white-tiger.webp" | "deer.webp") {
                (1800, 600)
            } else {
                (720, 2160)
            };
            fs::write(practice.join(name), mock_creature_webp(width, height)).unwrap();
        }
        fs::create_dir_all(root.join("crates/zipeek-web/static/pedagogy-worker")).unwrap();
        fs::write(
            root.join("crates/zipeek-web/static/pedagogy-worker/worker.mjs"),
            "import('./pedagogy_wasm.js').then(({start_worker}) => start_worker());",
        )
        .unwrap();
        crate::docs::build(&root).unwrap();
        root
    }
    #[test]
    fn source_preflight_allows_worker_not_yet_generated() {
        let root = fixture();
        fs::remove_dir_all(root.join("crates/zipeek-web/static/pedagogy-worker")).unwrap();
        assert_eq!(verify(&root, false).unwrap().docs_count, 1);
        assert!(verify(&root, true).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_bootstrap_is_rejected_if_bindings_exist() {
        let root = fixture();
        let worker = root.join("crates/zipeek-web/static/pedagogy-worker");
        fs::remove_file(worker.join("worker.mjs")).unwrap();
        fs::write(worker.join("pedagogy_wasm.js"), "partial").unwrap();
        let error = verify(&root, false).unwrap_err().to_string();
        assert!(error.contains("worker.mjs"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_practice_creature_is_rejected() {
        let root = fixture();
        fs::remove_file(root.join("crates/zipeek-web/static/practice/qilin.webp")).unwrap();
        let error = verify(&root, false).unwrap_err().to_string();
        assert!(error.contains("qilin.webp"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn incorrectly_oriented_practice_creature_is_rejected() {
        let root = fixture();
        fs::write(
            root.join("crates/zipeek-web/static/practice/deer.webp"),
            mock_creature_webp(720, 2160),
        )
        .unwrap();
        let error = verify(&root, false).unwrap_err().to_string();
        assert!(error.contains("incorrect orientation"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_vocabulary_artwork_is_rejected() {
        let root = fixture();
        fs::remove_file(
            root.join("crates/zipeek-web/static/courses/vocabulary-peony-garden.png"),
        )
        .unwrap();
        let error = verify(&root, false).unwrap_err().to_string();
        assert!(error.contains("vocabulary-peony-garden.png"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_allows_missing_worker_bindings_but_deploy_rejects() {
        let root = fixture();
        assert_eq!(verify(&root, false).unwrap().docs_count, 1);
        assert!(
            verify(&root, true)
                .unwrap_err()
                .to_string()
                .contains("missing pedagogy")
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn stale_docs_and_traversal_are_rejected() {
        let root = fixture();
        fs::write(root.join("docs/nested/a.md"), "changed").unwrap();
        assert!(
            verify(&root, false)
                .unwrap_err()
                .to_string()
                .contains("stale")
        );
        crate::docs::build(&root).unwrap();
        let index = root.join("crates/zipeek-web/static/docs/index.json");
        let mut entries: serde_json::Value =
            serde_json::from_slice(&fs::read(&index).unwrap()).unwrap();
        entries[0]["path"] = "../secret.md".into();
        fs::write(index, serde_json::to_vec(&entries).unwrap()).unwrap();
        assert!(
            verify(&root, false)
                .unwrap_err()
                .to_string()
                .contains("unsafe")
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn generated_bindings_must_be_complete_and_plausible() {
        let root = fixture();
        let dir = root.join("crates/zipeek-web/static/pedagogy-worker");
        fs::write(dir.join("pedagogy_wasm.js"), "partial").unwrap();
        assert!(
            verify(&root, false)
                .unwrap_err()
                .to_string()
                .contains("partial")
        );
        fs::write(dir.join("pedagogy_wasm_bg.wasm"), b"\0asm-valid").unwrap();
        fs::write(
            dir.join("pedagogy_wasm.d.ts"),
            "export declare function x(): void;",
        )
        .unwrap();
        fs::write(
            dir.join("pedagogy_wasm_bg.wasm.d.ts"),
            "export const x: number;",
        )
        .unwrap();
        assert!(
            verify(&root, true)
                .unwrap_err()
                .to_string()
                .contains("start_worker")
        );
        fs::write(
            dir.join("pedagogy_wasm.js"),
            "export function start_worker(){}",
        )
        .unwrap();
        assert!(verify(&root, true).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
}
