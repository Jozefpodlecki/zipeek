use crate::services::tauri_database;
use web_sys::Document;
use zipeek_db::wasm::{PersonalLearningHistory, PracticeOutcomeSummary};

pub async fn commit(document: &Document, session_id: &str, evidence: &str) -> Result<(), String> {
    if tauri_database::available() {
        tauri_database::commit_history(session_id, evidence).await
    } else {
        PersonalLearningHistory::commit(document, session_id, evidence)
            .await
            .map_err(|error| error.to_string())
    }
}

pub async fn summary(document: &Document) -> Result<PracticeOutcomeSummary, String> {
    if tauri_database::available() {
        let rows = tauri_database::history_rows().await?;
        if rows.len() > 100_000 {
            return Err("Too many native Practice sessions for summary".into());
        }
        PracticeOutcomeSummary::from_evidence_rows(
            rows.iter()
                .map(|row| (row.session_id.as_str(), row.evidence.as_str())),
        )
        .map_err(|error| error.to_string())
    } else {
        PersonalLearningHistory::recorded_outcome_summary(document)
            .await
            .map_err(|error| error.to_string())
    }
}

pub async fn count(document: &Document) -> Result<u64, String> {
    if tauri_database::available() {
        Ok(tauri_database::history_rows().await?.len() as u64)
    } else {
        PersonalLearningHistory::recorded_session_count(document)
            .await
            .map_err(|error| error.to_string())
    }
}

pub async fn erase(document: &Document) -> Result<(), String> {
    if tauri_database::available() {
        tauri_database::erase_history().await
    } else {
        PersonalLearningHistory::erase(document)
            .await
            .map_err(|error| error.to_string())
    }
}
