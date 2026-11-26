use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use serde::Serialize;
use serde_json::Value;
use tauri::{Manager, Runtime};
use zipeek_db::contracts::{DatabaseInfo, HistoryRow};
use zipeek_db::native::{Connection, Environment, Parameters, StorageLocation};

mod reset;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "details", rename_all = "snake_case")]
pub enum DatabaseError {
    WorkerUnavailable,
    TaskFailure(String),
    Operation(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkerUnavailable => formatter.write_str("Native database worker exited"),
            Self::TaskFailure(message) | Self::Operation(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for DatabaseError {}

enum Request {
    Open(mpsc::Sender<Result<DatabaseInfo, String>>),
    Query(String, Vec<Value>, mpsc::Sender<Result<Vec<Value>, String>>),
    CommitHistory(String, String, mpsc::Sender<Result<(), String>>),
    ReadHistory(mpsc::Sender<Result<Vec<HistoryRow>, String>>),
    EraseHistory(mpsc::Sender<Result<(), String>>),
    ResetPersonal(mpsc::Sender<Result<(), String>>),
}

pub struct NativeDatabase {
    tx: mpsc::Sender<Request>,
}

impl NativeDatabase {
    pub fn new<R: Runtime>(app: &tauri::AppHandle<R>) -> Self {
        let resource = app
            .path()
            .resolve(
                "dictionary/hanzi.duckdb",
                tauri::path::BaseDirectory::Resource,
            )
            .ok();
        let personal = app
            .path()
            .app_local_data_dir()
            .ok()
            .map(|dir| dir.join("zipeek-personal.duckdb"));
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("zipeek-native-duckdb".into())
            .spawn(move || worker(rx, resource, personal))
            .expect("database worker thread");
        Self { tx }
    }

    pub async fn open(&self) -> Result<DatabaseInfo, DatabaseError> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Request::Open(tx))
            .map_err(|_| DatabaseError::WorkerUnavailable)?;
        tauri::async_runtime::spawn_blocking(move || rx.recv())
            .await
            .map_err(|error| DatabaseError::TaskFailure(error.to_string()))?
            .map_err(|_| DatabaseError::WorkerUnavailable)?
            .map_err(DatabaseError::Operation)
    }

    pub async fn commit_history(
        &self,
        session_id: String,
        evidence: String,
    ) -> Result<(), DatabaseError> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Request::CommitHistory(session_id, evidence, tx))
            .map_err(|_| DatabaseError::WorkerUnavailable)?;
        tauri::async_runtime::spawn_blocking(move || rx.recv())
            .await
            .map_err(|error| DatabaseError::TaskFailure(error.to_string()))?
            .map_err(|_| DatabaseError::WorkerUnavailable)?
            .map_err(DatabaseError::Operation)
    }

    pub async fn history_rows(&self) -> Result<Vec<HistoryRow>, DatabaseError> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Request::ReadHistory(tx))
            .map_err(|_| DatabaseError::WorkerUnavailable)?;
        tauri::async_runtime::spawn_blocking(move || rx.recv())
            .await
            .map_err(|error| DatabaseError::TaskFailure(error.to_string()))?
            .map_err(|_| DatabaseError::WorkerUnavailable)?
            .map_err(DatabaseError::Operation)
    }

    pub async fn reset_personal_data(&self) -> Result<(), DatabaseError> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Request::ResetPersonal(tx))
            .map_err(|_| DatabaseError::WorkerUnavailable)?;
        tauri::async_runtime::spawn_blocking(move || rx.recv())
            .await
            .map_err(|error| DatabaseError::TaskFailure(error.to_string()))?
            .map_err(|_| DatabaseError::WorkerUnavailable)?
            .map_err(DatabaseError::Operation)
    }

    pub async fn erase_history(&self) -> Result<(), DatabaseError> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Request::EraseHistory(tx))
            .map_err(|_| DatabaseError::WorkerUnavailable)?;
        tauri::async_runtime::spawn_blocking(move || rx.recv())
            .await
            .map_err(|error| DatabaseError::TaskFailure(error.to_string()))?
            .map_err(|_| DatabaseError::WorkerUnavailable)?
            .map_err(DatabaseError::Operation)
    }

    pub async fn query(
        &self,
        sql: String,
        parameters: Vec<Value>,
    ) -> Result<Vec<Value>, DatabaseError> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Request::Query(sql, parameters, tx))
            .map_err(|_| DatabaseError::WorkerUnavailable)?;
        tauri::async_runtime::spawn_blocking(move || rx.recv())
            .await
            .map_err(|error| DatabaseError::TaskFailure(error.to_string()))?
            .map_err(|_| DatabaseError::WorkerUnavailable)?
            .map_err(DatabaseError::Operation)
    }
}

