use serde::de::DeserializeOwned;
use serde_json::Value;
use zipeek_db::contracts::{DatabaseInfo, HistoryRow};

use super::tauri::{TauriService, commands};

pub(crate) use super::tauri::commands::PersonalStorageInfo;

pub async fn open() -> Result<DatabaseInfo, String> {
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::DatabaseOpen>(&())
        .await
        .map_err(|error| error.to_string())
}

pub async fn query<T: DeserializeOwned>(sql: &str, parameters: Vec<Value>) -> Result<T, String> {
    let arguments = commands::DatabaseQuery {
        sql: sql.to_owned(),
        parameters,
    };
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .invoke(
            <commands::DatabaseQueryCommand as super::tauri::Command>::NAME,
            &arguments,
        )
        .await
        .map_err(|error| error.to_string())
}

pub async fn commit_history(session_id: &str, evidence: &str) -> Result<(), String> {
    let arguments = commands::HistoryCommit {
        session_id: session_id.to_owned(),
        evidence: evidence.to_owned(),
    };
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::HistoryCommitCommand>(&arguments)
        .await
        .map_err(|error| error.to_string())
}

pub async fn history_rows() -> Result<Vec<HistoryRow>, String> {
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::HistoryRows>(&())
        .await
        .map_err(|error| error.to_string())
}

pub async fn erase_history() -> Result<(), String> {
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::HistoryErase>(&())
        .await
        .map_err(|error| error.to_string())
}

pub async fn reset_personal_data() -> Result<(), String> {
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::ResetPersonalData>(&commands::ResetConfirmation { confirmed: true })
        .await
        .map_err(|error| error.to_string())
}

pub async fn personal_storage_info() -> Result<PersonalStorageInfo, String> {
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::PersonalStorageInformation>(&())
        .await
        .map_err(|error| error.to_string())
}

pub async fn personal_storage_reveal() -> Result<(), String> {
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<commands::PersonalStorageReveal>(&())
        .await
        .map_err(|error| error.to_string())
}
