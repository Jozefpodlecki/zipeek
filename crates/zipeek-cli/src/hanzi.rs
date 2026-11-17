use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    hex
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn is_han(character: char) -> bool {
    matches!(character as u32,
        0x2E80..=0x2EFF | 0x2F00..=0x2FDF | 0x3007 | 0x31C0..=0x31EF |
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF |
        0x20000..=0x2A6DF | 0x2A700..=0x2B73F | 0x2B740..=0x2B81F |
        0x2B820..=0x2CEAF | 0x2CEB0..=0x2EBEF | 0x30000..=0x323AF
    )
}

fn valid_character(stem: &str) -> Result<char> {
    let mut characters = stem.chars();
    let character = characters
        .next()
        .ok_or_else(|| invalid("empty Hanzi filename"))?;
    if !is_han(character) || characters.next().is_some() {
        return Err(invalid(format!("invalid Hanzi filename: {stem}")).into());
    }
    Ok(character)
}

fn stroke_data(character: char, input: &Value) -> Result<Value> {
    let strokes = input
        .get("strokes")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("missing strokes: {character}")))?;
    let medians = input
        .get("medians")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("missing medians: {character}")))?;
    if strokes.is_empty() || strokes.len() > 100 || strokes.len() != medians.len() {
        return Err(invalid(format!("invalid stroke count: {character}")).into());
    }
    for (index, (stroke, median)) in strokes.iter().zip(medians).enumerate() {
        let path = stroke
            .as_str()
            .ok_or_else(|| invalid(format!("invalid path {character}:{index}")))?;
        if path.len() > 100_000
            || !path.bytes().all(|b| {
                b.is_ascii_digit()
                    || matches!(
                        b,
                        b' ' | b'\n'
                            | b'\r'
                            | b'\t'
                            | b'.'
                            | b','
                            | b'+'
                            | b'-'
                            | b'M'
                            | b'm'
                            | b'L'
                            | b'l'
                            | b'H'
                            | b'h'
                            | b'V'
                            | b'v'
                            | b'C'
                            | b'c'
                            | b'S'
                            | b's'
                            | b'Q'
                            | b'q'
                            | b'T'
                            | b't'
                            | b'A'
                            | b'a'
                            | b'Z'
                            | b'z'
                            | b'E'
                            | b'e'
                    )
            })
        {
            return Err(invalid(format!("invalid path {character}:{index}")).into());
        }
        let points = median
            .as_array()
            .ok_or_else(|| invalid(format!("invalid median {character}:{index}")))?;
        if points.is_empty()
            || points.len() > 10_000
            || points.iter().any(|point| {
                let Some(coords) = point.as_array() else {
                    return true;
                };
                if coords.len() != 2 {
                    return true;
                }
                let (Some(x), Some(y)) = (coords[0].as_f64(), coords[1].as_f64()) else {
                    return true;
                };
                !x.is_finite()
                    || !y.is_finite()
                    || !(0.0..=1024.0).contains(&x)
                    || !(-124.0..=900.0).contains(&y)
            })
        {
            return Err(invalid(format!("invalid median {character}:{index}")).into());
        }
    }
    Ok(json!({"strokes": strokes, "medians": medians}))
}

const SOURCE_REPOSITORY: &str = "https://github.com/chanind/hanzi-writer-data.git";

fn references_ready(output: &Path) -> bool {
    let bundle_path = output.join("characters.json");
    let license_path = output.join("ARPHICPL.TXT");
    let manifest_path = output.join("manifest.json");
    if [
        bundle_path.as_path(),
        license_path.as_path(),
        manifest_path.as_path(),
    ]
    .iter()
    .any(|path| !path.is_file() || fs::metadata(path).map_or(true, |info| info.len() == 0))
    {
        return false;
    }
    let Ok(bytes) = fs::read(&bundle_path) else {
        return false;
    };
    let Ok(bundle) = serde_json::from_slice::<Value>(&bytes) else {
        return false;
    };
    let Ok(manifest) = fs::read(&manifest_path) else {
        return false;
    };
    let Ok(manifest) = serde_json::from_slice::<Value>(&manifest) else {
        return false;
    };
    let digest = sha256_hex(&bytes);
    bundle["version"] == 1
        && bundle["characters"]
            .as_object()
            .is_some_and(|items| !items.is_empty())
        && manifest["bundle_sha256"].as_str() == Some(digest.as_str())
        && manifest["count"].as_u64()
            == bundle["characters"]
                .as_object()
                .map(|items| items.len() as u64)
}

