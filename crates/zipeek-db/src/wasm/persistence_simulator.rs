use js_sys::{Array, Promise};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use wasm_bindgen_futures::future_to_promise;
use web_sys::Document;

use super::duckdb_api::{AccessMode, DuckDbModule};
use super::error::js_error;
use super::personal_history::{HISTORY_PATH, with_history_lock};
use super::prepared::execute_prepared;
use super::session::DbSession;
use super::{Arrow, BaseBundle, DbError};

const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS persistence_simulator (run_id VARCHAR PRIMARY KEY, revision INTEGER NOT NULL, payload VARCHAR NOT NULL, updated_at TIMESTAMP DEFAULT current_timestamp)";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimulationCheckpoint {
    pub run_id: String,
    pub revision: u32,
    pub verified_after_reopen: bool,
}

#[derive(Deserialize)]
struct StoredCheckpoint {
    run_id: String,
    revision: i32,
    payload: String,
}

pub struct PersonalPersistenceSimulator;

impl PersonalPersistenceSimulator {
    pub async fn save_and_verify(
        document: &Document,
        run_id: &str,
        revision: u32,
    ) -> Result<SimulationCheckpoint, DbError> {
        if !valid_run_id(run_id) || !(1..=10).contains(&revision) {
            return Err(DbError::Decode(
                "Invalid persistence simulation run or revision".into(),
            ));
        }
        let document = document.clone();
        let run_id = run_id.to_owned();
        let callback = Closure::<dyn FnMut() -> Promise>::new(move || {
            let document = document.clone();
            let run_id = run_id.clone();
            future_to_promise(async move {
                let result = save_locked(&document, &run_id, revision)
                    .await
                    .map_err(|error| JsValue::from_str(&error.to_string()))?;
                serde_wasm_bindgen::to_value(&result)
                    .map_err(|error| JsValue::from_str(&error.to_string()))
            })
        });
        let value = with_history_lock(callback.as_ref().unchecked_ref())
            .await
            .map_err(|error| js_error("Running persistence simulation", error))?;
        serde_wasm_bindgen::from_value(value).map_err(|error| DbError::Decode(error.to_string()))
    }

    pub async fn verify_latest(
        document: &Document,
    ) -> Result<Option<SimulationCheckpoint>, DbError> {
        let document = document.clone();
        let callback = Closure::<dyn FnMut() -> Promise>::new(move || {
            let document = document.clone();
            future_to_promise(async move {
                let result = verify_latest_locked(&document)
                    .await
                    .map_err(|error| JsValue::from_str(&error.to_string()))?;
                serde_wasm_bindgen::to_value(&result)
                    .map_err(|error| JsValue::from_str(&error.to_string()))
            })
        });
        let value = with_history_lock(callback.as_ref().unchecked_ref())
            .await
            .map_err(|error| js_error("Verifying persisted simulation after reload", error))?;
        serde_wasm_bindgen::from_value(value).map_err(|error| DbError::Decode(error.to_string()))
    }
}

fn valid_run_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn payload(run_id: &str, revision: u32) -> String {
    format!("simulated-session:{run_id}:revision:{revision}")
}

