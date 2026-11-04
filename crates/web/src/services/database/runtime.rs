use serde::{Deserialize, de::DeserializeOwned};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use super::DbError;

#[wasm_bindgen(module = "/src/services/database/bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = openDatabase)]
    async fn open_database() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = databaseInfo)]
    fn database_info(handle: &JsValue) -> JsValue;
    #[wasm_bindgen(catch, js_name = queryDatabase)]
    async fn query_database(
        handle: &JsValue,
        sql: &str,
        parameters: JsValue,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = closeDatabase)]
    async fn close_database(handle: JsValue) -> Result<(), JsValue>;
}

struct Handle(JsValue);

impl Drop for Handle {
    fn drop(&mut self) {
        let handle = self.0.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(error) = close_database(handle).await {
                log::warn!("Database cleanup failed: {error:?}");
            }
        });
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DatabaseInfo {
    pub engine: String,
    pub row_count: u32,
}

#[derive(Clone)]
pub struct Database {
    handle: Rc<Handle>,
    pub info: DatabaseInfo,
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.handle, &other.handle)
    }
}

impl Database {
    pub async fn open() -> Result<Self, DbError> {
        let handle = Rc::new(Handle(
            open_database()
                .await
                .map_err(|value| js_error("Opening dictionary", value))?,
        ));
        let info = serde_wasm_bindgen::from_value(database_info(&handle.0))
            .map_err(|error| DbError::Decode(error.to_string()))?;
        let database = Self { handle, info };
        database
            .words("SELECT * FROM words ORDER BY id ASC LIMIT 1", vec![])
            .await?;
        log::info!(
            "Dictionary ready: {} rows, WASM engine {}",
            database.info.row_count,
            database.info.engine
        );
        Ok(database)
    }

    pub(super) async fn query<T: DeserializeOwned>(
        &self,
        sql: &str,
        parameters: Vec<serde_json::Value>,
    ) -> Result<T, DbError> {
        let parameters = serde_wasm_bindgen::to_value(&parameters)
            .map_err(|error| DbError::Decode(error.to_string()))?;
        let value = query_database(&self.handle.0, sql, parameters)
            .await
            .map_err(|value| js_error("Querying dictionary", value))?;
        serde_wasm_bindgen::from_value(value).map_err(|error| DbError::Decode(error.to_string()))
    }
}

fn js_error(operation: &'static str, value: JsValue) -> DbError {
    let message = js_sys::Reflect::get(&value, &"message".into())
        .ok()
        .and_then(|message| message.as_string())
        .or_else(|| value.as_string())
        .unwrap_or_else(|| format!("{value:?}"));
    DbError::Operation { operation, message }
}
