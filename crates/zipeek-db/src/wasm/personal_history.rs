use js_sys::{Array, Object, Promise, Reflect};
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use wasm_bindgen_futures::future_to_promise;
use web_sys::Document;

use super::duckdb_api::{AccessMode, DuckDbModule};
use super::error::js_error;
use super::evidence_summary::PracticeOutcomeSummary;
use super::rows::table_rows;
use super::session::DbSession;
use super::{Arrow, BaseBundle, DbError};

const HISTORY_PATH: &str = "opfs://zipeek-personal-learning-v1.duckdb";
const HISTORY_LOCK: &str = "zipeek-personal-learning-v1";
const MAX_EVIDENCE_BYTES: usize = 512 * 1024;

#[derive(Deserialize)]
struct StoredEvidence {
    evidence: String,
}

#[derive(Deserialize)]
struct StoredEvidenceWithId {
    session_id: String,
    evidence: String,
}

#[derive(Deserialize)]
struct StoredSessionCount {
    session_count: String,
}

pub struct PersonalLearningHistory;

impl PersonalLearningHistory {
    pub async fn recorded_outcome_summary(
        document: &Document,
    ) -> Result<PracticeOutcomeSummary, DbError> {
        let document = document.clone();
        let callback = Closure::<dyn FnMut() -> Promise>::new(move || {
            let document = document.clone();
            future_to_promise(async move {
                let summary = summary_locked(&document)
                    .await
                    .map_err(|error| JsValue::from_str(&error.to_string()))?;
                serde_wasm_bindgen::to_value(&(
                    summary.sessions,
                    summary.scored_steps,
                    summary.completed_steps,
                    summary.skipped_steps,
                    summary.earned_points,
                    summary.possible_points,
                    summary.utc_completion_dates,
                ))
                .map_err(|error| JsValue::from_str(&error.to_string()))
            })
        });
        let value = with_history_lock(callback.as_ref().unchecked_ref())
            .await
            .map_err(|error| js_error("Summarizing Practice learning history", error))?;
        let (
            sessions,
            scored_steps,
            completed_steps,
            skipped_steps,
            earned_points,
            possible_points,
            utc_completion_dates,
        ): (u64, u64, u64, u64, u64, u64, u64) = serde_wasm_bindgen::from_value(value)
            .map_err(|error| DbError::Decode(error.to_string()))?;
        Ok(PracticeOutcomeSummary::from_transport_counts(
            sessions,
            scored_steps,
            completed_steps,
            skipped_steps,
            earned_points,
            possible_points,
            utc_completion_dates,
        ))
    }
    pub async fn recorded_session_count(document: &Document) -> Result<u64, DbError> {
        let document = document.clone();
        let callback = Closure::<dyn FnMut() -> Promise>::new(move || {
            let document = document.clone();
            future_to_promise(async move {
                let count = count_locked(&document)
                    .await
                    .map_err(|error| JsValue::from_str(&error.to_string()))?;
                Ok(JsValue::from_str(&count.to_string()))
            })
        });
        let count = with_history_lock(callback.as_ref().unchecked_ref())
            .await
            .map_err(|error| js_error("Reading personal learning history", error))?;
        parse_session_count(count.as_string().as_deref())
    }

    pub async fn erase(document: &Document) -> Result<(), DbError> {
        let document = document.clone();
        let callback = Closure::<dyn FnMut() -> Promise>::new(move || {
            let document = document.clone();
            future_to_promise(async move {
                erase_locked(&document)
                    .await
                    .map(|()| JsValue::UNDEFINED)
                    .map_err(|error| JsValue::from_str(&error.to_string()))
            })
        });
        with_history_lock(callback.as_ref().unchecked_ref())
            .await
            .map(|_| ())
            .map_err(|error| js_error("Deleting personal learning history", error))
    }

    pub async fn commit(
        document: &Document,
        session_id: &str,
        evidence: &str,
    ) -> Result<(), DbError> {
        if !valid_identifier(session_id)
            || evidence.len() > MAX_EVIDENCE_BYTES
            || evidence.is_empty()
        {
            return Err(DbError::Decode(
                "Invalid learning evidence identifier or size".into(),
            ));
        }
        let document = document.clone();
        let session_id = session_id.to_owned();
        let evidence = evidence.to_owned();
        let callback = Closure::<dyn FnMut() -> Promise>::new(move || {
            let document = document.clone();
            let session_id = session_id.clone();
            let evidence = evidence.clone();
            future_to_promise(async move {
                commit_locked(&document, &session_id, &evidence)
                    .await
                    .map(|()| JsValue::UNDEFINED)
                    .map_err(|error| JsValue::from_str(&error.to_string()))
            })
        });
        let result = with_history_lock(callback.as_ref().unchecked_ref()).await;
        result
            .map(|_| ())
            .map_err(|error| js_error("Committing personal learning history", error))
    }
}