fn run_git(root: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .current_dir(root)
        .args(args)
        .status()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Git is required to prepare Hanzi data: {error}"),
            )
        })?;
    if !status.success() {
        return Err(invalid(format!(
            "Git command failed with status {status}: git {}",
            args.join(" ")
        ))
        .into());
    }
    Ok(())
}

pub fn prepare_from_upstream(root: &Path, force: bool) -> Result<usize> {
    let output = root.join("crates/zipeek-web/static/hanzi-strokes");
    if references_ready(&output) && !force {
        println!("Hanzi references are already prepared; reusing the existing bundle.");
        let manifest: Value = serde_json::from_slice(&fs::read(output.join("manifest.json"))?)?;
        return Ok(manifest["count"].as_u64().unwrap_or(0) as usize);
    }

    let source_root = root.join("external/hanzi-writer-data");
    let input = source_root.join("data");
    let license = source_root.join("ARPHICPL.TXT");
    if !input.is_dir() || !license.is_file() {
        if source_root.exists() {
            return Err(invalid(format!(
                "Incomplete Hanzi Writer data at {}. Remove it and retry; only data/ and ARPHICPL.TXT are needed.",
                source_root.display()
            )).into());
        }
        fs::create_dir_all(root.join("external"))?;
        let destination = "external/hanzi-writer-data";
        let fetched = (|| -> Result<()> {
            run_git(
                root,
                &[
                    "clone",
                    "--quiet",
                    "--depth",
                    "1",
                    "--filter=blob:none",
                    "--sparse",
                    "--no-checkout",
                    SOURCE_REPOSITORY,
                    destination,
                ],
            )?;
            run_git(
                root,
                &[
                    "-C",
                    destination,
                    "sparse-checkout",
                    "set",
                    "--no-cone",
                    "--",
                    "/data/",
                    "/ARPHICPL.TXT",
                ],
            )?;
            run_git(root, &["-C", destination, "checkout", "--quiet", "HEAD"])
        })();
        if let Err(error) = fetched {
            let _ = fs::remove_dir_all(&source_root);
            return Err(error);
        }
    }
    if !input.is_dir() || !license.is_file() {
        return Err(invalid(format!(
            "Sparse checkout is missing data/ or ARPHICPL.TXT under {}",
            source_root.display()
        ))
        .into());
    }
    prepare_references(&input, &output, &license)
}

