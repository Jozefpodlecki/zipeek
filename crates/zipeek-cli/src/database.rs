use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

use crate::archive::Result;

const MANIFEST: &str = "hanzi.manifest.json";
const REQUIRED: &[&str] = &[
    "hanzi.diagnostics.json",
    "INPUTS.toml",
    "ATTRIBUTION.md",
    "notices/HSK-LICENSE.txt",
    "notices/HSK-SOURCE-README.txt",
    "notices/CC-BY-SA-4.0.txt",
];
const BROWSE: &[&str] = &[
    "words",
    "character_readings",
    "word_hsk_membership",
    "word_pos_membership",
    "word_senses",
    "sense_categories",
];
const ENRICHMENT: &[&str] = &[
    "word_senses",
    "sense_pos",
    "sense_categories",
    "classification_runs",
    "parts_of_speech",
    "categories",
];

fn checked_file(root: &Path, name: &str) -> Result<PathBuf> {
    if name.is_empty()
        || name.contains('\\')
        || name.contains(':')
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("Invalid database artifact path: {name}").into());
    }
    let relative = Path::new(name);
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("Unsafe database artifact path: {name}").into());
    }
    let mut cursor = root.to_path_buf();
    for part in relative.components() {
        cursor.push(part.as_os_str());
        if fs::symlink_metadata(&cursor)?.file_type().is_symlink() {
            return Err(format!("Artifact symlink refused: {}", cursor.display()).into());
        }
    }
    if !cursor.is_file() {
        return Err(format!("Artifact is not a file: {}", cursor.display()).into());
    }
    Ok(cursor)
}

pub(super) fn hash_file(path: &Path) -> Result<(u64, String)> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size += read as u64;
    }
    let digest: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok((size, digest))
}

fn dictionary_name(path: &str) -> bool {
    if path == "hanzi.duckdb" {
        return true;
    }
    let Some(stamp) = path
        .strip_prefix("hanzi-")
        .and_then(|name| name.strip_suffix(".duckdb"))
    else {
        return false;
    };
    stamp.len() == 15
        && stamp.as_bytes()[8] == b'-'
        && stamp
            .bytes()
            .enumerate()
            .all(|(i, byte)| i == 8 || byte.is_ascii_digit())
}

pub struct NativeDictionaryArtifact {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub sha256: String,
}

pub fn installed_native_dictionary(root: &Path) -> Result<NativeDictionaryArtifact> {
    let bundle = root.join("crates/zipeek-web/static");
    if fs::symlink_metadata(&bundle)?.file_type().is_symlink() || !bundle.is_dir() {
        return Err("Invalid installed dictionary directory".into());
    }
    let report: Value = serde_json::from_reader(fs::File::open(checked_file(&bundle, MANIFEST)?)?)?;
    if report["engine"] != "v2.0.0" || report["storage_compatibility"] != "v2.0.0" {
        return Err("Native dictionary engine/storage compatibility mismatch".into());
    }
    let files = report["files"]
        .as_array()
        .ok_or("Missing dictionary files")?;
    let databases: Vec<_> = files
        .iter()
        .filter(|entry| {
            entry["path"]
                .as_str()
                .is_some_and(|name| name.ends_with(".duckdb"))
        })
        .collect();
    if databases.len() != 1 {
        return Err("Dictionary manifest must include exactly one native database".into());
    }
    let entry = databases[0];
    let name = entry["path"]
        .as_str()
        .ok_or("Missing native dictionary path")?;
    if !dictionary_name(name) {
        return Err("Invalid native dictionary filename".into());
    }
    let path = checked_file(&bundle, name)?;
    let (size, digest) = hash_file(&path)?;
    if size == 0
        || entry["size_bytes"].as_u64() != Some(size)
        || entry["sha256"].as_str() != Some(&digest)
    {
        return Err("Native dictionary size/checksum mismatch".into());
    }
    Ok(NativeDictionaryArtifact {
        path,
        size_bytes: size,
        sha256: digest,
    })
}