fn extract_storage_version(tags: &str) -> Option<&str> {
    let offset = tags.find("storage_version")?;
    let remainder = &tags[offset + "storage_version".len()..];
    let start = remainder.find('v')?;
    let version = &remainder[start..];
    let len = version
        .bytes()
        .take_while(|byte| byte.is_ascii_digit() || matches!(byte, b'v' | b'.' | b'+'))
        .count();
    (len > 1).then_some(&version[..len])
}

fn dictionary_path(resource: Option<&Path>) -> Result<PathBuf, String> {
    let path = std::env::var_os("ZIPEEK_NATIVE_DICTIONARY")
        .map(PathBuf::from)
        .or_else(|| {
            resource
                .filter(|path| path.is_file())
                .map(Path::to_path_buf)
        })
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries/dictionary/hanzi.duckdb")
        });
    if !path.is_file() {
        return Err(format!(
            "Native dictionary missing: {}. Run cargo make desktop-sidecars after installing the dictionary",
            path.display()
        ));
    }
    Ok(path)
}

struct NativeConnections {
    dictionary: Connection,
    personal: Option<Connection>,
}

fn worker(rx: mpsc::Receiver<Request>, resource: Option<PathBuf>, personal: Option<PathBuf>) {
    let mut connections: Option<NativeConnections> = None;
    for request in rx {
        match request {
            Request::Open(reply) => {
                let result = (|| {
                    if connections.is_none() {
                        let dictionary =
                            open_dictionary_connection(&dictionary_path(resource.as_deref())?)?;
                        let personal = personal
                            .as_deref()
                            .map(open_personal_connection)
                            .transpose()?;
                        connections = Some(NativeConnections {
                            dictionary,
                            personal,
                        });
                    }
                    let dictionary = &connections
                        .as_ref()
                        .expect("connections initialized")
                        .dictionary;
                    let mut rows = execute_json(
                        dictionary,
                        "SELECT version() AS engine, CAST(count(*) AS INTEGER) AS row_count FROM words",
                        &[],
                    )?;
                    let metadata = rows.pop().ok_or("Dictionary metadata is missing")?;
                    let version_rows = execute_json(
                        dictionary,
                        "SELECT CAST(tags AS VARCHAR) AS tags FROM duckdb_databases() WHERE database_name = current_database()",
                        &[],
                    )?;
                    let actual_tags = version_rows
                        .first()
                        .and_then(|row| row["tags"].as_str())
                        .unwrap_or("");
                    let storage_version = extract_storage_version(actual_tags)
                        .ok_or("DuckDB did not report an actual dictionary storage version")?;
                    Ok(DatabaseInfo {
                        engine: metadata["engine"]
                            .as_str()
                            .ok_or("DuckDB version is invalid")?
                            .to_owned(),
                        native_engine: metadata["engine"]
                            .as_str()
                            .ok_or("DuckDB version is invalid")?
                            .to_owned(),
                        storage_compatibility: storage_version.to_owned(),
                        row_count: metadata["row_count"]
                            .as_u64()
                            .ok_or("Dictionary row count is invalid")?
                            as u32,
                    })
                })();
                let _ = reply.send(result);
            }
            Request::Query(sql, params, reply) => {
                let result = connections
                    .as_ref()
                    .ok_or_else(|| "Open native dictionary before querying".to_owned())
                    .and_then(|connections| execute_json(&connections.dictionary, &sql, &params));
                let _ = reply.send(result);
            }
            Request::CommitHistory(id, evidence, reply) => {
                let result = personal_connection(connections.as_ref())
                    .and_then(|connection| commit_history(connection, &id, &evidence));
                let _ = reply.send(result);
            }
            Request::ReadHistory(reply) => {
                let result = personal_connection(connections.as_ref())
                    .and_then(|connection| execute_json(connection,
                        "SELECT session_id, evidence_json AS evidence FROM personal.practice_sessions ORDER BY session_id LIMIT 100001", &[]))
                    .and_then(|rows| rows.into_iter().map(|row| serde_json::from_value(row).map_err(|error| error.to_string())).collect());
                let _ = reply.send(result);
            }
            Request::ResetPersonal(reply) => {
                let result = reset::reset_personal_database(&mut connections, personal.as_deref());
                let _ = reply.send(result);
            }
            Request::EraseHistory(reply) => {
                let result = personal_connection(connections.as_ref()).and_then(|connection| {
                    connection
                        .execute("DELETE FROM personal.practice_sessions", Parameters::None)
                        .map(|_| ())
                        .map_err(|error| error.to_string())
                });
                let _ = reply.send(result);
            }
        }
    }
}