fn parse_session_count(value: Option<&str>) -> Result<u64, DbError> {
    value
        .and_then(|count| count.parse::<u64>().ok())
        .ok_or_else(|| DbError::Decode("Invalid stored Practice session count".into()))
}

fn valid_identifier(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

pub(super) async fn with_history_lock(callback: &js_sys::Function) -> Result<JsValue, JsValue> {
    use super::browser_api::{args, call, method, property};

    let global = js_sys::global();
    let secure = property(&global, "isSecureContext")?.as_bool() == Some(true);
    let navigator = property(&global, "navigator")?;
    let storage = property(&navigator, "storage")?;
    if !secure || method(&storage, "getDirectory").is_err() {
        return Err(JsValue::from_str(
            "Learning history requires HTTPS/localhost and OPFS",
        ));
    }
    let locks = property(&navigator, "locks")?;
    if method(&locks, "request").is_err() {
        return Err(JsValue::from_str(
            "Learning history requires the Web Locks API",
        ));
    }
    let options = Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str("mode"),
        &JsValue::from_str("exclusive"),
    )?;
    Reflect::set(&options, &JsValue::from_str("ifAvailable"), &JsValue::TRUE)?;
    let task = callback.clone();
    let locked = Closure::<dyn FnMut(JsValue) -> Promise>::new(move |lock: JsValue| {
        if lock.is_null() || lock.is_undefined() {
            return Promise::reject(&JsValue::from_str(
                "Learning history is busy in another tab; retry after closing it",
            ));
        }
        match task.call0(&JsValue::UNDEFINED) {
            Ok(value) => Promise::resolve(&value),
            Err(error) => Promise::reject(&error),
        }
    });
    let requested = call(
        &locks,
        "request",
        &args(&[
            JsValue::from_str(HISTORY_LOCK),
            options.into(),
            locked.as_ref().clone(),
        ]),
    )?
    .dyn_into::<Promise>()
    .map_err(|_| JsValue::from_str("Web Locks request did not return a promise"))?;
    wasm_bindgen_futures::JsFuture::from(requested).await
}

async fn summary_locked(document: &Document) -> Result<PracticeOutcomeSummary, DbError> {
    let bundle = BaseBundle::browser_default(document)?;
    let arrow_url = bundle.module_url("arrow.mjs")?;
    let _arrow = Arrow::load(&arrow_url).await?;
    let module_url = bundle.module_url("duckdb-browser.mjs")?;
    let module = DuckDbModule::load(&module_url)
        .await
        .map_err(|error| js_error("Loading personal learning history", error))?;
    let session = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        HISTORY_PATH,
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    let summarized = async {
        let tables = session.connection.query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main' AND table_name = 'practice_sessions'"
        ).await.map_err(|error| js_error("Checking learning history table", error))?;
        if js_sys::Array::from(
            &table_rows(&tables).map_err(|error| js_error("Reading history table metadata", error))?
        ).length() == 0 {
            return Ok(PracticeOutcomeSummary::default());
        }
        let result = session.connection.query(
            "SELECT session_id, evidence_json AS evidence FROM practice_sessions LIMIT 100001"
        ).await.map_err(|error| js_error("Reading Practice evidence", error))?;
        let rows: Vec<StoredEvidenceWithId> = serde_wasm_bindgen::from_value(
            table_rows(&result).map_err(|error| js_error("Decoding Practice evidence", error))?
        ).map_err(|error| DbError::Decode(error.to_string()))?;
        if rows.len() > 100_000 {
            return Err(DbError::Decode("Too many Practice history records for this summary".into()));
        }
        let mut summary = PracticeOutcomeSummary::default();
        for row in &rows {
            summary.include(&row.session_id, &row.evidence)?;
        }
        Ok(summary)
    }.await;
    let closed = session.close().await;
    let summary = summarized?;
    closed?;
    Ok(summary)
}

async fn count_locked(document: &Document) -> Result<u64, DbError> {
    let bundle = BaseBundle::browser_default(document)?;
    let arrow_url = bundle.module_url("arrow.mjs")?;
    let _arrow = Arrow::load(&arrow_url).await?;
    let module_url = bundle.module_url("duckdb-browser.mjs")?;
    let module = DuckDbModule::load(&module_url)
        .await
        .map_err(|error| js_error("Loading personal learning history", error))?;
    let session = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        HISTORY_PATH,
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    let counted = async {
        let tables = session.connection.query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main' AND table_name = 'practice_sessions'"
        ).await.map_err(|error| js_error("Checking learning history table", error))?;
        let exists = js_sys::Array::from(
            &table_rows(&tables).map_err(|error| js_error("Reading history table metadata", error))?
        ).length() != 0;
        if !exists {
            return Ok(0);
        }
        let result = session.connection.query(
            "SELECT CAST(COUNT(*) AS VARCHAR) AS session_count FROM practice_sessions"
        ).await.map_err(|error| js_error("Counting Practice history records", error))?;
        let rows: Vec<StoredSessionCount> = serde_wasm_bindgen::from_value(
            table_rows(&result).map_err(|error| js_error("Reading Practice history count", error))?
        ).map_err(|error| DbError::Decode(error.to_string()))?;
        let [row] = rows.as_slice() else {
            return Err(DbError::Decode("Unexpected Practice history count response".into()));
        };
        row.session_count.parse::<u64>()
            .map_err(|_| DbError::Decode("Invalid Practice history count".into()))
    }.await;
    let closed = session.close().await;
    let count = counted?;
    closed?;
    Ok(count)
}

