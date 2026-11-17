use std::collections::HashMap;

use js_sys::{Array, Promise};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use wasm_bindgen_futures::future_to_promise;
use web_sys::Document;

use super::duckdb_api::{AccessMode, DuckDbModule};
use super::error::js_error;
use super::personal_history::with_history_lock;
use super::rows::table_rows;
use super::session::DbSession;
use super::{Arrow, BaseBundle, DbError};

const HISTORY_PATH: &str = "opfs://zipeek-personal-learning-v1.duckdb";
const MAX_BATCH_EVENTS: usize = 256;
const MAX_EVENT_BYTES: usize = 16 * 1024;
const MAX_READ_EVENTS: usize = 2_000_000;
const TABLE_NAME: &str = "recognition_evidence";
const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS recognition_evidence (attempt_id VARCHAR PRIMARY KEY, evidence_json VARCHAR NOT NULL)";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecognitionRecord {
    pub attempt_id: String,
    pub evidence_json: String,
}

#[derive(Deserialize)]
struct TableName {
    table_name: String,
}

pub struct PersonalRecognitionHistory;

impl PersonalRecognitionHistory {
    pub async fn append_verified(
        document: &Document,
        entries: &[RecognitionRecord],
    ) -> Result<(), DbError> {
        validate_batch(entries)?;
        if entries.is_empty() {
            return Ok(());
        }
        let document = document.clone();
        let entries = entries.to_vec();
        let callback = Closure::<dyn FnMut() -> Promise>::new(move || {
            let document = document.clone();
            let entries = entries.clone();
            future_to_promise(async move {
                append_locked(&document, &entries)
                    .await
                    .map(|()| JsValue::UNDEFINED)
                    .map_err(|error| JsValue::from_str(&error.to_string()))
            })
        });
        with_history_lock(callback.as_ref().unchecked_ref())
            .await
            .map(|_| ())
            .map_err(|error| js_error("Committing recognition evidence", error))
    }

    pub async fn read_all(document: &Document) -> Result<Vec<RecognitionRecord>, DbError> {
        let document = document.clone();
        let callback = Closure::<dyn FnMut() -> Promise>::new(move || {
            let document = document.clone();
            future_to_promise(async move {
                let rows = read_locked(&document)
                    .await
                    .map_err(|error| JsValue::from_str(&error.to_string()))?;
                serde_wasm_bindgen::to_value(&rows)
                    .map_err(|error| JsValue::from_str(&error.to_string()))
            })
        });
        let result = with_history_lock(callback.as_ref().unchecked_ref())
            .await
            .map_err(|error| js_error("Reading recognition evidence", error))?;
        serde_wasm_bindgen::from_value(result).map_err(|error| DbError::Decode(error.to_string()))
    }
}