fn personal_connection(connections: Option<&NativeConnections>) -> Result<&Connection, String> {
    connections
        .ok_or("Open native database before accessing history")?
        .personal
        .as_ref()
        .ok_or_else(|| "Native personal storage is unavailable".to_owned())
}

fn memory_connection() -> Result<Connection, String> {
    let environment = Environment::new().map_err(|error| error.to_string())?;
    let database = environment
        .open(StorageLocation::InMemory)
        .map_err(|error| error.to_string())?;
    database.connect().map_err(|error| error.to_string())
}

fn open_dictionary_connection(path: &Path) -> Result<Connection, String> {
    let connection = memory_connection()?;
    let escaped = path.to_string_lossy().replace('\'', "''");
    connection
        .execute(
            &format!("ATTACH '{escaped}' AS dictionary (READ_ONLY)"),
            Parameters::None,
        )
        .map_err(|error| error.to_string())?;
    connection
        .execute("USE dictionary", Parameters::None)
        .map_err(|error| error.to_string())?;
    Ok(connection)
}

fn open_personal_connection(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let connection = memory_connection()?;
    let escaped = path.to_string_lossy().replace('\'', "''");
    connection
        .execute(&format!("ATTACH '{escaped}' AS personal"), Parameters::None)
        .map_err(|error| error.to_string())?;
    connection.execute("CREATE TABLE IF NOT EXISTS personal.practice_sessions (session_id VARCHAR PRIMARY KEY, evidence_json VARCHAR NOT NULL)", Parameters::None)
        .map_err(|error| error.to_string())?;
    Ok(connection)
}

