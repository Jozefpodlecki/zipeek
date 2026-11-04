use std::{collections::BTreeSet, fs, path::Path};

use duckdb_neo::{Parameters, connection::Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::import::SourceSense;
use zipeek_core::Word;

use super::LlmClient;

pub const PROMPT_VERSION: &str = super::prompts::classification::VERSION;
pub const TAXONOMY_VERSION: u32 = 1;

const POS: &[&str] = &[
    "adjective", "adjective_adverbial", "adjective_morpheme", "adjective_nominal",
    "non_predicate_adjective", "conjunction", "adverb", "adverb_morpheme",
    "interjection", "directional_locality", "morpheme", "prefix", "idiom",
    "abbreviation", "suffix", "fixed_expression", "numeral", "numeric_morpheme",
    "noun", "noun_morpheme", "personal_name", "place_name", "organization_name",
    "nominal_string", "other_proper_noun", "onomatopoeia", "preposition",
    "classifier", "pronoun", "pronoun_morpheme", "space_word", "time_word",
    "time_word_morpheme", "auxiliary", "verb", "verb_adverbial",
    "verb_morpheme", "verb_nominal", "symbol_punctuation", "unclassified",
    "modal_particle", "descriptive",
];
const CATEGORIES: &[&str] = &[
    "travel", "transport", "food", "shopping", "family", "education",
    "work", "health", "nature", "time", "home", "communication",
];

#[derive(Debug, Error)]
pub enum ClassificationError {
    #[error("classification cache I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("classification JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid classification: {0}")]
    Invalid(String),
    #[error("classification database write: {0}")]
    Database(#[from] duckdb_neo::Error),
    #[error("classification model: {0}")]
    Model(#[from] super::LlmError),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClassificationLabels {
    pub pos: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CachedClassification {
    pub word_id: i32,
    pub sense_order: i32,
    pub input_digest: String,
    pub model: String,
    pub prompt_version: String,
    pub taxonomy_version: u32,
    pub labels: ClassificationLabels,
}

fn digest(sense: &SourceSense) -> String {
    let input = serde_json::json!([
        sense.word_id.as_i32(), sense.sense_order, sense.gloss,
        sense.source, sense.source_ref, sense.source_gloss_order,
    ]);
    format!("{:x}", Sha256::digest(input.to_string().as_bytes()))
}

#[cfg(test)]
pub(super) fn digest_for_test(sense: &SourceSense) -> String {
    digest(sense)
}

fn validate_labels(labels: &ClassificationLabels) -> Result<(), ClassificationError> {
    for (values, allowed, name) in [
        (&labels.pos, POS, "part of speech"),
        (&labels.categories, CATEGORIES, "category"),
    ] {
        let mut unique = BTreeSet::new();
        for value in values {
            if !allowed.contains(&value.as_str()) || !unique.insert(value) {
                return Err(ClassificationError::Invalid(format!(
                    "unknown or repeated {name}: {value}"
                )));
            }
        }
    }
    if labels.pos.is_empty() && labels.categories.is_empty() {
        return Err(ClassificationError::Invalid("empty classification".into()));
    }
    Ok(())
}

pub fn parse_model_response(text: &str) -> Result<ClassificationLabels, ClassificationError> {
    if text.trim().is_empty() {
        return Err(ClassificationError::Invalid(
            "model returned no final text; inspect LM Studio reasoning output, token budget, and AiMux streaming aggregation".into(),
        ));
    }
    let labels: ClassificationLabels = serde_json::from_str(text.trim())?;
    validate_labels(&labels)?;
    Ok(labels)
}

impl CachedClassification {
    pub fn validate(&self, sense: &SourceSense) -> Result<(), ClassificationError> {
        if self.word_id != sense.word_id.as_i32()
            || self.sense_order != sense.sense_order
            || self.input_digest != digest(sense)
            || self.prompt_version != PROMPT_VERSION
            || self.taxonomy_version != TAXONOMY_VERSION
            || self.model.trim().is_empty()
        {
            return Err(ClassificationError::Invalid(format!(
                "stale or mismatched cache record for {}:{}",
                self.word_id, self.sense_order
            )));
        }
        validate_labels(&self.labels)
    }
}

pub fn system_prompt() -> String {
    super::prompts::classification::system(POS, CATEGORIES)
}

pub fn prompt(word: &Word, sense: &SourceSense) -> String {
    super::prompts::classification::user(word, sense)
}

pub async fn classify(
    client: &LlmClient,
    model: &str,
    word: &Word,
    sense: &SourceSense,
) -> Result<CachedClassification, ClassificationError> {
    if word.id != sense.word_id {
        return Err(ClassificationError::Invalid("word and sense IDs differ".into()));
    }
    if model.trim().is_empty() {
        return Err(ClassificationError::Invalid("model is empty".into()));
    }
    let labels = super::recovery::classify_with_recovery(
        sense.word_id.as_i32(),
        sense.sense_order,
        || async {
            let response = client
                .complete_with_instructions(model, &prompt(word, sense), &system_prompt())
                .await?;
            let text = response.text();
            parse_model_response(&text).map_err(|error| {
                tracing::warn!(
                    word_id = sense.word_id.as_i32(),
                    sense_order = sense.sense_order,
                    model,
                    response_bytes = text.len(),
                    %error,
                    "classification response rejected"
                );
                error
            })
        },
    ).await?;
    Ok(CachedClassification {
        word_id: sense.word_id.as_i32(),
        sense_order: sense.sense_order,
        input_digest: digest(sense),
        model: model.to_owned(),
        prompt_version: PROMPT_VERSION.to_owned(),
        taxonomy_version: TAXONOMY_VERSION,
        labels,
    })
}

pub fn read_cache(path: &Path) -> Result<Vec<CachedClassification>, ClassificationError> {
    let content = if path.exists() { fs::read_to_string(path)? } else { String::new() };
    let mut result = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() { continue; }
        result.push(serde_json::from_str(line).map_err(|error| {
            ClassificationError::Invalid(format!("cache line {}: {error}", index + 1))
        })?);
    }
    result.extend(super::cache_store::read_checkpoints(path)?);
    Ok(result)
}

pub fn write_cache(path: &Path, records: &[CachedClassification]) -> Result<(), ClassificationError> {
    let mut lines = String::new();
    for record in records {
        lines.push_str(&serde_json::to_string(record)?);
        lines.push('\n');
    }
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("jsonl.tmp");
    let backup = path.with_extension("jsonl.bak");
    if temporary.exists() {
        return Err(ClassificationError::Invalid(format!(
            "cache temporary file already exists: {}", temporary.display()
        )));
    }
    if backup.exists() {
        return Err(ClassificationError::Invalid(format!(
            "cache backup already exists: {}", backup.display()
        )));
    }
    fs::write(&temporary, lines)?;
    if !path.exists() {
        fs::rename(&temporary, path)?;
        return Ok(());
    }
    if let Err(error) = fs::rename(path, &backup) {
        let _ = fs::remove_file(&temporary);
        return Err(ClassificationError::Io(error));
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if let Err(restore_error) = fs::rename(&backup, path) {
            return Err(ClassificationError::Invalid(format!(
                "cache install failed: {error}; backup restoration failed: {restore_error}; backup retained at {}",
                backup.display()
            )));
        }
        let _ = fs::remove_file(&temporary);
        return Err(ClassificationError::Io(error));
    }
    fs::remove_file(&backup)?;
    Ok(())
}

fn run_id(record: &CachedClassification) -> i64 {
    let input = format!("{}:{}:{}:{}", record.model, record.prompt_version,
        record.taxonomy_version, record.input_digest);
    let hash = Sha256::digest(input.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash[..8]);
    (u64::from_be_bytes(bytes) & 0x7fff_ffff_ffff_ffff) as i64
}

pub fn apply_cache(
    connection: &Connection,
    senses: &[SourceSense],
    records: &[CachedClassification],
) -> Result<usize, ClassificationError> {
    let by_key: std::collections::BTreeMap<_, _> = senses.iter()
        .map(|sense| ((sense.word_id.as_i32(), sense.sense_order), sense)).collect();
    let mut seen = BTreeSet::new();
    for record in records {
        let key = (record.word_id, record.sense_order);
        let sense = by_key.get(&key).ok_or_else(|| ClassificationError::Invalid(
            format!("cache references unknown sense {}:{}", key.0, key.1)
        ))?;
        if !seen.insert(key) {
            return Err(ClassificationError::Invalid(format!("duplicate cached sense {}:{}", key.0, key.1)));
        }
        record.validate(sense)?;
    }
    connection.execute("BEGIN TRANSACTION", Parameters::None)?;
    let result = (|| -> Result<(), ClassificationError> {
        for record in records {
            let run = run_id(record);
            connection.execute(
                "INSERT INTO classification_runs (id, model, prompt_version, taxonomy_version, input_digest) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
                Parameters::Positional(&[&run, &record.model, &record.prompt_version,
                    &(record.taxonomy_version as i32), &record.input_digest]),
            )?;
            for pos in &record.labels.pos {
                connection.execute(
                    "INSERT INTO sense_pos (word_id, sense_order, pos_code, classification_run_id) VALUES ($1,$2,$3,$4)",
                    Parameters::Positional(&[&record.word_id, &record.sense_order, pos, &run]),
                )?;
            }
            for category in &record.labels.categories {
                connection.execute(
                    "INSERT INTO sense_categories (word_id, sense_order, category_slug, classification_run_id) VALUES ($1,$2,$3,$4)",
                    Parameters::Positional(&[&record.word_id, &record.sense_order, category, &run]),
                )?;
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => { connection.execute("COMMIT", Parameters::None)?; Ok(records.len()) }
        Err(error) => { let _ = connection.execute("ROLLBACK", Parameters::None); Err(error) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_core::WordId;

    fn sense() -> SourceSense {
        SourceSense { word_id: WordId::try_from(10).unwrap(), sense_order: 1,
            gloss: "train ticket".into(), source: "hsk-primary", source_ref: 20,
            source_gloss_order: 1 }
    }

    #[test]
    fn model_output_rejects_unknown_and_duplicate_labels() {
        assert!(parse_model_response(r#"{"pos":["noun"],"categories":["travel"]}"#).is_ok());
        assert!(parse_model_response(r#"{"pos":["silly"],"categories":[]}"#).is_err());
        assert!(parse_model_response(r#"{"pos":["noun","noun"],"categories":[]}"#).is_err());
        assert!(parse_model_response("```json\n{}\n```").is_err());
    }

    #[test]
    fn classification_instructions_are_separate_from_entry_data() {
        let id = WordId::try_from(10).unwrap();
        let word = Word {
            id,
            simplified: "车票".into(),
            traditional: None,
            pinyin: "chē piào".into(),
            pinyin_numeric: None,
            english: Some("ticket".into()),
            part_of_speech: vec![],
            hsk2_level: None,
            hsk3_level: None,
            frequency: None,
        };
        let instructions = system_prompt();
        let data = prompt(&word, &sense());
        assert!(instructions.contains("noun"));
        assert!(instructions.contains("travel"));
        assert!(instructions.contains("empty array"));
        assert!(!instructions.contains("车票"));
        assert!(data.contains("车票"));
        assert!(data.contains("train ticket"));
        assert!(!data.contains("Allowed category codes"));
    }

    #[test]
    fn input_mutation_invalidates_cache() {
        let original = sense();
        let record = CachedClassification { word_id: 10, sense_order: 1,
            input_digest: digest(&original), model: "local".into(),
            prompt_version: PROMPT_VERSION.into(), taxonomy_version: TAXONOMY_VERSION,
            labels: parse_model_response(r#"{"pos":["noun"],"categories":["travel"]}"#).unwrap() };
        record.validate(&original).unwrap();
        let mut old_prompt = record.clone();
        old_prompt.prompt_version = "sense-classification-v2".into();
        assert!(old_prompt.validate(&original).is_err());
        let mut changed = original;
        changed.gloss = "different gloss".into();
        assert!(record.validate(&changed).is_err());
    }
}

#[cfg(test)]
mod response_tests {
    use super::parse_model_response;

    #[test]
    fn empty_or_whitespace_only_generation_reports_actionable_error() {
        for text in ["", " ", "\n\t"] {
            let error = parse_model_response(text).unwrap_err().to_string();
            assert!(error.contains("no final text"), "{error}");
        }
    }

    #[test]
    fn valid_json_after_stream_aggregation_is_accepted() {
        let chunks = ["{\"pos\":[\"noun\"],", "\"categories\":[\"food\"]}"];
        let result = parse_model_response(&chunks.concat()).unwrap();
        assert_eq!(result.pos, vec!["noun".to_owned()]);
        assert_eq!(result.categories, vec!["food".to_owned()]);
    }
}
