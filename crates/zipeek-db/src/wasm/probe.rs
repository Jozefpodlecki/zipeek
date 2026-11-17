use js_sys::{Array, Object, Promise, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use wasm_bindgen_futures::future_to_promise;
use web_sys::Document;

use super::duckdb_api::{AccessMode, DuckDbModule};
use super::rows::table_rows;
use super::session::DbSession;
use super::{Arrow, BaseBundle, DbError, error::js_error};

const PROBE_PATH: &str = "opfs://zipeek-personal-r119-probe.duckdb";

const PROBE_LOCK_NAME: &str = "zipeek-personal-db-r119-probe";

async fn with_probe_lock(callback: &js_sys::Function) -> Result<JsValue, JsValue> {
    use super::browser_api::{args, call, method, property};

    let global = js_sys::global();
    let secure = property(&global, "isSecureContext")?.as_bool() == Some(true);
    let navigator = property(&global, "navigator")?;
    let storage = property(&navigator, "storage")?;
    if !secure || method(&storage, "getDirectory").is_err() {
        return Err(JsValue::from_str(
            "Writable DuckDB testing requires HTTPS/localhost and OPFS support",
        ));
    }
    let crypto = property(&global, "crypto")?;
    let locks = property(&navigator, "locks")?;
    if method(&crypto, "randomUUID").is_err() || method(&locks, "request").is_err() {
        return Err(JsValue::from_str(
            "This browser lacks secure identifiers or the Web Locks API",
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
                "Another tab is already using the personal database test; close it and retry",
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
            JsValue::from_str(PROBE_LOCK_NAME),
            options.into(),
            locked.as_ref().clone(),
        ]),
    )?
    .dyn_into::<Promise>()
    .map_err(|_| JsValue::from_str("Web Locks request did not return a promise"))?;
    wasm_bindgen_futures::JsFuture::from(requested).await
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeReport {
    pub path: String,
    pub total_runs: u64,
    pub storage_tags: String,
    pub engine: String,
    pub fresh_database_setting: String,
}

impl ProbeReport {
    pub fn message(&self) -> String {
        format!(
            "Fresh WASM database created and queried (compatibility setting: {}). Writable OPFS probe passed: record survived shutdown and reopen. {} run(s) in isolated storage. Engine: {}. Actual stored file tags: {}. No learning records were changed.",
            self.fresh_database_setting, self.total_runs, self.engine, self.storage_tags
        )
    }
}

pub struct PersonalDbProbe;

impl PersonalDbProbe {
    pub async fn run(document: &Document) -> Result<ProbeReport, DbError> {
        let document = document.clone();
        let callback = Closure::<dyn FnMut() -> js_sys::Promise>::new(move || {
            let document = document.clone();
            future_to_promise(async move {
                run_round_trip(&document)
                    .await
                    .and_then(|report| {
                        serde_json::to_string(&report)
                            .map_err(|error| DbError::Decode(error.to_string()))
                    })
                    .map(|report| JsValue::from_str(&report))
                    .map_err(|error| JsValue::from_str(&error.to_string()))
            })
        });
        let serialized = with_probe_lock(callback.as_ref().unchecked_ref())
            .await
            .map_err(|error| js_error("Acquiring personal database lock", error))?;
        let serialized = serialized
            .as_string()
            .ok_or_else(|| DbError::Decode("Invalid DuckDB probe result".into()))?;
        serde_json::from_str(&serialized).map_err(|error| DbError::Decode(error.to_string()))
    }
}

async fn run_round_trip(document: &Document) -> Result<ProbeReport, DbError> {
    let bundle = BaseBundle::browser_default(document)?;
    let arrow_url = bundle.module_url("arrow.mjs")?;
    let _arrow = Arrow::load(&arrow_url).await?;
    let module_url = bundle.module_url("duckdb-browser.mjs")?;
    let module = DuckDbModule::load(&module_url)
        .await
        .map_err(|error| js_error("Loading probe DuckDB module", error))?;
    let window = document.default_view().ok_or_else(|| DbError::Operation {
        operation: "Generating probe token",
        message: "Document window unavailable".into(),
    })?;
    let token = window
        .crypto()
        .map_err(|error| js_error("Accessing browser crypto", error))?
        .random_uuid();

    let fresh = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        ":memory:",
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    let fresh_result = async {
        execute_query(&fresh, "CREATE TABLE compatibility_probe (value INTEGER)").await?;
        execute_query(&fresh, "INSERT INTO compatibility_probe VALUES (42)").await?;
        let rows = execute_query(
            &fresh,
            "SELECT current_setting('storage_compatibility_version') AS compatibility, CAST(value AS INTEGER) AS value FROM compatibility_probe",
        )
        .await?;
        let rows: Vec<FreshCompatibilityRow> = serde_wasm_bindgen::from_value(rows)
            .map_err(|error| DbError::Decode(error.to_string()))?;
        let Some(row) = rows.into_iter().next() else {
            return Err(DbError::Decode("Fresh WASM database did not return its storage setting".into()));
        };
        if row.value != 42 || row.compatibility.trim().is_empty() {
            return Err(DbError::Decode("Fresh WASM database compatibility query failed".into()));
        }
        Ok::<String, DbError>(row.compatibility)
    }
    .await;
    let fresh_closed = fresh.close().await;
    let fresh_setting = fresh_result?;
    fresh_closed?;

    let first = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        PROBE_PATH,
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    let written = async {
        execute_query(
            &first,
            "CREATE TABLE IF NOT EXISTS probe_events (token VARCHAR PRIMARY KEY)",
        )
        .await?;
        execute_prepared(&first, "INSERT INTO probe_events VALUES (?)", &token).await?;
        execute_query(&first, "CHECKPOINT").await?;
        Ok::<(), DbError>(())
    }
    .await;
    let first_closed = first.close().await;
    written?;
    first_closed?;

    let second = DbSession::open(
        &module,
        &bundle.base.main_module,
        &bundle.base.main_worker,
        PROBE_PATH,
        AccessMode::ReadWrite,
        None,
    )
    .await?;
    let verified = async {
        let rows = execute_prepared(
            &second,
            "SELECT token FROM probe_events WHERE token = ?",
            &token,
        )
        .await?;
        let rows: Vec<TokenRow> = serde_wasm_bindgen::from_value(rows)
            .map_err(|error| DbError::Decode(error.to_string()))?;
        if rows.len() != 1 || rows[0].token != token {
            return Err(DbError::Decode(
                "Personal database probe could not read the committed record after reopening"
                    .into(),
            ));
        }
        let rows = execute_query(
            &second,
            "SELECT CAST(count(*) AS DOUBLE) AS total FROM probe_events",
        )
        .await?;
        let rows: Vec<CountRow> = serde_wasm_bindgen::from_value(rows)
            .map_err(|error| DbError::Decode(error.to_string()))?;
        let total = rows
            .first()
            .map(|row| row.total)
            .filter(|total| {
                total.is_finite()
                    && *total >= 1.0
                    && *total <= 9_007_199_254_740_991.0
                    && total.fract() == 0.0
            })
            .ok_or_else(|| DbError::Decode("Invalid persisted probe count".into()))?;
        let metadata = execute_query(
            &second,
            "SELECT version() AS engine, CAST(tags AS VARCHAR) AS storage_tags FROM duckdb_databases() WHERE database_name = current_database()",
        )
        .await?;
        let metadata: Vec<StorageTagRow> = serde_wasm_bindgen::from_value(metadata)
            .map_err(|error| DbError::Decode(error.to_string()))?;
        let metadata = metadata.into_iter().next().ok_or_else(|| {
            DbError::Decode("The reopened probe did not report storage metadata".into())
        })?;
        if !metadata.storage_tags.contains("storage_version") {
            return Err(DbError::Decode(format!(
                "The reopened probe did not expose a persisted storage version: {}",
                metadata.storage_tags
            )));
        }
        Ok::<ProbeReport, DbError>(ProbeReport {
            path: PROBE_PATH.to_owned(),
            total_runs: total as u64,
            storage_tags: metadata.storage_tags,
            engine: metadata.engine,
            fresh_database_setting: fresh_setting,
        })
    }
    .await;
    let second_closed = second.close().await;
    let report = verified?;
    second_closed?;
    Ok(report)
}

async fn execute_query(session: &DbSession, sql: &str) -> Result<JsValue, DbError> {
    let table = session
        .connection
        .query(sql)
        .await
        .map_err(|error| js_error("Executing probe query", error))?;
    table_rows(&table).map_err(|error| js_error("Converting probe rows", error))
}

async fn execute_prepared(session: &DbSession, sql: &str, token: &str) -> Result<JsValue, DbError> {
    let parameters = Array::new();
    parameters.push(&JsValue::from_str(token));
    super::prepared::execute_prepared(
        &session.connection,
        sql,
        &parameters,
        "Executing probe statement",
    )
    .await
}

#[derive(Deserialize)]
struct TokenRow {
    token: String,
}

#[derive(Deserialize)]
struct CountRow {
    total: f64,
}

#[derive(Deserialize)]
struct StorageTagRow {
    engine: String,
    storage_tags: String,
}

#[derive(Deserialize)]
struct FreshCompatibilityRow {
    compatibility: String,
    value: i32,
}

#[cfg(test)]
mod tests {
    use super::ProbeReport;

    #[test]
    fn storage_probe_report_preserves_actual_versions() {
        let report = ProbeReport {
            path: "opfs://probe.duckdb".into(),
            total_runs: 2,
            storage_tags: "{'storage_version': 'v1.4.0'}".into(),
            engine: "v1.4.3".into(),
            fresh_database_setting: "v1.4.0".into(),
        };
        let encoded = serde_json::to_string(&report).unwrap();
        let decoded: ProbeReport = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, report);
    }
}