pub fn prepare_references(input: &Path, output: &Path, license: &Path) -> Result<usize> {
    if !input.is_dir() || !license.is_file() {
        return Err(invalid("input must be a directory and ARPHICPL.TXT must be a file").into());
    }
    if fs::metadata(license)?.len() == 0 {
        return Err(invalid("ARPHICPL.TXT must not be empty").into());
    }

    let mut sources: Vec<PathBuf> = fs::read_dir(input)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<_>>()?;

    sources.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
            && path.file_name().is_some_and(|name| name != "all.json")
    });
    sources.sort();
    if sources.is_empty() {
        return Err(invalid("no character JSON files found").into());
    }

    let mut assets = Vec::with_capacity(sources.len());
    let mut characters = serde_json::Map::new();
    for source in &sources {
        if !source.is_file() {
            return Err(invalid(format!("not a character file: {}", source.display())).into());
        }
        let stem = source
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| invalid(format!("invalid filename: {}", source.display())))?;
        let character = valid_character(stem)?;
        let input_data: Value = serde_json::from_slice(&fs::read(source)?)?;
        let data = stroke_data(character, &input_data)?;
        let filename = format!("{:x}.json", character as u32);
        let mut bytes = serde_json::to_vec(&data)?;
        bytes.push(b'\n');
        let hash = sha256_hex(&bytes);
        let count = data["strokes"]
            .as_array()
            .ok_or_else(|| invalid(format!("missing validated strokes: {character}")))?
            .len();
        assets.push(json!({"character": character.to_string(), "file": filename, "sha256": hash, "strokes": count}));
        let id = format!("{:x}", character as u32);
        if characters.insert(id, data).is_some() {
            return Err(invalid(format!("duplicate character: {character}")).into());
        }
    }

    let count = characters.len();
    let bundle = json!({"version": 1, "characters": characters});
    let mut bundle_bytes = serde_json::to_vec(&bundle)?;
    bundle_bytes.push(b'\n');
    let bundle_sha256 = sha256_hex(&bundle_bytes);
    fs::create_dir_all(output)?;
    let temporary = output.join("characters.json.tmp");
    fs::write(&temporary, &bundle_bytes)?;
    fs::rename(&temporary, output.join("characters.json"))?;
    for entry in fs::read_dir(output)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
            && path
                .file_stem()
                .and_then(|name| name.to_str())
                .is_some_and(|stem| {
                    !stem.is_empty() && stem.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
        {
            fs::remove_file(path)?;
        }
    }
    fs::copy(license, output.join("ARPHICPL.TXT"))?;
    fs::write(
        output.join("MODIFICATIONS.txt"),
        "Zipeek reference assets: generated from Hanzi Writer Data. Original glyph paths and medians are retained; unused input metadata was removed and characters were keyed by Unicode codepoint in characters.json. Packaged by zipeek-cli hanzi prepare-references. Derived from Make Me a Hanzi / Arphic font data under ARPHIC PUBLIC LICENSE. Source: https://github.com/chanind/hanzi-writer-data\n",
    )?;
    let manifest = json!({"format": 2, "source": "hanzi-writer-data / Make Me a Hanzi", "count": count, "bundle": "characters.json", "bundle_sha256": bundle_sha256, "assets": assets});
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    manifest_bytes.push(b'\n');
    fs::write(output.join("manifest.json"), manifest_bytes)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_bundle_reuse_is_verified_without_network_access() {
        let root =
            std::env::temp_dir().join(format!("zipeek-hanzi-reuse-test-{}", std::process::id(),));
        let _ = fs::remove_dir_all(&root);
        let output = root.join("crates/zipeek-web/static/hanzi-strokes");
        fs::create_dir_all(&output).unwrap();
        let bundle = b"{\"version\":1,\"characters\":{\"4eba\":{\"strokes\":[\"M 0 0\"],\"medians\":[[[0,0]]]}}}\n";
        fs::write(output.join("characters.json"), bundle).unwrap();
        fs::write(output.join("ARPHICPL.TXT"), "test licence").unwrap();
        fs::write(
            output.join("manifest.json"),
            serde_json::to_vec(&json!({
                "bundle_sha256": sha256_hex(bundle),
                "count": 1,
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(references_ready(&output));
        assert_eq!(prepare_from_upstream(&root, false).unwrap(), 1);
        assert!(!root.join("external/hanzi-writer-data").exists());
        fs::write(output.join("characters.json"), "corrupt").unwrap();
        assert!(!references_ready(&output));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn git_checkout_uses_repository_relative_path() {
        let root = std::env::temp_dir().join(format!(
            "zipeek-hanzi-git-path-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("external")).unwrap();
        run_git(&root, &["init", "--bare", "upstream.git"]).unwrap();
        run_git(&root, &["clone", "--quiet", "upstream.git", "external/hanzi-writer-data"])
        .unwrap();
        run_git(
            &root,
            &[
                "-C",
                "external/hanzi-writer-data",
                "rev-parse",
                "--is-inside-work-tree",
            ],
        )
        .unwrap();
        assert!(root.join("external/hanzi-writer-data/.git").is_dir());
        let error = run_git(
            &root,
            &[
                "clone",
                "--quiet",
                "missing-upstream.git",
                "external/missing-clone",
            ],
        )
        .unwrap_err();
        assert!(error.to_string().contains("Git command failed"));
        assert!(!root.join("external/missing-clone").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sha256_hex_matches_standard_test_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
        assert_eq!(sha256_hex(b"").len(), 64);
    }

    #[test]
    fn validates_reference_paths_and_medians() {
        let valid = json!({"strokes": ["M 0 0 L 10 10"], "medians": [[[0,900],[1024,-124]]], "radical": "unused"});
        let filtered = stroke_data('人', &valid).unwrap();
        assert!(filtered.get("radical").is_none());
        assert_eq!(filtered["strokes"], valid["strokes"]);
        assert!(
            stroke_data(
                '人',
                &json!({"strokes": ["<script>"], "medians": [[[0,0]]]})
            )
            .is_err()
        );
        assert!(stroke_data('人', &json!({"strokes": ["M 0 0"], "medians": []})).is_err());
        assert!(
            stroke_data(
                '人',
                &json!({"strokes": ["M 0 0"], "medians": [[[5000,0]]]})
            )
            .is_err()
        );
        assert!(valid_character("A").is_err());
        assert!(valid_character("人人").is_err());
        assert_eq!(valid_character("𠀀").unwrap(), '𠀀');
        assert_eq!(valid_character("⺀").unwrap(), '⺀');
        assert_eq!(valid_character("⼀").unwrap(), '⼀');
        assert_eq!(valid_character("㇀").unwrap(), '㇀');
    }

    #[test]
    fn rejects_empty_source_and_license_without_writing() {
        let dir = std::env::temp_dir().join(format!(
            "zipeek-hanzi-test-{}-{}",
            std::process::id(),
            "empty"
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("input")).unwrap();
        fs::write(dir.join("ARPHICPL.TXT"), "test-license").unwrap();
        assert!(
            prepare_references(
                &dir.join("input"),
                &dir.join("output"),
                &dir.join("ARPHICPL.TXT")
            )
            .is_err()
        );
        assert!(!dir.join("output").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn ignores_upstream_aggregate_but_imports_character_and_radical_files() {
        let dir = std::env::temp_dir().join(format!(
            "zipeek-hanzi-test-{}-aggregate",
            std::process::id(),
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("input")).unwrap();
        fs::write(dir.join("ARPHICPL.TXT"), "test-license").unwrap();
        fs::write(dir.join("input/all.json"), r#"{"aggregate":true}"#).unwrap();
        for character in ['人', '⺀'] {
            fs::write(
                dir.join("input").join(format!("{character}.json")),
                r#"{"strokes":["M 0 0 L 10 10"],"medians":[[[0,900],[1024,-124]]]}"#,
            )
            .unwrap();
        }

        let count = prepare_references(
            &dir.join("input"),
            &dir.join("output"),
            &dir.join("ARPHICPL.TXT"),
        )
        .unwrap();
        assert_eq!(count, 2);
        assert!(dir.join("output/characters.json").is_file());
        assert!(!dir.join("output/4eba.json").exists());
        let bundle: Value =
            serde_json::from_slice(&fs::read(dir.join("output/characters.json")).unwrap()).unwrap();
        assert_eq!(
            bundle["characters"]["4eba"]["strokes"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            bundle["characters"]["2e80"]["strokes"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(!dir.join("output/all.json").exists());
        let manifest: Value =
            serde_json::from_slice(&fs::read(dir.join("output/manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["count"], 2);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn unexpected_multicharacter_json_is_still_an_error() {
        let dir =
            std::env::temp_dir().join(format!("zipeek-hanzi-test-{}-unknown", std::process::id(),));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("input")).unwrap();
        fs::write(dir.join("ARPHICPL.TXT"), "test-license").unwrap();
        fs::write(dir.join("input/overview.json"), "{}").unwrap();
        assert!(
            prepare_references(
                &dir.join("input"),
                &dir.join("output"),
                &dir.join("ARPHICPL.TXT"),
            )
            .is_err()
        );
        assert!(!dir.join("output").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn packages_character_and_license_with_hash_manifest() {
        let dir = std::env::temp_dir().join(format!(
            "zipeek-hanzi-test-{}-{}",
            std::process::id(),
            "complete"
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("input")).unwrap();
        fs::write(dir.join("ARPHICPL.TXT"), "test-license").unwrap();
        fs::write(
            dir.join("input/人.json"),
            r#"{"strokes":["M 0 0 L 10 10"],"medians":[[[0,900],[1024,-124]]],"radical":"unused"}"#,
        )
        .unwrap();
        assert_eq!(
            prepare_references(
                &dir.join("input"),
                &dir.join("output"),
                &dir.join("ARPHICPL.TXT")
            )
            .unwrap(),
            1
        );
        let bytes = fs::read(dir.join("output/characters.json")).unwrap();
        let bundle: Value = serde_json::from_slice(&bytes).unwrap();
        let output_data = &bundle["characters"]["4eba"];
        assert!(output_data.get("radical").is_none());
        assert_eq!(
            fs::read_to_string(dir.join("output/ARPHICPL.TXT")).unwrap(),
            "test-license"
        );
        let manifest: Value =
            serde_json::from_slice(&fs::read(dir.join("output/manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["count"], 1);
        assert_eq!(manifest["bundle_sha256"], sha256_hex(&bytes));
        assert_eq!(manifest["bundle"], "characters.json");
        fs::remove_dir_all(dir).unwrap();
    }
}
