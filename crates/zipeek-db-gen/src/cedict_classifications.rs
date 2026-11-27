use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde::Deserialize;
use zipeek_core::{Word, normalize_for_comparison};

use crate::{
    import::SourceSense,
    llm::classification::{self, ClassificationLabels},
};

pub const MODEL: &str = "cedict-gloss-rules-v1";

#[derive(Debug, Deserialize)]
struct GlossClassification {
    source_line: i32,
    gloss_index: i32,
    simplified: String,
    pinyin: String,
    english_gloss: String,
    pos: Vec<String>,
    categories: Vec<String>,
    needs_review: bool,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    source_sha256: String,
    export_sha256: String,
    taxonomy_version: u32,
    senses: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    simplified: String,
    english_gloss: String,
    pinyin: String,
    labels: ClassificationLabels,
    needs_review: bool,
}

#[derive(Debug, Default)]
pub struct Summary {
    pub total: usize,
    pub exact_source: usize,
    pub exact_gloss: usize,
    pub unresolved: usize,
    pub review: usize,
    pub ambiguous: usize,
}

fn gloss_key(word: &str, pinyin: &str, gloss: &str) -> (String, String, String) {
    (
        word.to_owned(),
        normalize_for_comparison(pinyin),
        gloss.trim().to_lowercase(),
    )
}

