use crate::services::tauri_database;
use serde::{Deserialize, de::DeserializeOwned};
use web_sys::Document;
use zipeek_db::wasm::{BaseBundle, DuckDb};

use super::DbError;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DatabaseInfo {
    pub engine: String,
    pub native_engine: String,
    pub storage_compatibility: String,
    pub row_count: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct StorageStatistics {
    pub database_size: String,
    pub block_size: String,
    pub total_blocks: String,
    pub used_blocks: String,
    pub free_blocks: String,
    pub wal_size: String,
}

const STORAGE_STATISTICS_QUERY: &str = r#"
    SELECT
        CAST(database_size AS VARCHAR) AS database_size,
        CAST(block_size AS VARCHAR) AS block_size,
        CAST(total_blocks AS VARCHAR) AS total_blocks,
        CAST(used_blocks AS VARCHAR) AS used_blocks,
        CAST(free_blocks AS VARCHAR) AS free_blocks,
        CAST(wal_size AS VARCHAR) AS wal_size
    FROM pragma_database_size()
"#;

#[derive(Clone, PartialEq)]
pub struct Database {
    handle: DatabaseHandle,
    pub info: DatabaseInfo,
}

#[derive(Clone, PartialEq)]
enum DatabaseHandle {
    Browser(DuckDb),
    Native,
}

impl Database {
    pub async fn open(document: &Document, is_tauri: bool) -> Result<Self, DbError> {
        if is_tauri {
            let info = tauri_database::open()
                .await
                .map_err(|message| DbError::Operation {
                    operation: "Opening native Tauri dictionary",
                    message,
                })?;
            let database = Self {
                handle: DatabaseHandle::Native,
                info: DatabaseInfo {
                    engine: info.engine,
                    native_engine: info.native_engine,
                    storage_compatibility: info.storage_compatibility,
                    row_count: info.row_count,
                },
            };
            database
                .words("SELECT * FROM words ORDER BY id ASC LIMIT 1", vec![])
                .await?;
            return Ok(database);
        }
        let bundle = BaseBundle::browser_default(document).map_err(|error| DbError::Operation {
            operation: "Resolving DuckDB browser assets",
            message: error.to_string(),
        })?;
        let handle = DuckDb::from_bundle(bundle, document)
            .await
            .map_err(adapt_error)?;
        let db_info = handle.info().map_err(adapt_error)?;
        let database = Self {
            handle: DatabaseHandle::Browser(handle),
            info: DatabaseInfo {
                engine: db_info.engine,
                native_engine: db_info.native_engine,
                storage_compatibility: db_info.storage_compatibility,
                row_count: db_info.row_count,
            },
        };
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

    pub async fn actual_storage_tags(&self) -> Result<String, DbError> {
        #[derive(Deserialize)]
        struct TagRow {
            storage_tags: String,
        }
        let rows: Vec<TagRow> = self.query(
            "SELECT CAST(tags AS VARCHAR) AS storage_tags FROM duckdb_databases() WHERE database_name = current_database()",
            vec![],
        ).await?;
        rows.into_iter()
            .next()
            .map(|row| row.storage_tags)
            .ok_or_else(|| {
                DbError::Decode("DuckDB did not report the loaded database storage tags".into())
            })
    }

    pub async fn storage_statistics(&self) -> Result<StorageStatistics, DbError> {
        let mut rows: Vec<StorageStatistics> = self.query(STORAGE_STATISTICS_QUERY, vec![]).await?;
        if rows.len() != 1 {
            return Err(DbError::Decode(format!(
                "Expected one DuckDB storage statistics row, received {}",
                rows.len()
            )));
        }
        Ok(rows.remove(0))
    }

    pub async fn debug_select(
        &self,
        sql: &str,
    ) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, DbError> {
        self.query(sql, vec![]).await
    }

    pub(super) async fn query<T: DeserializeOwned>(
        &self,
        sql: &str,
        parameters: Vec<serde_json::Value>,
    ) -> Result<T, DbError> {
        match &self.handle {
            DatabaseHandle::Browser(handle) => {
                handle.query(sql, parameters).await.map_err(adapt_error)
            }
            DatabaseHandle::Native => {
                tauri_database::query(sql, parameters)
                    .await
                    .map_err(|message| DbError::Operation {
                        operation: "Native DuckDB IPC query",
                        message,
                    })
            }
        }
    }
}

fn adapt_error(error: zipeek_db::wasm::DbError) -> DbError {
    match error {
        zipeek_db::wasm::DbError::Operation { operation, message } => {
            DbError::Operation { operation, message }
        }
        zipeek_db::wasm::DbError::Decode(message) => DbError::Decode(message),
        zipeek_db::wasm::DbError::DuckDb(error) => DbError::Operation {
            operation: "DuckDB operation",
            message: error.to_string(),
        },
        zipeek_db::wasm::DbError::Manifest(error) => DbError::Manifest(error.to_string()),
        zipeek_db::wasm::DbError::Bundle(error) => DbError::Operation {
            operation: "Resolving DuckDB browser assets",
            message: error.to_string(),
        },
    }
}

#[cfg(test)]
mod storage_statistics_tests {
    use super::*;

    #[test]
    fn statistics_query_exposes_only_fixed_read_only_metadata() {
        assert!(STORAGE_STATISTICS_QUERY.contains("FROM pragma_database_size()"));
        for name in [
            "database_size",
            "block_size",
            "total_blocks",
            "used_blocks",
            "free_blocks",
            "wal_size",
        ] {
            assert!(STORAGE_STATISTICS_QUERY.contains(name));
        }
        assert!(
            !STORAGE_STATISTICS_QUERY
                .to_ascii_lowercase()
                .contains("insert ")
        );
    }

    #[test]
    fn storage_statistics_decode_requires_all_fields() {
        let valid = serde_json::json!({
            "database_size":"1.0 MiB", "block_size":"262144", "total_blocks":"4",
            "used_blocks":"3", "free_blocks":"1", "wal_size":"0 bytes"
        });
        let decoded: StorageStatistics = serde_json::from_value(valid.clone()).unwrap();
        assert_eq!(decoded.block_size, "262144");
        let mut incomplete = valid;
        incomplete.as_object_mut().unwrap().remove("free_blocks");
        assert!(serde_json::from_value::<StorageStatistics>(incomplete).is_err());
    }
}