async fn erase_locked(document: &Document) -> Result<(), DbError> {
    let bundle = BaseBundle::browser_default(document)?;
    let arrow_url = bundle.module_url("arrow.mjs")?;
    let _arrow = Arrow::load(&arrow_url).await?;
    let module_url = bundle.module_url("duckdb-browser.mjs")?;
    let module = DuckDbModule::load(&module_url)
        .await
        .map_err(|error| js_error("Loading learning history for deletion", error))?;
    let session = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        HISTORY_PATH,
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    let erased = async {
        session
            .connection
            .query("BEGIN TRANSACTION")
            .await
            .map_err(|error| js_error("Beginning learning history deletion", error))?;
        let deleted = async {
            session
                .connection
                .query("DROP TABLE IF EXISTS practice_sessions")
                .await
                .map_err(|error| js_error("Erasing learning history", error))?;
            session
                .connection
                .query("DROP TABLE IF EXISTS recognition_evidence")
                .await
                .map_err(|error| js_error("Erasing recognition evidence", error))?;
            Ok::<(), DbError>(())
        }
        .await;
        if let Err(error) = deleted {
            if let Err(rollback) = session.connection.query("ROLLBACK").await {
                return Err(DbError::Decode(format!(
                    "{error}; deletion rollback failed: {rollback:?}"
                )));
            }
            return Err(error);
        }
        session
            .connection
            .query("COMMIT")
            .await
            .map_err(|error| js_error("Committing learning history deletion", error))?;
        session
            .connection
            .query("CHECKPOINT")
            .await
            .map_err(|error| js_error("Checkpointing learning history deletion", error))?;
        Ok::<(), DbError>(())
    }
    .await;
    let closed = session.close().await;
    erased?;
    closed
}

async fn commit_locked(document: &Document, id: &str, evidence: &str) -> Result<(), DbError> {
    let bundle = BaseBundle::browser_default(document)?;
    let arrow_url = bundle.module_url("arrow.mjs")?;
    let _arrow = Arrow::load(&arrow_url).await?;
    let module_url = bundle.module_url("duckdb-browser.mjs")?;
    let module = DuckDbModule::load(&module_url)
        .await
        .map_err(|error| js_error("Loading personal learning DuckDB module", error))?;
    let session = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        HISTORY_PATH,
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    let written = async {
        session.connection.query(
            "CREATE TABLE IF NOT EXISTS practice_sessions (session_id VARCHAR PRIMARY KEY, evidence_json VARCHAR NOT NULL)"
        ).await.map_err(|error| js_error("Creating learning history table", error))?;
        let params = Array::new();
        params.push(&JsValue::from_str(id));
        params.push(&JsValue::from_str(evidence));
        super::prepared::execute_prepared(
            &session.connection,
            "INSERT INTO practice_sessions (session_id, evidence_json) VALUES (?, ?) ON CONFLICT DO NOTHING",
            &params,
            "Saving learning history session",
        ).await?;
        session.connection.query("CHECKPOINT")
            .await.map_err(|error| js_error("Checkpointing learning history", error))?;
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
        let params = Array::new();
        params.push(&JsValue::from_str(id));
        let result = super::prepared::execute_prepared(
            &reopened.connection,
            "SELECT evidence_json AS evidence FROM practice_sessions WHERE session_id = ?",
            &params,
            "Verifying committed learning history",
        ).await?;
        let rows: Vec<StoredEvidence> = serde_wasm_bindgen::from_value(result)
            .map_err(|error| DbError::Decode(error.to_string()))?;
        if rows.len() != 1 || rows[0].evidence != evidence {
            return Err(DbError::Decode("Learning history session conflicts with an existing record or could not be verified".into()));
        }
        Ok::<(), DbError>(())
    }.await;
    let closed = reopened.close().await;
    verified?;
    closed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_count_requires_valid_unsigned_integer() {
        assert_eq!(parse_session_count(Some("0")).unwrap(), 0);
        assert_eq!(
            parse_session_count(Some("18446744073709551615")).unwrap(),
            u64::MAX
        );
        assert!(parse_session_count(Some("-1")).is_err());
        assert!(parse_session_count(Some("1.5")).is_err());
        assert!(parse_session_count(None).is_err());
    }

    #[test]
    fn only_uuid_identifiers_are_accepted() {
        assert!(valid_identifier("123e4567-e89b-42d3-a456-426614174000"));
        assert!(!valid_identifier("' OR 1=1 --"));
        assert!(!valid_identifier("123e4567-e89b-42d3-a456-42661417400!"));
    }
}
