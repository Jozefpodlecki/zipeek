use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use zipeek_db::contracts::{DatabaseInfo, HistoryRow};

pub(crate) trait Command {
    type Args: Serialize;
    type Output: DeserializeOwned;
    const NAME: &'static str;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseQuery {
    pub sql: String,
    pub parameters: Vec<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HistoryCommit {
    pub session_id: String,
    pub evidence: String,
}

#[derive(Serialize)]
pub(crate) struct Service {
    pub service: String,
}

#[derive(Serialize)]
pub(crate) struct ResetConfirmation {
    pub confirmed: bool,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PersonalStorageInfo {
    pub path: String,
    pub exists: bool,
    pub size_bytes: Option<u64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServiceStatus {
    pub service: String,
    pub label: String,
    pub phase: String,
    pub running: bool,
    pub healthy: bool,
    pub owned: bool,
    pub pid: Option<u32>,
    pub message: String,
}

macro_rules! command {
    ($name:ident, $literal:literal, $args:ty, $response:ty) => {
        pub(crate) struct $name;
        impl Command for $name {
            type Args = $args;
            type Output = $response;
            const NAME: &'static str = $literal;
        }
    };
}

command!(AiComplete, "ai_complete", Value, String);
command!(AiModels, "ai_models", Value, Vec<String>);
command!(ImageHealth, "image_health", Value, String);
command!(ImageGenerate, "image_generate", Value, Value);
command!(DatabaseOpen, "database_open", (), DatabaseInfo);
command!(DatabaseQueryCommand, "database_query", DatabaseQuery, Value);
command!(HistoryCommitCommand, "history_commit", HistoryCommit, ());
command!(HistoryRows, "history_rows", (), Vec<HistoryRow>);
command!(HistoryErase, "history_erase", (), ());
command!(
    ResetPersonalData,
    "personal_data_reset",
    ResetConfirmation,
    ()
);
command!(
    PersonalStorageInformation,
    "personal_storage_info",
    (),
    PersonalStorageInfo
);
command!(PersonalStorageReveal, "personal_storage_reveal", (), ());
command!(StartService, "start_service", Service, ServiceStatus);
command!(StopService, "stop_service", Service, ServiceStatus);
command!(
    ServiceStatusCommand,
    "service_status",
    Service,
    ServiceStatus
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_command_names_match_desktop_contracts() {
        assert_eq!(AiComplete::NAME, "ai_complete");
        assert_eq!(AiModels::NAME, "ai_models");
        assert_eq!(ImageHealth::NAME, "image_health");
        assert_eq!(ImageGenerate::NAME, "image_generate");
        assert_eq!(DatabaseOpen::NAME, "database_open");
        assert_eq!(DatabaseQueryCommand::NAME, "database_query");
        assert_eq!(HistoryCommitCommand::NAME, "history_commit");
        assert_eq!(HistoryRows::NAME, "history_rows");
        assert_eq!(HistoryErase::NAME, "history_erase");
        assert_eq!(ResetPersonalData::NAME, "personal_data_reset");
        assert_eq!(PersonalStorageInformation::NAME, "personal_storage_info");
        assert_eq!(PersonalStorageReveal::NAME, "personal_storage_reveal");
        assert_eq!(StartService::NAME, "start_service");
        assert_eq!(StopService::NAME, "stop_service");
        assert_eq!(ServiceStatusCommand::NAME, "service_status");
    }

    #[test]
    fn history_arguments_use_tauri_camel_case() {
        let args = HistoryCommit {
            session_id: "session-1".into(),
            evidence: "assessed".into(),
        };
        assert_eq!(
            serde_json::to_value(args).expect("request"),
            serde_json::json!({"sessionId":"session-1","evidence":"assessed"})
        );
    }

    #[test]
    fn database_query_arguments_preserve_arrays() {
        let args = DatabaseQuery {
            sql: "select ?".into(),
            parameters: vec![serde_json::json!("朋友")],
        };
        assert_eq!(
            serde_json::to_value(args).expect("request"),
            serde_json::json!({"sql":"select ?","parameters":["朋友"]})
        );
    }

    #[test]
    fn personal_storage_response_is_camel_case() {
        let info: PersonalStorageInfo = serde_json::from_value(serde_json::json!({
            "path":"C:/data/zipeek-personal.duckdb", "exists":true, "sizeBytes":4096
        }))
        .expect("response");
        assert_eq!(info.size_bytes, Some(4096));
    }
}
