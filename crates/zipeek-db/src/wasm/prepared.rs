use js_sys::Array;
use wasm_bindgen::JsValue;

use super::duckdb_api::DuckDbConnection;
use super::error::js_error;
use super::rows::table_rows;
use super::{DbError, DuckDbError};

pub(super) async fn execute_prepared(
    connection: &DuckDbConnection,
    sql: &str,
    parameters: &Array,
    operation: &'static str,
) -> Result<JsValue, DbError> {
    let statement = connection
        .prepare(sql)
        .await
        .map_err(|error| js_error("Preparing DuckDB statement", error))?;
    let queried = statement
        .query_parameters(parameters)
        .await
        .map_err(|error| js_error(operation, error))
        .and_then(|table| {
            table_rows(&table).map_err(|error| js_error("Converting DuckDB query rows", error))
        });
    let closed = statement
        .close_statement()
        .await
        .map_err(|error| js_error("Closing DuckDB statement", error));
    resolve_query_and_close(queried, closed)
}

fn resolve_query_and_close<T>(
    query: Result<T, DbError>,
    close: Result<(), DbError>,
) -> Result<T, DbError> {
    match (query, close) {
        (Ok(rows), Ok(())) => Ok(rows),
        (Err(query), Ok(())) => Err(query),
        (Ok(_), Err(close)) => Err(close),
        (Err(query), Err(close)) => Err(DuckDbError::QueryAndCleanup {
            query: query.to_string(),
            cleanup: close.to_string(),
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fail(message: &str) -> DbError {
        DuckDbError::Query(message.to_owned()).into()
    }

    #[test]
    fn preserves_both_query_and_cleanup_errors() {
        let result: Result<(), DbError> =
            resolve_query_and_close(Err(fail("bad query")), Err(fail("bad close")));
        let message = result.unwrap_err().to_string();
        assert!(message.contains("bad query"));
        assert!(message.contains("bad close"));
    }

    #[test]
    fn rejects_close_failure_even_after_successful_query() {
        let result = resolve_query_and_close(Ok(12), Err(fail("bad close")));
        assert!(result.is_err());
    }
}
