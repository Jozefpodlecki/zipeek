use serde_json::{Map, Value};
use web_sys::Storage;

use super::backup::{self, Backup, Category};
use super::error::PersonalDataError;

#[derive(Clone)]
pub struct PersonalStorage(Storage);

impl PersonalStorage {
    pub fn new(storage: Storage) -> Self {
        Self(storage)
    }

    pub fn export(&self, exported_at: String) -> Result<String, PersonalDataError> {
        let mut data = Map::new();
        for category in Category::ALL {
            let Some(raw) = self
                .0
                .get_item(category.key())
                .map_err(|e| PersonalDataError::Storage(format!("{e:?}")))?
            else {
                continue;
            };
            let value: Value = serde_json::from_str(&raw).map_err(|e| {
                PersonalDataError::InvalidBackup(format!(
                    "Existing {} data is invalid: {e}",
                    category.name()
                ))
            })?;
            backup::validate(category, &value)?;
            data.insert(category.name().to_owned(), value);
        }
        let exported = serde_json::to_string_pretty(&Backup {
            format: backup::FORMAT.to_owned(),
            version: backup::VERSION,
            exported_at: Some(exported_at),
            data,
        })
        .map_err(|e| PersonalDataError::InvalidBackup(e.to_string()))?;
        validate_export_size(exported)
    }

    pub fn import(&self, raw: &str) -> Result<String, PersonalDataError> {
        let backup = backup::parse(raw)?;
        let changes = backup
            .data
            .into_iter()
            .map(|(name, value)| {
                let category = Category::try_from(name.as_str())?;
                let json = serde_json::to_string(&value)
                    .map_err(|e| PersonalDataError::InvalidBackup(e.to_string()))?;
                Ok((category.key(), Some(json)))
            })
            .collect::<Result<Vec<_>, PersonalDataError>>()?;
        self.commit(&changes)?;
        Ok(format!(
            "Restored {} learning-data categories. Reload to refresh open views.",
            changes.len()
        ))
    }

    pub fn delete(&self, categories: &[Category]) -> Result<String, PersonalDataError> {
        if categories.is_empty() || {
            let mut unique = std::collections::HashSet::new();
            categories.iter().any(|category| !unique.insert(*category))
        } {
            return Err(PersonalDataError::InvalidBackup(
                "Choose distinct categories to delete".into(),
            ));
        }
        let changes = categories
            .iter()
            .map(|category| (category.key(), None))
            .collect::<Vec<_>>();
        self.commit(&changes)?;
        Ok(format!(
            "Deleted {} learning-data categories. Reload to refresh open views.",
            categories.len()
        ))
    }

    fn commit(&self, changes: &[(&str, Option<String>)]) -> Result<(), PersonalDataError> {
        let previous = changes
            .iter()
            .map(|(key, _)| {
                self.0
                    .get_item(key)
                    .map(|value| (*key, value))
                    .map_err(|error| {
                        PersonalDataError::Storage(format!(
                            "Cannot read {key} before mutation: {error:?}"
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (index, (key, value)) in changes.iter().enumerate() {
            let result = match value {
                Some(value) => self.0.set_item(key, value),
                None => self.0.remove_item(key),
            };
            if let Err(error) = result {
                let mut failures = Vec::new();
                for (old_key, old_value) in previous[..=index].iter().rev() {
                    let restored = match old_value {
                        Some(value) => self.0.set_item(old_key, value),
                        None => self.0.remove_item(old_key),
                    };
                    if let Err(rollback_error) = restored {
                        failures.push(format!("{old_key}: {rollback_error:?}"));
                    }
                }
                return Err(PersonalDataError::PartialRollback {
                    operation: format!("{key}: {error:?}"),
                    rollback: if failures.is_empty() {
                        "succeeded".into()
                    } else {
                        format!("failed for {}", failures.join(", "))
                    },
                });
            }
        }
        Ok(())
    }
}

fn validate_export_size(encoded: String) -> Result<String, PersonalDataError> {
    if encoded.len() > backup::MAX_IMPORT_BYTES {
        return Err(PersonalDataError::InvalidBackup(
            "Export exceeds the 2 MB backup import limit".into(),
        ));
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_backup_must_fit_its_own_import_limit() {
        assert!(validate_export_size("small".to_owned()).is_ok());
        assert!(validate_export_size("x".repeat(backup::MAX_IMPORT_BYTES + 1)).is_err());
    }
}