fn validate(
    root: &Path,
    bundle: &Path,
    enriched: bool,
) -> Result<(Value, String, Option<String>, Vec<String>)> {
    if fs::symlink_metadata(bundle)?.file_type().is_symlink() || !bundle.is_dir() {
        return Err("Invalid dictionary bundle directory".into());
    }
    let report: Value = serde_json::from_reader(fs::File::open(checked_file(bundle, MANIFEST)?)?)?;
    if report["engine"] != "v2.0.0"
        || report["storage_compatibility"] != "v2.0.0"
        || report["wasm_package"] != "@duckdb/duckdb-wasm@1.33.1-dev114.0"
    {
        return Err("Dictionary engine/storage/WASM compatibility mismatch".into());
    }
    if report["rows"].as_u64().is_none() {
        return Err("Invalid dictionary row count".into());
    }
    let files = report["files"]
        .as_array()
        .ok_or("Missing dictionary files")?;
    let mut seen = std::collections::HashSet::new();
    let mut databases = Vec::new();
    let mut compressed = None;
    for entry in files {
        let name = entry["path"].as_str().ok_or("Missing artifact path")?;
        if name == MANIFEST || !seen.insert(name.to_owned()) {
            return Err(format!("Duplicate dictionary artifact: {name}").into());
        }
        let path = checked_file(bundle, name)?;
        let (size, digest) = hash_file(&path)?;
        if entry["size_bytes"].as_u64() != Some(size) || entry["sha256"].as_str() != Some(&digest) {
            return Err(format!("Dictionary checksum mismatch: {name}").into());
        }
        if dictionary_name(name) {
            databases.push(name.to_owned());
        }
    }
    if databases.len() != 1 {
        return Err("Dictionary manifest must include exactly one database".into());
    }
    for required in REQUIRED {
        if !seen.contains(*required) {
            return Err(format!("Missing required artifact: {required}").into());
        }
    }
    let (_, inputs_digest) = hash_file(&checked_file(
        root,
        "crates/zipeek-db-gen/data/MANIFEST.toml",
    )?)?;
    if files
        .iter()
        .find(|file| file["path"] == "INPUTS.toml")
        .and_then(|file| file["sha256"].as_str())
        != Some(inputs_digest.as_str())
    {
        return Err("Bundle does not match pinned generator inputs".into());
    }
    let table_names = report["tables"]
        .as_array()
        .ok_or("Missing schema table list")?;
    for table in BROWSE.iter().chain(if enriched {
        ENRICHMENT.iter()
    } else {
        [].iter()
    }) {
        if !table_names.iter().any(|name| name.as_str() == Some(*table)) {
            return Err(format!("Dictionary is missing table {table}").into());
        }
    }
    let database = databases.remove(0);
    if seen.contains(&format!("{database}.gz")) {
        compressed = Some(format!("{database}.gz"));
    }
    Ok((report, database, compressed, seen.into_iter().collect()))
}

pub fn browse_check(root: &Path) -> Result<()> {
    let installed = root.join("crates/zipeek-web/static");
    let (report, _, _, _) = validate(root, &installed, false)?;
    let tables = report["tables"].as_array().ok_or("Missing dictionary tables")?;
    for name in ["word_search_documents", "word_search_keys"] {
        if !tables.iter().any(|table| table.as_str() == Some(name)) {
            return Err(format!("Installed dictionary is missing search table {name}").into());
        }
    }
    println!("Installed dictionary bundle and Browse/search schema verified");
    Ok(())
}

pub fn compress_latest(root: &Path) -> Result<()> {
    let workflow = root.join("target/db-workflow").canonicalize()?;
    let pointer = workflow.join("latest.json");
    if fs::symlink_metadata(&pointer)?.file_type().is_symlink() {
        return Err("Dictionary latest pointer must not be a symlink".into());
    }
    let selection: Value = serde_json::from_reader(fs::File::open(pointer)?)?;
    let relative = selection["directory"]
        .as_str()
        .ok_or("Dictionary latest pointer has no directory")?;
    let candidate = root.join(relative).canonicalize()?;
    if !candidate.starts_with(workflow.join("generated")) || !candidate.is_dir() {
        return Err("Dictionary candidate must be inside target/db-workflow/generated".into());
    }
    compress_candidate(root, &candidate)
}