async fn open(document: &Document) -> Result<(DbSession, DuckDbModule, BaseBundle), DbError> {
    let bundle = BaseBundle::browser_default(document)?;
    let _arrow = Arrow::load(&bundle.module_url("arrow.mjs")?).await?;
    let module = DuckDbModule::load(&bundle.module_url("duckdb-browser.mjs")?)
        .await
        .map_err(|error| js_error("Loading personal DuckDB for persistence simulator", error))?;
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

async fn read_checkpoint(
    session: &DbSession,
    run_id: &str,
) -> Result<SimulationCheckpoint, DbError> {
    let params = Array::new();
    params.push(&JsValue::from_str(run_id));
    let value = execute_prepared(
        &session.connection,
        "SELECT run_id, revision, payload FROM persistence_simulator WHERE run_id = ?",
        &params,
        "Reading simulator checkpoint",
    )
    .await?;
    let rows: Vec<StoredCheckpoint> = serde_wasm_bindgen::from_value(value)
        .map_err(|error| DbError::Decode(error.to_string()))?;
    let [row] = rows.as_slice() else {
        return Err(DbError::Decode(
            "Simulator checkpoint missing after database reopen".into(),
        ));
    };
    let revision = u32::try_from(row.revision)
        .map_err(|_| DbError::Decode("Invalid simulator revision".into()))?;
    if row.run_id != run_id
        || row.payload != payload(run_id, revision)
        || !(1..=10).contains(&revision)
    {
        return Err(DbError::Decode(
            "Persisted simulator payload mismatch".into(),
        ));
    }
    Ok(SimulationCheckpoint {
        run_id: run_id.to_owned(),
        revision,
        verified_after_reopen: true,
    })
}

async fn save_locked(
    document: &Document,
    run_id: &str,
    revision: u32,
) -> Result<SimulationCheckpoint, DbError> {
    let (session, module, bundle) = open(document).await?;
    let written = async {
        session.connection.query(CREATE_TABLE).await
            .map_err(|error| js_error("Creating simulator table", error))?;
        session.connection.query("BEGIN TRANSACTION").await
            .map_err(|error| js_error("Beginning simulator transaction", error))?;
        let attempted = async {
            let params = Array::new();
            params.push(&JsValue::from_str(run_id));
            params.push(&JsValue::from_f64(revision as f64));
            params.push(&JsValue::from_str(&payload(run_id, revision)));
            execute_prepared(
                &session.connection,
                "INSERT INTO persistence_simulator (run_id, revision, payload, updated_at) VALUES (?, ?, ?, current_timestamp) ON CONFLICT (run_id) DO UPDATE SET revision = excluded.revision, payload = excluded.payload, updated_at = current_timestamp WHERE persistence_simulator.revision = excluded.revision - 1",
                &params,
                "Writing simulator revision",
            ).await?;
            Ok::<(), DbError>(())
        }.await;
        if let Err(error) = attempted {
            let rollback = session.connection.query("ROLLBACK").await;
            return match rollback {
                Ok(_) => Err(error),
                Err(rollback) => Err(DbError::Decode(format!("{error}; rollback failed: {rollback:?}"))),
            };
        }
        session.connection.query("COMMIT").await
            .map_err(|error| js_error("Committing simulator transaction", error))?;
        session.connection.query("CHECKPOINT").await
            .map_err(|error| js_error("Checkpointing simulated session", error))?;
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
    let verified = read_checkpoint(&reopened, run_id).await;
    let closed = reopened.close().await;
    let record = verified?;
    closed?;
    if record.revision != revision {
        return Err(DbError::Decode(
            "Stored revision is not the requested revision".into(),
        ));
    }
    Ok(record)
}

async fn verify_latest_locked(
    document: &Document,
) -> Result<Option<SimulationCheckpoint>, DbError> {
    let (session, _, _) = open(document).await?;
    let checked = async {
        let result = session.connection.query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main' AND table_name = 'persistence_simulator'"
        ).await.map_err(|error| js_error("Checking simulator table", error))?;
        let found = js_sys::Array::from(&super::rows::table_rows(&result)
            .map_err(|error| js_error("Reading simulator table list", error))?).length() != 0;
        if !found {
            return Ok(None);
        }
        let result = session.connection.query(
            "SELECT run_id, revision, payload FROM persistence_simulator ORDER BY updated_at DESC, run_id DESC LIMIT 1"
        ).await.map_err(|error| js_error("Reading latest persisted simulator session", error))?;
        let rows: Vec<StoredCheckpoint> = serde_wasm_bindgen::from_value(
            super::rows::table_rows(&result)
                .map_err(|error| js_error("Decoding simulator checkpoint", error))?
        ).map_err(|error| DbError::Decode(error.to_string()))?;
        let Some(row) = rows.first() else { return Ok(None); };
        let revision = u32::try_from(row.revision)
            .map_err(|_| DbError::Decode("Invalid persisted simulator revision".into()))?;
        if !valid_run_id(&row.run_id) || !(1..=10).contains(&revision) || row.payload != payload(&row.run_id, revision) {
            return Err(DbError::Decode("Latest simulator checkpoint failed integrity validation".into()));
        }
        Ok(Some(SimulationCheckpoint {run_id: row.run_id.clone(), revision, verified_after_reopen: true}))
    }.await;
    let closed = session.close().await;
    let result = checked?;
    closed?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{payload, valid_run_id};

    #[test]
    fn rejects_invalid_run_ids() {
        assert!(!valid_run_id(""));
        assert!(!valid_run_id("a' OR 1=1"));
        assert!(valid_run_id("018f3e10-1234-789a-bcde-123456789abc"));
    }

    #[test]
    fn payload_is_bound_to_run_and_revision() {
        assert_ne!(payload("first", 1), payload("first", 2));
        assert_ne!(payload("first", 1), payload("second", 1));
    }
}
