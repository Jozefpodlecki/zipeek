use serde_json::Value;
use tauri::State;

use crate::database::{DatabaseError, DatabaseInfo, NativeDatabase};

#[tauri::command]
pub async fn database_open(
    state: State<'_, NativeDatabase>,
) -> Result<DatabaseInfo, DatabaseError> {
    state.open().await
}

#[tauri::command]
pub async fn database_query(
    sql: String,
    parameters: Vec<Value>,
    state: State<'_, NativeDatabase>,
) -> Result<Vec<Value>, DatabaseError> {
    state.query(sql, parameters).await
}

#[tauri::command]
pub async fn history_commit(
    session_id: String,
    evidence: String,
    state: State<'_, NativeDatabase>,
) -> Result<(), DatabaseError> {
    state.commit_history(session_id, evidence).await
}

#[tauri::command]
pub async fn history_rows(state: State<'_, NativeDatabase>) -> Result<Vec<Value>, DatabaseError> {
    state.history_rows().await
}

#[tauri::command]
pub async fn history_erase(state: State<'_, NativeDatabase>) -> Result<(), DatabaseError> {
    state.erase_history().await
}
