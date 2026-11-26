use super::{DbError, MAX_BATCH_EVENTS, MAX_EVENT_BYTES, RecognitionRecord};
use serde::Deserialize;
use std::collections::HashMap;

pub(super) fn valid_attempt_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[derive(Deserialize)]
struct RecognitionWireV1 {
    schema_version: u32,
    attempt_id: String,
    character: String,
    session: SessionWireV1,
    occurred_at_utc_ms: i64,
    timezone_iana: String,
    timezone_offset_minutes: i32,
    local_date: String,
    mistake_allowance: u8,
    evaluator_id: String,
    assessment_version: String,
    correct: bool,
    assisted: bool,
    format: FormatWireV1,
}

#[derive(Deserialize)]
struct SessionWireV1 {
    session_id: String,
    completed: bool,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FormatWireV1 {
    IndependentRecall,
    ValidatedChoice { validation_id: String },
    SupportingOnly,
}

pub(super) fn validate_batch(entries: &[RecognitionRecord]) -> Result<(), DbError> {
    if entries.len() > MAX_BATCH_EVENTS {
        return Err(DbError::Decode(
            "Too many recognition events in a single batch".into(),
        ));
    }
    let mut seen = HashMap::<&str, &str>::new();
    for entry in entries {
        if !valid_attempt_id(&entry.attempt_id)
            || entry.evidence_json.is_empty()
            || entry.evidence_json.len() > MAX_EVENT_BYTES
        {
            return Err(DbError::Decode(
                "Invalid recognition attempt ID or evidence size".into(),
            ));
        }
        let event: RecognitionWireV1 =
            serde_json::from_str(&entry.evidence_json).map_err(|error| {
                DbError::Decode(format!("Invalid recognition evidence v1: {error}"))
            })?;
        if event.schema_version != 1
            || event.attempt_id != entry.attempt_id
            || single_hanzi(&event.character).is_none()
            || !valid_attempt_id(&event.session.session_id)
            || event.timezone_iana.is_empty()
            || event.local_date.len() != 10
            || event.mistake_allowance > 2
            || event.evaluator_id.is_empty()
            || event.assessment_version.is_empty()
            || !(-840..=840).contains(&event.timezone_offset_minutes)
            || event.occurred_at_utc_ms < 0
        {
            return Err(DbError::Decode(
                "Recognition evidence contains invalid identity, timing or policy fields".into(),
            ));
        }
        if let FormatWireV1::ValidatedChoice { validation_id } = &event.format {
            if validation_id.is_empty() {
                return Err(DbError::Decode(
                    "Validated recognition evidence requires provenance".into(),
                ));
            }
        }
        let _ = (event.session.completed, event.correct, event.assisted);
        if let Some(previous) = seen.insert(&entry.attempt_id, &entry.evidence_json) {
            if previous != entry.evidence_json {
                return Err(DbError::Decode(
                    "Conflicting attempt IDs in recognition batch".into(),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn single_hanzi(value: &str) -> Option<char> {
    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        (Some(character), None)
            if ('\u{3400}'..='\u{9fff}').contains(&character)
                || ('\u{20000}'..='\u{2fa1f}').contains(&character) =>
        {
            Some(character)
        }
        _ => None,
    }
}
