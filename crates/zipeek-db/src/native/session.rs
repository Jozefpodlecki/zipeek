use super::{Connection, Parameters, Transaction};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("dictionary SQL statement {query} failed: {source}")]
pub struct StatementError {
    pub query: String,
    #[source]
    pub source: super::Error,
}

#[derive(Clone, Copy)]
pub struct DatabaseSession<'a> {
    connection: &'a Connection,
}

impl<'a> DatabaseSession<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn from_database(database: &'a super::GeneratorDatabase) -> Self {
        Self::new(database.connection())
    }

    pub fn execute(&self, query: &str) -> Result<(), StatementError> {
        self.connection
            .execute(query, Parameters::None)
            .map_err(|source| StatementError {
                query: query.to_owned(),
                source,
            })?;
        Ok(())
    }

    pub fn transaction(self) -> Result<Transaction<'a>, super::Error> {
        Transaction::begin(self)
    }

    pub fn connection(self) -> &'a Connection {
        self.connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::GeneratorDatabase;

    #[test]
    fn errors_preserve_failing_statement() -> Result<(), Box<dyn std::error::Error>> {
        let database = GeneratorDatabase::open()?;
        let session = DatabaseSession::from_database(&database);
        let error = session.execute("SELECT * FROM nonexistent_test_table").unwrap_err();
        assert!(error.to_string().contains("nonexistent_test_table"));
        Ok(())
    }
}