pub fn import(
    export: &Path,
    manifest_path: &Path,
    cedict_source: &Path,
    words: &[Word],
    senses: &[SourceSense],
    cache: &Path,
) -> Result<Summary, Box<dyn std::error::Error>> {
    let manifest: Manifest = serde_json::from_slice(&fs::read(manifest_path)?)?;
    if manifest.taxonomy_version != classification::TAXONOMY_VERSION {
        return Err("CC-CEDICT export taxonomy does not match db-gen".into());
    }
    let checksum = crate::sha256_hex(&fs::read(cedict_source)?);
    if checksum != manifest.source_sha256 {
        return Err("CC-CEDICT export source checksum mismatch".into());
    }
    let export_bytes = fs::read(export)?;
    if crate::sha256_hex(&export_bytes) != manifest.export_sha256 {
        return Err("CC-CEDICT classification export checksum mismatch".into());
    }
    let data: Vec<GlossClassification> = serde_json::from_slice(&export_bytes)?;
    if data.len() != manifest.senses {
        return Err("CC-CEDICT export record count mismatch".into());
    }
    let mut source = BTreeMap::new();
    let mut lexical: BTreeMap<_, Option<Candidate>> = BTreeMap::new();
    for row in data {
        let labels = ClassificationLabels {
            pos: row.pos,
            categories: row.categories,
        };
        classification::validate_import_labels(&labels)?;
        let candidate = Candidate {
            simplified: row.simplified.clone(),
            english_gloss: row.english_gloss.clone(),
            pinyin: row.pinyin.clone(),
            labels,
            needs_review: row.needs_review,
        };
        if source
            .insert((row.source_line, row.gloss_index), candidate.clone())
            .is_some()
        {
            return Err("duplicate CC-CEDICT line/gloss locator".into());
        }
        let key = gloss_key(&row.simplified, &row.pinyin, &row.english_gloss);
        lexical
            .entry(key)
            .and_modify(|existing| {
                if existing.as_ref() != Some(&candidate) {
                    *existing = None;
                }
            })
            .or_insert(Some(candidate));
    }
    let by_word: BTreeMap<_, _> = words.iter().map(|word| (word.id.as_i32(), word)).collect();
    if by_word.len() != words.len() {
        return Err("duplicate imported word IDs".into());
    }
    let mut summary = Summary {
        total: senses.len(),
        ..Summary::default()
    };
    let mut seen = BTreeSet::new();
    let mut records = Vec::with_capacity(senses.len());
    for sense in senses {
        let id = sense.word_id.as_i32();
        if !seen.insert((id, sense.sense_order)) {
            return Err("duplicate imported sense IDs".into());
        }
        let word = by_word.get(&id).ok_or("imported sense without a word")?;
        let candidate = if sense.source == "cc-cedict-fallback" {
            let row = source
                .get(&(sense.source_ref, sense.source_gloss_order))
                .filter(|row| {
                    row.simplified == word.simplified
                        && row.english_gloss == sense.gloss
                        && normalize_for_comparison(&row.pinyin)
                            == normalize_for_comparison(
                                word.pinyin_numeric.as_deref().unwrap_or(&word.pinyin),
                            )
                });
            if row.is_some() {
                summary.exact_source += 1;
            }
            row
        } else {
            let key = gloss_key(
                &word.simplified,
                word.pinyin_numeric.as_deref().unwrap_or(&word.pinyin),
                &sense.gloss,
            );
            match lexical.get(&key) {
                Some(Some(row)) => {
                    summary.exact_gloss += 1;
                    Some(row)
                }
                Some(None) => {
                    summary.ambiguous += 1;
                    None
                }
                None => None,
            }
        };
        let labels = if let Some(row) = candidate {
            if row.needs_review {
                summary.review += 1;
            }
            row.labels.clone()
        } else {
            summary.unresolved += 1;
            ClassificationLabels {
                pos: vec!["unclassified".to_owned()],
                categories: Vec::new(),
            }
        };
        records.push(classification::imported_record(sense, MODEL, labels)?);
    }

    if cache.exists() {
        return Err(format!("classification cache already exists: {}", cache.display()).into());
    }
    classification::write_cache(cache, &records)?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imports_by_provenance_and_preserves_unmatched_as_unclassified() {
        use zipeek_core::WordId;
        let path = std::env::temp_dir().join(format!(
            "zipeek-cedict-import-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        let source = path.join("cedict.u8");
        fs::write(&source, "test source").unwrap();
        let export = path.join("export.json");
        fs::write(
            &export,
            serde_json::json!([{
                "source_line": 7, "gloss_index": 1,
                "simplified": "苹果", "pinyin": "ping2 guo3", "english_gloss": "apple (fruit)",
                "pos": ["noun"], "categories": ["food"], "needs_review": false
            }])
            .to_string(),
        )
        .unwrap();
        let manifest = path.join("manifest.json");
        fs::write(
            &manifest,
            serde_json::json!({
                "source_sha256": crate::sha256_hex(b"test source"),
                "export_sha256": crate::sha256_hex(&fs::read(&export).unwrap()),
                "taxonomy_version": classification::TAXONOMY_VERSION, "senses": 1
            })
            .to_string(),
        )
        .unwrap();
        let id = WordId::try_from(1).unwrap();
        let word = Word {
            id,
            simplified: "苹果".into(),
            traditional: None,
            pinyin: "píng guǒ".into(),
            pinyin_numeric: Some("ping2 guo3".into()),
            english: Some("apple (fruit)".into()),
            part_of_speech: vec![],
            hsk2_level: None,
            hsk3_level: None,
            frequency: None,
        };
        let fallback = SourceSense {
            word_id: id,
            sense_order: 1,
            gloss: "apple (fruit)".into(),
            source: "cc-cedict-fallback",
            source_ref: 7,
            source_gloss_order: 1,
        };
        let unmatched = SourceSense {
            word_id: id,
            sense_order: 2,
            gloss: "not in CEDICT".into(),
            source: "hsk-primary",
            source_ref: 9,
            source_gloss_order: 2,
        };
        let cache = path.join("cache.jsonl");
        let summary = import(
            &export,
            &manifest,
            &source,
            &[word],
            &[fallback, unmatched],
            &cache,
        )
        .unwrap();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.exact_source, 1);
        assert_eq!(summary.unresolved, 1);
        let records = classification::read_cache(&cache).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].labels.categories, vec!["food"]);
        assert_eq!(records[1].labels.pos, vec!["unclassified"]);
        assert!(import(&export, &manifest, &source, &[], &[], &cache).is_err());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn lexical_keys_normalize_case_and_pinyin() {
        assert_eq!(
            gloss_key("苹果", "ping2 guo3", "Apple "),
            gloss_key("苹果", "píng guǒ", "apple")
        );
        assert_ne!(
            gloss_key("苹果", "ping2 guo3", "apple"),
            gloss_key("苹果", "ping2 guo3", "computer")
        );
    }
}
