use serde_json::Value;
use tauri::{State, command};

use super::error::CommandError;
use crate::database::{DatabaseError, NativeDatabase};
use zipeek_db::contracts::{DatabaseInfo, HistoryRow};

#[command]
pub async fn database_open(state: State<'_, NativeDatabase>) -> Result<DatabaseInfo, CommandError> {
    Ok(state.open().await?)
}

#[command]
pub async fn database_query(
    sql: String,
    parameters: Vec<Value>,
    state: State<'_, NativeDatabase>,
) -> Result<Vec<Value>, CommandError> {
    Ok(state.query(sql, parameters).await?)
}

#[command]
pub async fn history_commit(
    session_id: String,
    evidence: String,
    state: State<'_, NativeDatabase>,
) -> Result<(), CommandError> {
    Ok(state.commit_history(session_id, evidence).await?)
}

#[command]
pub async fn history_rows(
    state: State<'_, NativeDatabase>,
) -> Result<Vec<HistoryRow>, CommandError> {
    Ok(state.history_rows().await?)
}

#[command]
pub async fn history_erase(state: State<'_, NativeDatabase>) -> Result<(), CommandError> {
    Ok(state.erase_history().await?)
}

#[command]
pub async fn personal_data_reset(
    confirmed: bool,
    state: State<'_, NativeDatabase>,
) -> Result<(), CommandError> {
    if !confirmed {
        return Err(DatabaseError::Operation("Reset must be explicitly confirmed".into()).into());
    }
    Ok(state.reset_personal_data().await?)
}

#[command]
pub fn personal_storage_info(
    state: State<'_, NativeDatabase>,
) -> Result<crate::database::PersonalStorageInfo, CommandError> {
    Ok(state.personal_storage_info()?)
}

#[command]
pub fn personal_storage_reveal(state: State<'_, NativeDatabase>) -> Result<(), CommandError> {
    Ok(state.reveal_personal_storage()?)
}
