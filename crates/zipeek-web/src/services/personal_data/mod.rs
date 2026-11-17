mod backup;
mod error;
mod files;
mod opfs;
mod storage;

use web_sys::{Document, Navigator, Storage};

pub use backup::Category as PersonalCategory;
pub use error::PersonalDataError;
use storage::PersonalStorage;

#[derive(Clone)]
pub struct PersonalDataService {
    storage: PersonalStorage,
    document: Document,
    navigator: Navigator,
}

impl PersonalDataService {
    pub fn new(storage: Storage, document: Document, navigator: Navigator) -> Self {
        Self {
            storage: PersonalStorage::new(storage),
            document,
            navigator,
        }
    }

    pub fn export_backup(&self) -> Result<String, PersonalDataError> {
        let timestamp = js_sys::Date::new_0()
            .to_iso_string()
            .as_string()
            .unwrap_or_default();
        self.storage.export(timestamp)
    }

    pub fn preview_backup(&self, raw: &str) -> Result<String, PersonalDataError> {
        backup::preview(raw)
    }

    pub fn import_backup(&self, raw: &str) -> Result<String, PersonalDataError> {
        self.storage.import(raw)
    }

    pub fn download_backup(&self, raw: &str) -> Result<(), PersonalDataError> {
        files::download(&self.document, raw)
    }

    pub async fn pick_backup_file(&self) -> Result<String, PersonalDataError> {
        files::pick(&self.document).await
    }

    pub async fn storage_status(&self) -> Result<String, PersonalDataError> {
        opfs::status(&self.navigator).await
    }

    pub async fn save_snapshot(&self) -> Result<String, PersonalDataError> {
        let raw = self.export_backup()?;
        opfs::save(&self.navigator, &raw).await
    }

    pub async fn read_snapshot(&self) -> Result<String, PersonalDataError> {
        opfs::read(&self.navigator).await
    }

    pub async fn delete_snapshot(&self) -> Result<String, PersonalDataError> {
        opfs::delete(&self.navigator).await
    }

    pub async fn probe_opfs(&self) -> Result<String, PersonalDataError> {
        opfs::probe(&self.navigator).await
    }

    pub async fn delete_selected(&self, names: &[String]) -> Result<String, PersonalDeletionError> {
        let categories = validate_deletion(names)?;
        let description = self
            .storage
            .delete(&categories)
            .map_err(|error| PersonalDeletionError::Categories(error.to_string()))?;
        self.delete_snapshot()
            .await
            .map_err(|error| PersonalDeletionError::SnapshotAfterCategories(error.to_string()))?;
        Ok(format!("{description} Private snapshot removed."))
    }
}

pub fn error_message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

pub async fn probe_personal_duckdb(document: &Document) -> Result<String, String> {
    if crate::services::tauri_database::available() {
        let rows = crate::services::tauri_database::history_rows().await?;
        return Ok(format!(
            "Native DuckDB learning-history access succeeded ({} sessions)",
            rows.len()
        ));
    }
    zipeek_db::wasm::PersonalDbProbe::run(document)
        .await
        .map(|report| report.message())
        .map_err(|error| error.to_string())
}

#[derive(Debug, PartialEq, Eq)]
pub enum PersonalDeletionError {
    InvalidSelection(String),
    Categories(String),
    SnapshotAfterCategories(String),
}

impl std::fmt::Display for PersonalDeletionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSelection(message) => {
                write!(formatter, "Deletion selection is invalid: {message}")
            }
            Self::Categories(message) => {
                write!(formatter, "No categories confirmed deleted: {message}")
            }
            Self::SnapshotAfterCategories(message) => write!(
                formatter,
                "Categories deleted, but private snapshot deletion failed: {message}. Retry snapshot deletion."
            ),
        }
    }
}

impl std::error::Error for PersonalDeletionError {}

pub fn validate_deletion(names: &[String]) -> Result<Vec<PersonalCategory>, PersonalDeletionError> {
    if names.is_empty() {
        return Err(PersonalDeletionError::InvalidSelection(
            "Choose at least one category".into(),
        ));
    }
    let mut categories = Vec::with_capacity(names.len());
    for name in names {
        let category = PersonalCategory::try_from(name.as_str())
            .map_err(|error| PersonalDeletionError::InvalidSelection(error.to_string()))?;
        if categories.contains(&category) {
            return Err(PersonalDeletionError::InvalidSelection(
                "Duplicate category".into(),
            ));
        }
        categories.push(category);
    }
    Ok(categories)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletion_rejects_duplicate_and_invalid_categories() {
        assert!(validate_deletion(&[]).is_err());
        assert!(validate_deletion(&["word_sets".into(), "word_sets".into()]).is_err());
        assert!(validate_deletion(&["unexpected".into()]).is_err());
        assert_eq!(
            validate_deletion(&["word_sets".into(), "exclusions".into()])
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn snapshot_failure_does_not_claim_total_failure() {
        let error = PersonalDeletionError::SnapshotAfterCategories("offline".into());
        assert!(error.to_string().contains("Categories deleted"));
    }
}