fn compress_candidate(root: &Path, candidate: &Path) -> Result<()> {
    let (mut manifest, database, _, _) = validate(root, candidate, false)?;
    let source = checked_file(candidate, &database)?;
    let compressed_name = format!("{database}.gz");
    let compressed = candidate.join(&compressed_name);
    if compressed.exists() && fs::symlink_metadata(&compressed)?.file_type().is_symlink() {
        return Err("Compressed dictionary output must not be a symlink".into());
    }
    let temporary = candidate.join(format!("{compressed_name}.part"));
    let manifest_path = candidate.join(MANIFEST);
    let temporary_manifest = candidate.join(format!("{MANIFEST}.part"));
    if temporary.exists() || temporary_manifest.exists() {
        return Err(
            "Previous compression temporary file exists; resolve it before retrying".into(),
        );
    }
    let result = (|| -> Result<()> {
        let mut reader = fs::File::open(&source)?;
        let writer = fs::File::create(&temporary)?;
        let mut encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::best());
        std::io::copy(&mut reader, &mut encoder)?;
        encoder.finish()?.sync_all()?;
        let (size, sha256) = hash_file(&temporary)?;
        let files = manifest["files"]
            .as_array_mut()
            .ok_or("Missing artifact entries")?;
        files.retain(|entry| entry["path"].as_str() != Some(compressed_name.as_str()));
        files.push(serde_json::json!({
            "path": compressed_name,
            "size_bytes": size,
            "sha256": sha256
        }));
        fs::write(
            &temporary_manifest,
            format!("{}\n", serde_json::to_string_pretty(&manifest)?),
        )?;
        fs::rename(&temporary, &compressed)?;
        fs::rename(&temporary_manifest, &manifest_path)?;
        validate(root, candidate, false)?;
        println!(
            "Compressed dictionary: {} ({:.2} MiB, {} bytes)",
            compressed.display(),
            size as f64 / (1024.0 * 1024.0),
            size
        );
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
        let _ = fs::remove_file(&temporary_manifest);
    }
    result
}

pub fn validate_candidate(root: &Path, bundle: &Path) -> Result<()> {
    validate(root, bundle, false).map(|_| ())
}

fn update_html(html: &str, dictionary_asset: &str) -> Result<String> {
    if !html.contains("href=\"hanzi.manifest.json\"") {
        return Err("Manifest preload element missing in index.html".into());
    }
    let marker = "href=\"hanzi-";
    let start = html
        .find(marker)
        .ok_or("Dictionary preload element missing in index.html")?;
    let tag_start = html[..start]
        .rfind("<link")
        .ok_or("Dictionary preload element is not a link")?;
    if !html[tag_start..start].contains("rel=\"preload\"")
        || !html[tag_start..start].contains("as=\"fetch\"")
    {
        return Err("Dictionary asset link must be a fetch preload".into());
    }
    let rest = &html[start + marker.len()..];
    let end = start
        + marker.len()
        + rest
            .find('"')
            .ok_or("Unterminated dictionary preload URL")?
        + 1;
    let mut updated = html.to_owned();
    updated.replace_range(start..end, &format!("href=\"{dictionary_asset}\""));
    Ok(updated)
}

fn installed_database_name(name: &str) -> bool {
    dictionary_name(name) || name.strip_suffix(".gz").is_some_and(dictionary_name)
}

fn dictionary_bundle_name(name: &str) -> bool {
    name == MANIFEST || REQUIRED.contains(&name) || installed_database_name(name)
}

