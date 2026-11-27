use super::{Connection, DatabaseSession, Parameters};

pub struct Transaction<'a> {
    session: DatabaseSession<'a>,
    active: bool,
}

impl<'a> Transaction<'a> {
    pub(super) fn begin(session: DatabaseSession<'a>) -> Result<Self, super::Error> {
        session
            .connection()
            .execute("BEGIN TRANSACTION", Parameters::None)?;
        Ok(Self {
            session,
            active: true,
        })
    }

    pub fn connection(&self) -> &Connection {
        self.session.connection()
    }

    pub fn execute(&self, query: &str) -> Result<(), super::StatementError> {
        self.session.execute(query)
    }

    pub fn commit(&mut self) -> Result<(), super::Error> {
        self.session
            .connection()
            .execute("COMMIT", Parameters::None)?;
        self.active = false;
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), super::Error> {
        self.session
            .connection()
            .execute("ROLLBACK", Parameters::None)?;
        self.active = false;
        Ok(())
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if self.active {
            if let Err(error) = self.rollback() {
                tracing::error!(%error, "Could not roll back uncommitted dictionary transaction");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::GeneratorDatabase;

    fn count(connection: &Connection) -> Result<i64, Box<dyn std::error::Error>> {
        let mut result = connection.query("SELECT count(*) FROM transaction_test", Parameters::None)?;
        let chunk = result.next_chunk()?.ok_or("missing count")?;
        let vector = chunk.get_vector_at::<i64>(0)?.reader()?;
        let count = vector.iter().next().flatten().copied().ok_or("null count")?;
        Ok(count)
    }

    #[test]
    fn dropping_uncommitted_transaction_rolls_back() -> Result<(), Box<dyn std::error::Error>> {
        let database = GeneratorDatabase::open()?;
        let connection = database.connection();
        connection.execute("CREATE TABLE transaction_test (id INTEGER)", Parameters::None)?;
        {
            let transaction = DatabaseSession::new(connection).transaction()?;
            transaction.execute("INSERT INTO transaction_test VALUES (1)")?;
        }
        assert_eq!(count(connection)?, 0);
        Ok(())
    }

    #[test]
    fn failed_statement_does_not_commit_earlier_writes() -> Result<(), Box<dyn std::error::Error>> {
        let database = GeneratorDatabase::open()?;
        let connection = database.connection();
        connection.execute("CREATE TABLE transaction_test (id INTEGER)", Parameters::None)?;
        {
            let transaction = DatabaseSession::new(connection).transaction()?;
            transaction.execute("INSERT INTO transaction_test VALUES (1)")?;
            assert!(transaction.execute("INSERT INTO missing_table VALUES (1)").is_err());
        }
        assert_eq!(count(connection)?, 0);
        Ok(())
    }

    #[test]
    fn committed_transaction_persists_changes() -> Result<(), Box<dyn std::error::Error>> {
        let database = GeneratorDatabase::open()?;
        let connection = database.connection();
        connection.execute("CREATE TABLE transaction_test (id INTEGER)", Parameters::None)?;
        let mut transaction = DatabaseSession::new(connection).transaction()?;
        transaction.execute("INSERT INTO transaction_test VALUES (1)")?;
        transaction.commit()?;
        assert_eq!(count(connection)?, 1);
        Ok(())
    }
}
