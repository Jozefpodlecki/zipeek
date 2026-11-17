use crate::database::DbError;
use duckdb_neo::{Parameters, connection::Connection};

pub const SCHEMA_SQL: &str = include_str!("schema.sql");

pub fn create_schema(connection: &Connection) -> Result<(), DbError> {
    for query in SCHEMA_SQL
        .split(';')
        .map(str::trim)
        .filter(|query| !query.is_empty())
    {
        connection
            .execute(query, Parameters::None)
            .map_err(|source| DbError::Query {
                query: SCHEMA_SQL,
                source,
            })?;
    }
    Ok(())
}