fn existing_file(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("Refusing symlinked dictionary target: {}", path.display()).into())
        }
        Ok(metadata) if !metadata.is_file() => {
            Err(format!("Dictionary target is not a file: {}", path.display()).into())
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn install_dictionary_files(
    stage: &Path,
    dest: &Path,
    backup: &Path,
    html_path: &Path,
    next_html: &str,
    names: &[String],
) -> Result<()> {
    if dest.is_symlink() || html_path.is_symlink() {
        return Err("Refusing symlinked installation target".into());
    }
    if dest.exists() && !dest.is_dir() {
        return Err("Dictionary installation target must be a directory".into());
    }
    let mut published = BTreeSet::new();
    for name in names {
        if !dictionary_bundle_name(name) {
            return Err(format!("Unexpected dictionary install artifact: {name}").into());
        }
        checked_file(stage, name)?;
        let relative = Path::new(name);
        let mut parent = dest.to_path_buf();
        for component in relative
            .parent()
            .into_iter()
            .flat_map(|parent| parent.components())
        {
            parent.push(component.as_os_str());
            if parent.is_symlink() || (parent.exists() && !parent.is_dir()) {
                return Err(
                    format!("Unsafe dictionary target parent: {}", parent.display()).into(),
                );
            }
        }
        existing_file(&dest.join(relative))?;
        if !published.insert(relative.to_path_buf()) {
            return Err(format!("Duplicate dictionary install artifact: {name}").into());
        }
    }
    let mut obsolete = BTreeSet::new();
    if dest.exists() {
        for entry in fs::read_dir(dest)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if installed_database_name(name) && !published.contains(Path::new(name)) {
                existing_file(&entry.path())?;
                obsolete.insert(PathBuf::from(name));
            }
        }
    }
    let temp_html = html_path.with_extension("html.part");
    if existing_file(&temp_html)? {
        return Err("Previous HTML installation temporary file exists".into());
    }
    let saved_html = backup.join("index.html");
    fs::create_dir_all(backup)?;
    fs::write(&temp_html, next_html)?;
    let mut backed_up = Vec::new();
    let mut installed = Vec::new();
    let mut html_backed_up = false;
    let result = (|| -> Result<()> {
        fs::create_dir_all(dest)?;
        for relative in published.iter().chain(obsolete.iter()) {
            let current = dest.join(relative);
            if existing_file(&current)? {
                let saved = backup.join(relative);
                fs::create_dir_all(saved.parent().ok_or("Missing backup parent")?)?;
                fs::rename(&current, &saved)?;
                backed_up.push(relative.clone());
            }
        }
        for relative in &published {
            let destination = dest.join(relative);
            fs::create_dir_all(destination.parent().ok_or("Missing artifact parent")?)?;
            fs::rename(stage.join(relative), &destination)?;
            installed.push(relative.clone());
        }
        fs::rename(html_path, &saved_html)?;
        html_backed_up = true;
        fs::rename(&temp_html, html_path)?;
        Ok(())
    })();
    if let Err(error) = result {
        if html_backed_up {
            fs::rename(&saved_html, html_path)?;
        }
        for relative in installed.iter().rev() {
            fs::remove_file(dest.join(relative))?;
        }
        for relative in backed_up.iter().rev() {
            fs::rename(backup.join(relative), dest.join(relative))?;
        }
        let _ = fs::remove_file(&temp_html);
        return Err(error);
    }
    Ok(())
}