fn commit_history(connection: &Connection, id: &str, evidence: &str) -> Result<(), String> {
    if id.len() != 36
        || !id.bytes().enumerate().all(|(i, byte)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
        || evidence.is_empty()
        || evidence.len() > 512 * 1024
    {
        return Err("Invalid learning-history session or evidence size".into());
    }
    let document: Value = serde_json::from_str(evidence).map_err(|error| error.to_string())?;
    if document["schema_version"] != 2
        || document["session_id"] != id
        || document["steps"].as_array().map_or(true, Vec::is_empty)
    {
        return Err("Invalid learning-history evidence".into());
    }
    let existing = execute_json(
        connection,
        "SELECT evidence_json AS evidence FROM personal.practice_sessions WHERE session_id = ?",
        &[Value::String(id.into())],
    )?;
    if let Some(row) = existing.first() {
        return if row["evidence"] == evidence {
            Ok(())
        } else {
            Err("Conflicting existing session evidence".into())
        };
    }
    let sql = bind_parameters(
        "INSERT INTO personal.practice_sessions (session_id, evidence_json) VALUES (?, ?)",
        &[Value::String(id.into()), Value::String(evidence.into())],
    )?;
    connection
        .execute(&sql, Parameters::None)
        .map_err(|error| error.to_string())?;
    let stored = execute_json(
        connection,
        "SELECT evidence_json AS evidence FROM personal.practice_sessions WHERE session_id = ?",
        &[Value::String(id.into())],
    )?;
    if stored.len() != 1 || stored[0]["evidence"] != evidence {
        return Err("Committed learning-history evidence failed readback".into());
    }
    Ok(())
}

fn sql_literal(value: &Value) -> Result<String, String> {
    Ok(match value {
        Value::Null => "NULL".into(),
        Value::Bool(value) => (if *value { "TRUE" } else { "FALSE" }).into(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => format!("'{}'", value.replace('\'', "''")),
        _ => return Err("Dictionary parameter must be a scalar".into()),
    })
}

fn bind_parameters(sql: &str, parameters: &[Value]) -> Result<String, String> {
    let mut output = String::with_capacity(sql.len());
    let mut quoted = false;
    let mut chars = sql.chars().peekable();
    let mut index = 0;
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            output.push(ch);
            if quoted && chars.peek() == Some(&'\'') {
                output.push(chars.next().expect("quoted escape"));
            } else {
                quoted = !quoted;
            }
        } else if ch == '?' && !quoted {
            let value = parameters
                .get(index)
                .ok_or("Not enough dictionary query parameters")?;
            output.push_str(&sql_literal(value)?);
            index += 1;
        } else {
            output.push(ch);
        }
    }
    if quoted || index != parameters.len() {
        return Err("Invalid dictionary SQL or parameter count".into());
    }
    Ok(output)
}

fn read_only_query(sql: &str) -> Result<(), String> {
    let sql = sql.trim();
    let lower = sql.to_ascii_lowercase();
    if !(lower.starts_with("select ")
        || lower.starts_with("select\n")
        || lower.starts_with("with ")
        || lower.starts_with("with\n"))
        || sql.contains(';')
        || lower.contains("read_csv")
        || lower.contains("read_parquet")
        || lower.contains("read_text")
        || lower.contains("read_json")
        || lower.contains("httpfs")
        || lower.contains("pragma_") && !lower.contains("pragma_database_size()")
    {
        return Err("Only read-only dictionary SELECT queries are accepted".into());
    }
    Ok(())
}

