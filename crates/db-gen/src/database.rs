use duckdb_neo::{
    Parameters,
    connection::Connection,
    environment::{Environment, StorageLocation},
};
use thiserror::Error;

use crate::policy::{ENGINE_VERSION, STORAGE_VERSION};

#[derive(Debug, Error)]
pub enum DbError {
    #[error("DuckDB {operation} failed: {source}")]
    Operation {
        operation: &'static str,
        #[source]
        source: duckdb_neo::Error,
    },
    #[error("DuckDB query {query} failed: {source}")]
    Query {
        query: &'static str,
        #[source]
        source: duckdb_neo::Error,
    },
    #[error("DuckDB query {query} returned no value; refusing generation")]
    MissingQueryValue { query: &'static str },
    #[error("DuckDB {setting} mismatch: expected {expected}, got {actual}; refusing generation")]
    VersionMismatch {
        setting: &'static str,
        expected: &'static str,
        actual: String,
    },
}

#[derive(Debug)]
pub struct GeneratorDatabase {
    connection: Connection,
    engine_version: String,
    storage_version: String,
}

impl GeneratorDatabase {
    pub fn open() -> Result<Self, DbError> {
        let environment = Environment::new().map_err(|source| DbError::Operation {
            operation: "environment initialization",
            source,
        })?;
        let database = environment
            .open(StorageLocation::InMemory)
            .map_err(|source| DbError::Operation {
                operation: "in-memory database creation",
                source,
            })?;
        let connection = database.connect().map_err(|source| DbError::Operation {
            operation: "connection creation",
            source,
        })?;
        let engine_version = query_string(&connection, "SELECT version()")?;
        tracing::info!(version = %engine_version, "DuckDB engine");
        require_version("engine version", ENGINE_VERSION, &engine_version)?;
        let storage_query = "SET storage_compatibility_version = 'v1.4.0'";
        connection
            .execute(storage_query, Parameters::None)
            .map_err(|source| DbError::Query {
                query: storage_query,
                source,
            })?;
        let storage_version = query_string(
            &connection,
            "SELECT current_setting('storage_compatibility_version')",
        )?;
        tracing::info!(version = %storage_version, "DuckDB storage compatibility");
        require_version("storage version", STORAGE_VERSION, &storage_version)?;
        Ok(Self {
            connection,
            engine_version,
            storage_version,
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn engine_version(&self) -> &str {
        &self.engine_version
    }

    pub fn storage_version(&self) -> &str {
        &self.storage_version
    }
}

fn require_version(
    setting: &'static str,
    expected: &'static str,
    actual: &str,
) -> Result<(), DbError> {
    if actual != expected {
        return Err(DbError::VersionMismatch {
            setting,
            expected,
            actual: actual.to_owned(),
        });
    }
    Ok(())
}

fn query_string(connection: &Connection, query: &'static str) -> Result<String, DbError> {
    let read = || -> duckdb_neo::Result<Option<String>> {
        let mut result = connection.query(query, Parameters::None)?;
        let Some(chunk) = result.next_chunk()? else {
            return Ok(None);
        };
        let values = chunk.get_vector_at::<String>(0)?.reader()?;
        Ok(values.iter().next().flatten().map(str::to_owned))
    };
    read()
        .map_err(|source| DbError::Query { query, source })?
        .ok_or(DbError::MissingQueryValue { query })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_connection_uses_required_engine_and_storage() -> Result<(), DbError> {
        let database = GeneratorDatabase::open()?;
        assert_eq!(database.engine_version(), "v2.0.0");
        assert_eq!(database.storage_version(), "v1.4.0");
        assert_eq!(
            query_string(database.connection(), "SELECT version()")?,
            "v2.0.0"
        );
        Ok(())
    }

    #[test]
    fn unexpected_engine_versions_refuse_generation() {
        for actual in ["v1.4.3", "v2.0.1", "v2.0.00", "v2.0.0-dev", ""] {
            let error = require_version("engine version", ENGINE_VERSION, actual)
                .expect_err("unexpected version must fail");
            assert!(matches!(error, DbError::VersionMismatch { .. }));
            assert!(error.to_string().contains(actual));
            assert!(error.to_string().contains("expected v2.0.0"));
        }
    }

    #[test]
    fn incompatible_storage_is_rejected() {
        let error = require_version("storage version", STORAGE_VERSION, "latest")
            .expect_err("default storage must not bypass the compatibility policy");
        assert!(error.to_string().contains("expected v1.4.0, got latest"));
    }

    #[test]
    fn query_errors_preserve_the_failing_sql() -> Result<(), DbError> {
        let database = GeneratorDatabase::open()?;
        let query = "SELECT missing_column FROM missing_table";
        let error = query_string(database.connection(), query).expect_err("query must fail");
        assert!(matches!(error, DbError::Query { .. }));
        assert!(error.to_string().contains(query));
        Ok(())
    }

    #[test]
    fn null_and_empty_results_are_errors() -> Result<(), DbError> {
        let database = GeneratorDatabase::open()?;
        for query in ["SELECT NULL::VARCHAR", "SELECT 'version' WHERE false"] {
            let error = query_string(database.connection(), query).expect_err("value is required");
            assert!(matches!(error, DbError::MissingQueryValue { .. }));
            assert!(error.to_string().contains(query));
        }
        Ok(())
    }
}
