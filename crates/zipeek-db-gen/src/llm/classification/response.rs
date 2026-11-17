use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use zipeek_core::Word;

use super::{ClassificationError, ClassificationLabels, validate_labels};
use crate::import::SourceSense;

fn json_payload(text: &str) -> &str {
    let mut text = text.trim_start_matches('\u{feff}').trim();
    if let Some(reasoning) = text.strip_prefix("<think>") {
        if let Some((_, final_text)) = reasoning.split_once("</think>") {
            text = final_text.trim();
        }
    }
    if let Some(fenced) = text
        .strip_prefix("```json")
        .or_else(|| text.strip_prefix("```JSON"))
        .or_else(|| text.strip_prefix("```"))
    {
        text = fenced.trim();
        if let Some(without_closing) = text.strip_suffix("```") {
            text = without_closing.trim();
        }
    }
    text
}

pub(super) fn response_preview(text: &str) -> String {
    text.chars()
        .take(120)
        .collect::<String>()
        .escape_debug()
        .to_string()
}

pub fn parse_model_response(text: &str) -> Result<ClassificationLabels, ClassificationError> {
    if text.trim().is_empty() {
        return Err(ClassificationError::Invalid(
            "model returned no final text; inspect LM Studio reasoning output, token budget, and AiMux streaming aggregation".into(),
        ));
    }
    let labels: ClassificationLabels = serde_json::from_str(json_payload(text))?;
    validate_labels(&labels)?;
    Ok(labels)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchResponse {
    results: Vec<BatchRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchRecord {
    word_id: i32,
    sense_order: i32,
    pos: Vec<String>,
    categories: Vec<String>,
}

pub fn parse_batch_response(
    text: &str,
    expected: &[(Word, SourceSense)],
) -> Result<Vec<ClassificationLabels>, ClassificationError> {
    if text.trim().is_empty() {
        return Err(ClassificationError::Invalid(
            "empty batch model response".into(),
        ));
    }
    let response: BatchResponse = serde_json::from_str(json_payload(text))?;
    let mut entries = BTreeMap::new();
    for record in response.results {
        let key = (record.word_id, record.sense_order);
        let labels = ClassificationLabels {
            pos: record.pos,
            categories: record.categories,
        };
        validate_labels(&labels)?;
        if entries.insert(key, labels).is_some() {
            return Err(ClassificationError::Invalid(format!(
                "duplicate result {key:?}"
            )));
        }
    }
    let mut ordered = Vec::with_capacity(expected.len());
    let mut seen = BTreeSet::new();
    for (word, sense) in expected {
        let key = (sense.word_id.as_i32(), sense.sense_order);
        if word.id != sense.word_id || !seen.insert(key) {
            return Err(ClassificationError::Invalid(format!(
                "invalid requested sense {key:?}"
            )));
        }
        ordered.push(
            entries
                .remove(&key)
                .ok_or_else(|| ClassificationError::Invalid(format!("missing result {key:?}")))?,
        );
    }
    if !entries.is_empty() {
        return Err(ClassificationError::Invalid("unexpected result IDs".into()));
    }
    Ok(ordered)
}
