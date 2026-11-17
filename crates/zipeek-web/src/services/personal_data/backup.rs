use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::error::PersonalDataError;
use crate::services::word_sets;

pub const FORMAT: &str = "zipeek-personal-backup";
pub const VERSION: u8 = 1;
pub const MAX_IMPORT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    WordSets,
    Exclusions,
}

impl Category {
    pub const ALL: [Category; 2] = [Self::WordSets, Self::Exclusions];

    pub const fn key(self) -> &'static str {
        match self {
            Self::WordSets => "zipeek.user_word_sets.v1",
            Self::Exclusions => "zipeek.excluded_words",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::WordSets => "word_sets",
            Self::Exclusions => "exclusions",
        }
    }
}

impl TryFrom<&str> for Category {
    type Error = PersonalDataError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == value)
            .ok_or_else(|| PersonalDataError::UnknownCategory(value.to_owned()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Backup {
    pub format: String,
    pub version: u8,
    #[serde(default)]
    pub exported_at: Option<String>,
    pub data: Map<String, Value>,
}

pub fn validate(category: Category, value: &Value) -> Result<(), PersonalDataError> {
    let invalid = |message: &str| PersonalDataError::InvalidBackup(message.to_owned());
    match category {
        Category::WordSets => {
            let encoded =
                serde_json::to_string(value).map_err(|error| invalid(&error.to_string()))?;
            word_sets::decode(&encoded).map_err(|error| invalid(&error))?;
        }
        Category::Exclusions => {
            let entries = value
                .as_array()
                .ok_or_else(|| invalid("Exclusions must be an array"))?;
            if entries.len() > 100_000 || entries.iter().any(|entry| {
                !matches!(entry.as_i64(), Some(id) if (1..=i32::MAX as i64).contains(&id))
            }) {
                return Err(invalid("Invalid excluded word ID"));
            }
        }
    }
    Ok(())
}

pub fn parse(raw: &str) -> Result<Backup, PersonalDataError> {
    if raw.len() > MAX_IMPORT_BYTES {
        return Err(PersonalDataError::InvalidBackup(
            "Backup exceeds the 2 MB limit".into(),
        ));
    }
    let backup: Backup = serde_json::from_str(raw)
        .map_err(|error| PersonalDataError::InvalidBackup(error.to_string()))?;
    if backup.format != FORMAT || backup.version != VERSION {
        return Err(PersonalDataError::InvalidBackup(
            "Unsupported backup format or version".into(),
        ));
    }
    for (name, value) in &backup.data {
        validate(Category::try_from(name.as_str())?, value)?;
    }
    Ok(backup)
}

pub fn preview(raw: &str) -> Result<String, PersonalDataError> {
    let backup = parse(raw)?;
    let groups = backup
        .data
        .iter()
        .map(|(name, value)| {
            let count = value.as_array().map_or(0, Vec::len);
            format!("{}: {count}", name.replace('_', " "))
        })
        .collect::<Vec<_>>();
    Ok(if groups.is_empty() {
        "Empty backup".to_owned()
    } else {
        groups.join(" · ")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_backup_categories() {
        assert!(
            parse(r#"{"format":"zipeek-personal-backup","version":1,"data":{"secret":[]}}"#)
                .is_err()
        );
    }

    #[test]
    fn accepts_empty_valid_backup() {
        assert_eq!(
            preview(r#"{"format":"zipeek-personal-backup","version":1,"data":{}}"#).unwrap(),
            "Empty backup"
        );
    }

    #[test]
    fn rejects_unverified_course_progress_category() {
        assert!(
            parse(r#"{"format":"zipeek-personal-backup","version":1,"data":{"course_progress":["tones"]}}"#)
                .is_err()
        );
    }

    #[test]
    fn rejects_invalid_exclusions() {
        assert!(validate(Category::Exclusions, &serde_json::json!([0])).is_err());
    }
}