fn execute_json(
    connection: &Connection,
    sql: &str,
    parameters: &[Value],
) -> Result<Vec<Value>, String> {
    read_only_query(sql)?;
    let bound = bind_parameters(sql, parameters)?;
    let query = format!("SELECT to_json(row_value)::VARCHAR FROM ({bound}) AS row_value");
    let mut result = connection
        .query(&query, Parameters::None)
        .map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    while let Some(chunk) = result.next_chunk().map_err(|error| error.to_string())? {
        let vector = chunk
            .get_vector_at::<String>(0)
            .map_err(|error| error.to_string())?;
        let reader = vector.reader().map_err(|error| error.to_string())?;
        for item in reader.iter() {
            let json = item.ok_or("Null dictionary JSON row")?;
            rows.push(serde_json::from_str(json).map_err(|error| error.to_string())?);
            if rows.len() > 25_000 {
                return Err("Dictionary result exceeded 25,000 rows".into());
            }
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_actual_dictionary_storage_tags() {
        assert_eq!(
            extract_storage_version("{storage_version=v2.0.0+}"),
            Some("v2.0.0+")
        );
        assert_eq!(
            extract_storage_version("{'storage_version': 'v2.0.0+'}"),
            Some("v2.0.0+")
        );
        assert_eq!(extract_storage_version("{}"), None);
    }

    #[test]
    fn parameters_escape_quotes_and_reject_mismatch() {
        assert_eq!(
            bind_parameters(
                "SELECT * FROM words WHERE simplified = ?",
                &[Value::String("a'b".into())]
            )
            .unwrap(),
            "SELECT * FROM words WHERE simplified = 'a''b'"
        );
        assert!(bind_parameters("SELECT ?", &[]).is_err());
        assert!(bind_parameters("SELECT 1", &[Value::Null]).is_err());
    }
    #[test]
    fn json_rows_round_trip_through_native_duckdb() {
        let environment = Environment::new().unwrap();
        let database = environment.open(StorageLocation::InMemory).unwrap();
        let connection = database.connect().unwrap();
        let rows = execute_json(
            &connection,
            "SELECT ? AS simplified, 42 AS id, NULL::VARCHAR AS english",
            &[Value::String("学习".into())],
        )
        .unwrap();
        assert_eq!(
            rows,
            vec![serde_json::json!({"simplified":"学习", "id":42, "english":null})]
        );
    }

    #[test]
    fn dictionary_query_connection_cannot_access_personal_history() {
        let dictionary = memory_connection().expect("dictionary connection");
        let personal = memory_connection().expect("personal connection");
        personal
            .execute(
                "CREATE TABLE practice_sessions (session_id VARCHAR, evidence_json VARCHAR)",
                Parameters::None,
            )
            .expect("personal table");
        personal
            .execute(
                "INSERT INTO practice_sessions VALUES ('private', 'evidence')",
                Parameters::None,
            )
            .expect("private entry");
        assert!(execute_json(&dictionary, "SELECT * FROM practice_sessions", &[],).is_err());
        assert!(
            execute_json(&dictionary, "SELECT * FROM personal.practice_sessions", &[],).is_err()
        );
        let rows = execute_json(&personal, "SELECT session_id FROM practice_sessions", &[])
            .expect("personal query");
        assert_eq!(rows, vec![serde_json::json!({"session_id": "private"})]);
    }

    #[test]
    fn personal_history_database_survives_close_and_reopen() {
        let filename = format!(
            "zipeek-native-test-{}-{}.duckdb",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos(),
        );
        let path = std::env::temp_dir().join(filename);
        {
            let personal = open_personal_connection(&path).expect("create personal database");
            personal
                .execute(
                    "INSERT INTO personal.practice_sessions VALUES ('test-session', 'persisted')",
                    Parameters::None,
                )
                .expect("store test session");
        }
        {
            let personal = open_personal_connection(&path).expect("reopen personal database");
            let rows = execute_json(
                &personal,
                "SELECT evidence_json FROM personal.practice_sessions WHERE session_id = 'test-session'",
                &[],
            ).expect("read persisted session");
            assert_eq!(
                rows,
                vec![serde_json::json!({"evidence_json": "persisted"})]
            );
        }
        std::fs::remove_file(path).expect("remove temporary personal database");
    }

    #[test]
    fn rejects_non_select_sql() {
        assert!(read_only_query("DELETE FROM words").is_err());
        assert!(read_only_query("SELECT 1; DROP TABLE words").is_err());
        assert!(read_only_query("SELECT * FROM read_csv('/secret')").is_err());
        assert!(read_only_query("SELECT * FROM words LIMIT 10").is_ok());
    }
}

#[cfg(test)]
mod ipc_error_tests {
    use super::*;

    #[test]
    fn database_errors_serialize_as_structured_rejections() {
        assert_eq!(
            serde_json::to_value(DatabaseError::WorkerUnavailable).expect("serialize"),
            serde_json::json!({"kind": "worker_unavailable"}),
        );
        assert_eq!(
            serde_json::to_value(DatabaseError::Operation("invalid query".into()))
                .expect("serialize"),
            serde_json::json!({"kind": "operation", "details": "invalid query"}),
        );
    }
}