fn valid_attempt_id(id: &str) -> bool {
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

fn validate_batch(entries: &[RecognitionRecord]) -> Result<(), DbError> {
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

fn single_hanzi(value: &str) -> Option<char> {
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

async fn open_session(
    document: &Document,
) -> Result<(DbSession, DuckDbModule, BaseBundle), DbError> {
    let bundle = BaseBundle::browser_default(document)?;
    let arrow_url = bundle.module_url("arrow.mjs")?;
    let _arrow = Arrow::load(&arrow_url).await?;
    let module_url = bundle.module_url("duckdb-browser.mjs")?;
    let module = DuckDbModule::load(&module_url)
        .await
        .map_err(|error| js_error("Loading personal recognition DuckDB module", error))?;
    let session = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        HISTORY_PATH,
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    Ok((session, module, bundle))
}

async fn recognition_rows(
    connection: &super::duckdb_api::DuckDbConnection,
) -> Result<Vec<RecognitionRecord>, DbError> {
    let metadata = connection
        .query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'main' AND table_name = 'recognition_evidence'")
        .await
        .map_err(|error| js_error("Checking recognition evidence table", error))?;
    let tables: Vec<TableName> = serde_wasm_bindgen::from_value(
        table_rows(&metadata)
            .map_err(|error| js_error("Reading recognition table metadata", error))?,
    )
    .map_err(|error| DbError::Decode(error.to_string()))?;
    if !tables.iter().any(|table| table.table_name == TABLE_NAME) {
        return Ok(Vec::new());
    }
    let data = connection
        .query("SELECT attempt_id, evidence_json FROM recognition_evidence ORDER BY attempt_id LIMIT 2000001")
        .await
        .map_err(|error| js_error("Reading recognition evidence rows", error))?;
    let rows: Vec<RecognitionRecord> = serde_wasm_bindgen::from_value(
        table_rows(&data).map_err(|error| js_error("Decoding recognition rows", error))?,
    )
    .map_err(|error| DbError::Decode(error.to_string()))?;
    if rows.len() > MAX_READ_EVENTS {
        return Err(DbError::Decode(
            "Recognition history exceeds safe readback limit".into(),
        ));
    }
    Ok(rows)
}

async fn read_locked(document: &Document) -> Result<Vec<RecognitionRecord>, DbError> {
    let (session, _, _) = open_session(document).await?;
    let result = recognition_rows(&session.connection).await;
    let closed = session.close().await;
    let rows = result?;
    closed?;
    Ok(rows)
}

async fn append_locked(document: &Document, entries: &[RecognitionRecord]) -> Result<(), DbError> {
    let (session, module, bundle) = open_session(document).await?;
    let written = async {
        session.connection.query(CREATE_TABLE).await
            .map_err(|error| js_error("Creating recognition evidence table", error))?;
        session.connection.query("BEGIN TRANSACTION").await
            .map_err(|error| js_error("Beginning recognition evidence transaction", error))?;
        let inserted = async {
            for entry in entries {
                let params = Array::new();
                params.push(&JsValue::from_str(&entry.attempt_id));
                params.push(&JsValue::from_str(&entry.evidence_json));
                super::prepared::execute_prepared(
                    &session.connection,
                    "INSERT INTO recognition_evidence (attempt_id, evidence_json) VALUES (?, ?) ON CONFLICT DO NOTHING",
                    &params,
                    "Inserting recognition evidence",
                ).await?;
            }
            Ok::<(), DbError>(())
        }.await;
        if let Err(error) = inserted {
            if let Err(rollback) = session.connection.query("ROLLBACK").await {
                return Err(DbError::Decode(format!("{error}; rollback failed: {rollback:?}")));
            }
            return Err(error);
        }
        session.connection.query("COMMIT").await
            .map_err(|error| js_error("Committing recognition evidence transaction", error))?;
        session.connection.query("CHECKPOINT").await
            .map_err(|error| js_error("Checkpointing recognition evidence", error))?;
        Ok::<(), DbError>(())
    }.await;
    let closed = session.close().await;
    written?;
    closed?;

    let reopened = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        HISTORY_PATH,
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    let verified = async {
        let mut stored = Vec::with_capacity(entries.len());
        for entry in entries {
            let params = Array::new();
            params.push(&JsValue::from_str(&entry.attempt_id));
            let result = super::prepared::execute_prepared(
                &reopened.connection,
                "SELECT attempt_id, evidence_json FROM recognition_evidence WHERE attempt_id = ?",
                &params,
                "Verifying recognition evidence",
            )
            .await?;
            let rows: Vec<RecognitionRecord> = serde_wasm_bindgen::from_value(result)
                .map_err(|error| DbError::Decode(error.to_string()))?;
            stored.extend(rows);
        }
        verify_readback(entries, &stored)
    }
    .await;
    let closed = reopened.close().await;
    verified?;
    closed
}

fn verify_readback(
    incoming: &[RecognitionRecord],
    stored: &[RecognitionRecord],
) -> Result<(), DbError> {
    let index: HashMap<&str, &str> = stored
        .iter()
        .map(|row| (row.attempt_id.as_str(), row.evidence_json.as_str()))
        .collect();
    for entry in incoming {
        match index.get(entry.attempt_id.as_str()) {
            None => {
                return Err(DbError::Decode(format!(
                    "Recognition attempt {} was not persisted",
                    entry.attempt_id
                )));
            }
            Some(saved) if *saved != entry.evidence_json => {
                return Err(DbError::Decode(format!(
                    "Recognition attempt {} conflicts with stored evidence",
                    entry.attempt_id
                )));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str) -> RecognitionRecord {
        RecognitionRecord {
            attempt_id: id.into(),
            evidence_json: format!(
                r#"{{"schema_version":1,"attempt_id":"{id}","character":"学","session":{{"session_id":"session-a","completed":true}},"occurred_at_utc_ms":100,"timezone_iana":"Etc/UTC","timezone_offset_minutes":0,"local_date":"2026-11-12","mistake_allowance":0,"evaluator_id":"reference","assessment_version":"v1","correct":true,"assisted":false,"format":{{"kind":"independent_recall"}}}}"#
            ),
        }
    }

    #[test]
    fn accepts_valid_recognition_evidence() {
        assert!(validate_batch(&[record("attempt-1")]).is_ok());
        assert!(validate_batch(&[record("attempt-1"), record("attempt-1")]).is_ok());
    }

    #[test]
    fn rejects_invalid_identity_and_inconsistent_payload() {
        let mut invalid = record("attempt-1");
        invalid.attempt_id = "../attempt-1".into();
        assert!(validate_batch(&[invalid]).is_err());
        let mut invalid = record("attempt-1");
        invalid.evidence_json = invalid.evidence_json.replace("学", "学习");
        assert!(validate_batch(&[invalid]).is_err());
        let mut invalid = record("attempt-1");
        invalid.evidence_json = invalid.evidence_json.replace("attempt-1", "attempt-2");
        assert!(validate_batch(&[invalid]).is_err());
    }

    #[test]
    fn rejects_conflicting_duplicate_batch_records() {
        let mut conflicting = record("attempt-1");
        conflicting.evidence_json = conflicting
            .evidence_json
            .replace("\"correct\":true", "\"correct\":false");
        assert!(validate_batch(&[record("attempt-1"), conflicting]).is_err());
    }

    #[test]
    fn verifies_existing_retries_and_missing_or_conflicting_readback() {
        let existing = record("attempt-1");
        assert!(verify_readback(&[existing.clone()], &[existing.clone()]).is_ok());
        assert!(verify_readback(&[existing.clone()], &[]).is_err());
        let mut changed = existing.clone();
        changed.evidence_json.push(' ');
        assert!(verify_readback(&[existing], &[changed]).is_err());
    }

    #[test]
    fn rejects_oversized_batches_and_events() {
        assert!(validate_batch(&vec![record("attempt-1"); MAX_BATCH_EVENTS + 1]).is_err());
        let mut oversized = record("attempt-1");
        oversized.evidence_json = "x".repeat(MAX_EVENT_BYTES + 1);
        assert!(validate_batch(&[oversized]).is_err());
    }
}
