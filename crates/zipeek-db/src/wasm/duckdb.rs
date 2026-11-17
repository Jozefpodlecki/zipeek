use std::rc::Rc;

use js_sys::{Array, Object, Reflect};
use serde::{Deserialize, de::DeserializeOwned};
use wasm_bindgen::JsValue;
use web_sys::Document;

use super::deadline::with_deadline;
use super::duckdb_api::{AccessMode, DuckDbModule};
use super::rows::table_rows;
use super::session::DbSession;
use super::transport::download_dictionary;
use super::{Arrow, BaseBundle, DbError, DuckDbError, error::js_error};

struct Handle(DbSession);

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DuckDbInfo {
    pub engine: String,
    pub native_engine: String,
    pub storage_compatibility: String,
    pub row_count: u32,
}

#[derive(Debug, Deserialize)]
struct EngineMetadata {
    engine: String,
    row_count: u32,
}

#[derive(Clone)]
pub struct DuckDb {
    session: Rc<Handle>,
    info: DuckDbInfo,
}

impl PartialEq for DuckDb {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.session, &other.session)
    }
}

impl DuckDb {
    pub async fn from_bundle(bundle: BaseBundle, document: &Document) -> Result<Self, DbError> {
        let window = document.default_view().ok_or_else(|| DbError::Operation {
            operation: "Opening dictionary",
            message: "Document window unavailable".into(),
        })?;
        with_deadline(
            &window,
            45_000,
            "Opening dictionary",
            Self::initialize(bundle, document),
        )
        .await
    }

    async fn initialize(bundle: BaseBundle, document: &Document) -> Result<Self, DbError> {
        let arrow_url = bundle.module_url("arrow.mjs")?;
        let _arrow = Arrow::load(&arrow_url).await?;
        let module_url = bundle.module_url("duckdb-browser.mjs")?;
        let module = DuckDbModule::load(&module_url)
            .await
            .map_err(|e| DuckDbError::Module(format!("{e:?}")))?;
        let base = Object::new();
        Reflect::set(
            &base,
            &JsValue::from_str("mainModule"),
            &JsValue::from_str(&bundle.base.main_module),
        )
        .map_err(|e| js_error("Building DuckDB bundle", e))?;
        Reflect::set(
            &base,
            &JsValue::from_str("mainWorker"),
            &JsValue::from_str(&bundle.base.main_worker),
        )
        .map_err(|e| js_error("Building DuckDB bundle", e))?;
        let candidates = Object::new();
        Reflect::set(&candidates, &JsValue::from_str("base"), &base)
            .map_err(|e| js_error("Building DuckDB bundle", e))?;
        let selected = module
            .select_bundle(&candidates.into())
            .await
            .map_err(|e| DuckDbError::BundleSelection(format!("{e:?}")))?;
        let main_module = Reflect::get(&selected, &JsValue::from_str("mainModule"))
            .map_err(|e| js_error("Reading selected DuckDB module", e))?
            .as_string()
            .ok_or_else(|| {
                DbError::from(DuckDbError::BundleSelection(
                    "Selected DuckDB module URL is invalid".into(),
                ))
            })?;
        let main_worker = Reflect::get(&selected, &JsValue::from_str("mainWorker"))
            .map_err(|e| js_error("Reading selected DuckDB worker", e))?
            .as_string()
            .ok_or_else(|| {
                DbError::from(DuckDbError::BundleSelection(
                    "Selected DuckDB worker URL is invalid".into(),
                ))
            })?;
        let window = document.default_view().ok_or_else(|| DbError::Operation {
            operation: "Opening dictionary",
            message: "Document window unavailable".into(),
        })?;
        let dictionary = download_dictionary(document, &window).await?;
        let session = DbSession::open(
            &module,
            &main_module,
            &main_worker,
            &dictionary.filename,
            AccessMode::ReadOnly,
            Some(&dictionary.bytes),
        )
        .await?;
        if let Err(error) = session.connection.query("INSTALL AND LOAD fts;").await {
            log::warn!("DuckDB FTS extension unavailable; English relevance search will report an error: {error:?}");
        }
        let rows = session
            .connection
            .query("SELECT version() AS engine, CAST(count(*) AS INTEGER) AS row_count FROM words")
            .await
            .map_err(|e| js_error("Reading dictionary metadata", e))
            .and_then(|table| {
                table_rows(&table).map_err(|e| js_error("Converting metadata rows", e))
            });
        let info: Result<EngineMetadata, DbError> = rows.and_then(|rows| {
            let results: Vec<EngineMetadata> =
                serde_wasm_bindgen::from_value(rows).map_err(|e| DbError::Decode(e.to_string()))?;
            results
                .into_iter()
                .next()
                .ok_or_else(|| DbError::Decode("Dictionary metadata row missing".into()))
        });
        let metadata = match info {
            Ok(info) => info,
            Err(error) => {
                if let Err(close_error) = session.close().await {
                    log::warn!("Metadata failure cleanup failed: {close_error}");
                }
                return Err(error);
            }
        };
        let info = DuckDbInfo {
            engine: metadata.engine,
            row_count: metadata.row_count,
            native_engine: dictionary.native_engine,
            storage_compatibility: dictionary.storage_compatibility,
        };
        Ok(Self {
            session: Rc::new(Handle(session)),
            info,
        })
    }

    pub fn info(&self) -> Result<DuckDbInfo, DbError> {
        Ok(self.info.clone())
    }

    pub async fn query<T: DeserializeOwned>(
        &self,
        sql: &str,
        parameters: Vec<serde_json::Value>,
    ) -> Result<T, DbError> {
        let arguments = Array::new();
        for parameter in parameters {
            let value = serde_wasm_bindgen::to_value(&parameter)
                .map_err(|error| DbError::Decode(error.to_string()))?;
            arguments.push(&value);
        }
        let rows = super::prepared::execute_prepared(
            &self.session.0.connection,
            sql,
            &arguments,
            "Executing dictionary query",
        )
        .await?;
        serde_wasm_bindgen::from_value(rows).map_err(|e| DbError::Decode(e.to_string()))
    }
}