pub fn install(root: &Path, require_enrichment: bool) -> Result<()> {
    let work = root.join("target/db-workflow");
    let pointer: Value = serde_json::from_reader(fs::File::open(work.join("latest.json"))?)?;
    let relative = pointer["directory"]
        .as_str()
        .ok_or("Invalid latest dictionary pointer")?;
    let candidate = root.join(relative).canonicalize()?;
    let work_root = work.canonicalize()?;
    if !candidate.starts_with(work_root.join("generated")) {
        return Err("Dictionary candidate must be inside target/db-workflow/generated".into());
    }
    let (report, database, compressed, files) = validate(root, &candidate, require_enrichment)?;
    if compressed.is_none() {
        return Err("Dictionary bundle has no verified .duckdb.gz; run cargo make db-compress before installation".into());
    }
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let stamped = format!("hanzi-{stamp}.duckdb");
    let selected = compressed
        .as_ref()
        .map(|_| format!("{stamped}.gz"))
        .unwrap_or_else(|| stamped.clone());
    let html_path = root.join("crates/zipeek-web/index.html");
    let previous_html = fs::read_to_string(&html_path)?;
    let next_html = update_html(&previous_html, &selected)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let unique = format!("{}-{}-{nonce}", stamp, std::process::id());
    let stage = work.join("staging").join(&unique);
    let backup = work.join("backups").join(&unique);
    fs::create_dir_all(&stage)?;
    for name in &files {
        let source = checked_file(&candidate, name)?;
        let destination = stage.join(name);
        fs::create_dir_all(destination.parent().ok_or("Missing destination parent")?)?;
        fs::copy(source, destination)?;
    }
    fs::copy(checked_file(&candidate, MANIFEST)?, stage.join(MANIFEST))?;
    if database != stamped {
        fs::rename(stage.join(&database), stage.join(&stamped))?;
    }
    if let Some(gzip) = &compressed {
        if gzip != &selected {
            fs::rename(stage.join(gzip), stage.join(&selected))?;
        }
    }
    let mut report = report;
    for item in report["files"]
        .as_array_mut()
        .ok_or("Malformed artifact entries")?
    {
        if item["path"].as_str() == Some(database.as_str()) {
            item["path"] = Value::String(stamped.clone());
        }
        if compressed
            .as_ref()
            .is_some_and(|gzip| item["path"].as_str() == Some(gzip.as_str()))
        {
            item["path"] = Value::String(selected.clone());
        }
    }
    fs::write(
        stage.join(MANIFEST),
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )?;
    validate(root, &stage, require_enrichment)?;
    let dest = root.join("crates/zipeek-web/static");
    fs::create_dir_all(backup.parent().ok_or("Invalid backup destination")?)?;
    let mut installed_names = files;
    for name in &mut installed_names {
        if *name == database {
            *name = stamped.clone();
        } else if compressed.as_ref() == Some(name) {
            *name = selected.clone();
        }
    }
    installed_names.push(MANIFEST.to_owned());
    install_dictionary_files(
        &stage,
        &dest,
        &backup,
        &html_path,
        &next_html,
        &installed_names,
    )?;
    println!("Installed dictionary: {}", dest.join(&stamped).display());
    println!("HTML preload: {selected}");
    println!("Previous dictionary backup: {}", backup.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_selection_uses_verified_manifest_without_a_browser_gzip() -> super::Result<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-native-dictionary-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let bundle = root.join("crates/zipeek-web/static");
        std::fs::create_dir_all(&bundle)?;
        let name = "hanzi-20261124-214934.duckdb";
        let path = bundle.join(name);
        std::fs::write(&path, b"test dictionary")?;
        let (size, digest) = super::hash_file(&path)?;
        let mut report = serde_json::json!({"engine":"v2.0.0","storage_compatibility":"v2.0.0","files":[{"path":name,"size_bytes":size,"sha256":digest}]});
        let manifest = bundle.join("hanzi.manifest.json");
        std::fs::write(&manifest, serde_json::to_vec(&report)?)?;
        assert_eq!(super::installed_native_dictionary(&root)?.path, path);
        std::fs::write(&path, b"tampered dictionary")?;
        assert!(super::installed_native_dictionary(&root).is_err());
        report["files"][0]["path"] = "../outside.duckdb".into();
        std::fs::write(&manifest, serde_json::to_vec(&report)?)?;
        assert!(super::installed_native_dictionary(&root).is_err());
        let duplicate = report["files"][0].clone();
        report["files"].as_array_mut().unwrap().push(duplicate);
        std::fs::write(&manifest, serde_json::to_vec(&report)?)?;
        assert!(super::installed_native_dictionary(&root).is_err());
        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    use super::*;
    #[test]
    fn preload_update_preserves_manifest_hint() {
        let html = "<link rel=\"preload\" as=\"fetch\" crossorigin=\"anonymous\" href=\"hanzi.manifest.json\">\n<link rel=\"preload\" as=\"fetch\" crossorigin=\"anonymous\" href=\"hanzi-20261106-111200.duckdb.gz\">";
        let updated = update_html(html, "hanzi-20261107-120000.duckdb.gz").unwrap();
        assert!(updated.contains("href=\"hanzi.manifest.json\""));
        assert!(updated.contains("href=\"hanzi-20261107-120000.duckdb.gz\""));
        assert!(!updated.contains("hanzi-20261106-111200.duckdb.gz"));
    }
    #[test]
    fn dictionary_filename_requires_correct_timestamp() {
        assert!(dictionary_name("hanzi.duckdb"));
        assert!(dictionary_name("hanzi-20261106-111200.duckdb"));
        assert!(!dictionary_name("hanzi-foo.duckdb"));
    }
    #[test]
    fn installing_dictionary_preserves_unrelated_static_assets() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("zipeek-db-install-{}-{unique}", std::process::id()));
        let stage = root.join("stage");
        let dest = root.join("static");
        let backup = root.join("backup");
        let html = root.join("index.html");
        fs::create_dir_all(stage.join("notices")).unwrap();
        fs::create_dir_all(dest.join("practice")).unwrap();
        fs::create_dir_all(dest.join("courses")).unwrap();
        fs::create_dir_all(dest.join("brand")).unwrap();
        fs::write(dest.join("practice/white-tiger.webp"), b"white tiger").unwrap();
        fs::write(dest.join("courses/card.png"), b"course image").unwrap();
        fs::write(dest.join("brand/logo.png"), b"brand image").unwrap();
        fs::write(dest.join("custom.txt"), b"unrelated asset").unwrap();
        fs::write(dest.join("hanzi-20261106-111200.duckdb"), b"old db").unwrap();
        fs::write(dest.join("hanzi-20261106-111200.duckdb.gz"), b"old gz").unwrap();
        fs::write(dest.join("hanzi.manifest.json"), b"old manifest").unwrap();
        fs::write(stage.join("hanzi-20261117-200000.duckdb"), b"new db").unwrap();
        fs::write(stage.join("hanzi-20261117-200000.duckdb.gz"), b"new gz").unwrap();
        fs::write(stage.join("hanzi.manifest.json"), b"new manifest").unwrap();
        fs::write(&html, b"original html").unwrap();
        let names = vec![
            "hanzi-20261117-200000.duckdb".to_owned(),
            "hanzi-20261117-200000.duckdb.gz".to_owned(),
            MANIFEST.to_owned(),
        ];
        install_dictionary_files(&stage, &dest, &backup, &html, "new html", &names).unwrap();
        for (path, expected) in [
            ("practice/white-tiger.webp", b"white tiger".as_slice()),
            ("courses/card.png", b"course image".as_slice()),
            ("brand/logo.png", b"brand image".as_slice()),
            ("custom.txt", b"unrelated asset".as_slice()),
            ("hanzi-20261117-200000.duckdb", b"new db".as_slice()),
            ("hanzi-20261117-200000.duckdb.gz", b"new gz".as_slice()),
        ] {
            assert_eq!(fs::read(dest.join(path)).unwrap(), expected);
        }
        assert!(!dest.join("hanzi-20261106-111200.duckdb").exists());
        assert!(!dest.join("hanzi-20261106-111200.duckdb.gz").exists());
        assert_eq!(
            fs::read(backup.join("hanzi-20261106-111200.duckdb")).unwrap(),
            b"old db"
        );
        assert_eq!(
            fs::read(backup.join("hanzi.manifest.json")).unwrap(),
            b"old manifest"
        );
        assert_eq!(fs::read(&html).unwrap(), b"new html");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn install_rejects_unrelated_manifest_paths_without_touching_assets() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("zipeek-db-reject-{}-{unique}", std::process::id()));
        let stage = root.join("stage");
        let dest = root.join("static");
        fs::create_dir_all(stage.join("practice")).unwrap();
        fs::create_dir_all(dest.join("practice")).unwrap();
        fs::write(stage.join("practice/white-tiger.webp"), b"injected").unwrap();
        fs::write(dest.join("practice/white-tiger.webp"), b"original").unwrap();
        let html = root.join("index.html");
        fs::write(&html, b"original html").unwrap();
        assert!(
            install_dictionary_files(
                &stage,
                &dest,
                &root.join("backup"),
                &html,
                "new html",
                &["practice/white-tiger.webp".to_owned()],
            )
            .is_err()
        );
        assert_eq!(
            fs::read(dest.join("practice/white-tiger.webp")).unwrap(),
            b"original"
        );
        assert_eq!(fs::read(&html).unwrap(), b"original html");
        fs::remove_dir_all(root).unwrap();
    }
}
