use std::{collections::BTreeSet, fs, path::Path};

use duckdb_neo::{Parameters, connection::Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::import::SourceSense;
use zipeek_ai_protocol::prompts::classification::{CATEGORIES, ClassificationInput, POS};
use zipeek_core::Word;

use super::LlmClient;

pub const PROMPT_VERSION: &str = zipeek_ai_protocol::prompts::classification::VERSION;
pub const TAXONOMY_VERSION: u32 = zipeek_ai_protocol::prompts::classification::TAXONOMY_VERSION;

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
        sense.word_id.as_i32(),
        sense.sense_order,
        sense.gloss,
        sense.source,
        sense.source_ref,
        sense.source_gloss_order,
    ]);
    crate::sha256_hex(input.to_string().as_bytes())
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
    if labels.pos.is_empty() {
        return Err(ClassificationError::Invalid(
            "part of speech must not be empty".into(),
        ));
    }
    if labels.pos.len() > 1 && labels.pos.iter().any(|pos| pos == "unclassified") {
        return Err(ClassificationError::Invalid(
            "unclassified cannot accompany a specific part of speech".into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_import_labels(
    labels: &ClassificationLabels,
) -> Result<(), ClassificationError> {
    validate_labels(labels)
}

mod response;
use response::response_preview;
pub use response::{parse_batch_response, parse_model_response};

fn classification_schema(batch: bool) -> serde_json::Value {
    let labels = serde_json::json!({
        "type": "object",
        "properties": {
            "pos": {"type": "array", "items": {"type": "string", "enum": POS}, "minItems": 1, "uniqueItems": true},
            "categories": {"type": "array", "items": {"type": "string", "enum": CATEGORIES}, "uniqueItems": true}
        },
        "required": ["pos", "categories"],
        "additionalProperties": false
    });
    if !batch {
        return labels;
    }
    let mut record = labels;
    if let Some(properties) = record["properties"].as_object_mut() {
        properties.insert("word_id".into(), serde_json::json!({"type":"integer"}));
        properties.insert("sense_order".into(), serde_json::json!({"type":"integer"}));
    }
    record["required"] = serde_json::json!(["word_id", "sense_order", "pos", "categories"]);
    serde_json::json!({
        "type": "object",
        "properties": {"results": {"type": "array", "items": record, "minItems": 1}},
        "required": ["results"],
        "additionalProperties": false
    })
}

fn batch_token_budget(senses: usize) -> u32 {
    (2048usize.saturating_add(senses.saturating_mul(144))).min(16384) as u32
}

pub(super) fn classification_output(batch: bool) -> aimux_core::output::Output {
    aimux_core::output::Output::Object {
        schema: classification_schema(batch),
        name: Some(
            if batch {
                "sense_batch"
            } else {
                "sense_classification"
            }
            .into(),
        ),
        description: Some("Validated dictionary-sense labels; no reasoning or prose".into()),
    }
}

fn json_with_trailing_periods(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    let mut stream = serde_json::Deserializer::from_str(trimmed).into_iter::<serde_json::Value>();
    let value = stream.next()?.ok()?;
    let suffix = trimmed.get(stream.byte_offset()..)?.trim();
    if suffix.is_empty() || suffix.len() > 2 || !suffix.bytes().all(|byte| byte == b'.') {
        return None;
    }
    Some(value.to_string())
}

fn validated_output(response: &aimux_core::generate::GenerateTextResult) -> Result<String, String> {
    match response.output() {
        Ok(value) => Ok(value.to_string()),
        Err(error) => {
            if let Some(recovered) = json_with_trailing_periods(&response.text()) {
                tracing::warn!(
                    "LM Studio appended trailing periods to valid JSON; recovering structured result"
                );
                Ok(recovered)
            } else {
                Err(error.to_string())
            }
        }
    }
}

fn reject_details(response: &aimux_core::generate::GenerateTextResult) -> String {
    format!(
        "finish={:?}; raw_finish={:?}; usage={:?}; warnings={:?}; reasoning={:?}",
        response.finish_reason(),
        response.raw_finish_reason(),
        response.usage(),
        response.warnings(),
        response.final_step().reasoning_text()
    )
}

pub async fn classify_many(
    client: &LlmClient,
    model: &str,
    items: &[(Word, SourceSense)],
) -> Result<Vec<CachedClassification>, ClassificationError> {
    if model.trim().is_empty() || items.is_empty() {
        return Err(ClassificationError::Invalid(
            "model or classification batch is empty".into(),
        ));
    }
    let inputs: Vec<_> = items
        .iter()
        .map(|(word, sense)| prompt_input(word, sense))
        .collect();
    let prompt = zipeek_ai_protocol::prompts::classification::batch_user(&inputs);
    let response = client
        .complete_classification(
            model,
            &prompt,
            &system_prompt(),
            classification_output(true),
            batch_token_budget(items.len()),
        )
        .await?;
    let text = response.text();
    let labels = validated_output(&response)
        .map_err(ClassificationError::Invalid)
        .and_then(|output| parse_batch_response(&output, items))
        .map_err(|error| {
            super::rejected_response::record_details(
                model,
                &items
                    .iter()
                    .map(|(_, sense)| (sense.word_id.as_i32(), sense.sense_order))
                    .collect::<Vec<_>>(),
                &text,
                &error.to_string(),
                &reject_details(&response),
            );
            tracing::warn!(
                batch_size = items.len(),
                response_bytes = text.len(),
                response_prefix = %response_preview(&text),
                %error,
                "batch classification response rejected"
            );
            error
        })?;
    Ok(items
        .iter()
        .zip(labels)
        .map(|((_, sense), labels)| CachedClassification {
            word_id: sense.word_id.as_i32(),
            sense_order: sense.sense_order,
            input_digest: digest(sense),
            model: model.to_owned(),
            prompt_version: PROMPT_VERSION.to_owned(),
            taxonomy_version: TAXONOMY_VERSION,
            labels,
        })
        .collect())
}

pub(crate) fn imported_record(
    sense: &SourceSense,
    model: &str,
    labels: ClassificationLabels,
) -> Result<CachedClassification, ClassificationError> {
    validate_labels(&labels)?;
    let record = CachedClassification {
        word_id: sense.word_id.as_i32(),
        sense_order: sense.sense_order,
        input_digest: digest(sense),
        model: model.to_owned(),
        prompt_version: PROMPT_VERSION.to_owned(),
        taxonomy_version: TAXONOMY_VERSION,
        labels,
    };
    record.validate(sense)?;
    Ok(record)
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
    zipeek_ai_protocol::prompts::classification::batch_system()
}

fn prompt_input<'a>(word: &'a Word, sense: &'a SourceSense) -> ClassificationInput<'a> {
    ClassificationInput {
        word_id: sense.word_id.as_i32(),
        sense_order: sense.sense_order,
        simplified: &word.simplified,
        traditional: word.traditional.as_deref().unwrap_or(&word.simplified),
        pinyin: &word.pinyin,
        english_gloss: &sense.gloss,
    }
}

pub fn prompt(word: &Word, sense: &SourceSense) -> String {
    zipeek_ai_protocol::prompts::classification::single_user(&prompt_input(word, sense))
}

pub async fn classify(
    client: &LlmClient,
    model: &str,
    word: &Word,
    sense: &SourceSense,
) -> Result<CachedClassification, ClassificationError> {
    if word.id != sense.word_id {
        return Err(ClassificationError::Invalid(
            "word and sense IDs differ".into(),
        ));
    }
    if model.trim().is_empty() {
        return Err(ClassificationError::Invalid("model is empty".into()));
    }
    let labels = super::recovery::classify_with_recovery(
        sense.word_id.as_i32(),
        sense.sense_order,
        || async {
            let response = client
                .complete_classification(
                    model,
                    &prompt(word, sense),
                    &zipeek_ai_protocol::prompts::classification::single_system(),
                    classification_output(false),
                    batch_token_budget(1),
                )
                .await?;
            let text = response.text();
            validated_output(&response)
                .map_err(ClassificationError::Invalid)
                .and_then(|output| parse_model_response(&output))
                .map_err(|error| {
                    super::rejected_response::record_details(
                        model,
                        &[(sense.word_id.as_i32(), sense.sense_order)],
                        &text,
                        &error.to_string(),
                        &reject_details(&response),
                    );
                    tracing::warn!(
                        word_id = sense.word_id.as_i32(),
                        sense_order = sense.sense_order,
                        model,
                        response_bytes = text.len(),
                        response_prefix = %response_preview(&text),
                        %error,
                        "classification response rejected"
                    );
                    error
                })
        },
    )
    .await?;
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
    let content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut result = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        result.push(serde_json::from_str(line).map_err(|error| {
            ClassificationError::Invalid(format!("cache line {}: {error}", index + 1))
        })?);
    }
    result.extend(super::cache_store::read_checkpoints(path)?);
    Ok(result)
}

pub fn write_cache(
    path: &Path,
    records: &[CachedClassification],
) -> Result<(), ClassificationError> {
    let mut lines = String::new();
    for record in records {
        lines.push_str(&serde_json::to_string(record)?);
        lines.push('\n');
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("jsonl.tmp");
    let backup = path.with_extension("jsonl.bak");
    if temporary.exists() {
        return Err(ClassificationError::Invalid(format!(
            "cache temporary file already exists: {}",
            temporary.display()
        )));
    }
    if backup.exists() {
        return Err(ClassificationError::Invalid(format!(
            "cache backup already exists: {}",
            backup.display()
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
    let input = format!(
        "{}:{}:{}:{}",
        record.model, record.prompt_version, record.taxonomy_version, record.input_digest
    );
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
    let by_key: std::collections::BTreeMap<_, _> = senses
        .iter()
        .map(|sense| ((sense.word_id.as_i32(), sense.sense_order), sense))
        .collect();
    let mut seen = BTreeSet::new();
    for record in records {
        let key = (record.word_id, record.sense_order);
        let sense = by_key.get(&key).ok_or_else(|| {
            ClassificationError::Invalid(format!(
                "cache references unknown sense {}:{}",
                key.0, key.1
            ))
        })?;
        if !seen.insert(key) {
            return Err(ClassificationError::Invalid(format!(
                "duplicate cached sense {}:{}",
                key.0, key.1
            )));
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
        Ok(()) => {
            connection.execute("COMMIT", Parameters::None)?;
            Ok(records.len())
        }
        Err(error) => {
            let _ = connection.execute("ROLLBACK", Parameters::None);
            Err(error)
        }
    }
}

#[cfg(test)]
mod response_envelope_tests;
#[cfg(test)]
mod response_tests;
#[cfg(test)]
mod smoke_output_tests;
#[cfg(test)]
mod tests;
