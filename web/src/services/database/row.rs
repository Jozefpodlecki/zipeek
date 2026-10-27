use serde::Deserialize;
use zipeek_core::{Word, WordId};

use super::DbError;

#[derive(Deserialize)]
pub(super) struct WordRow {
    id: i32,
    simplified: String,
    traditional: Option<String>,
    pinyin: String,
    pinyin_numeric: Option<String>,
    english: Option<String>,
    part_of_speech: Option<String>,
    hsk2_level: Option<i32>,
    hsk3_level: Option<i32>,
    frequency: Option<i32>,
}

impl TryFrom<WordRow> for Word {
    type Error = DbError;

    fn try_from(row: WordRow) -> Result<Self, DbError> {
        let invalid = |message: String| DbError::InvalidRow {
            id: row.id,
            message,
        };
        let id = WordId::try_from(i64::from(row.id)).map_err(|error| invalid(error.to_string()))?;
        let hsk2_level = row
            .hsk2_level
            .map(TryInto::try_into)
            .transpose()
            .map_err(|error: zipeek_core::HskLevelError| invalid(error.to_string()))?;
        let hsk3_level = row
            .hsk3_level
            .map(TryInto::try_into)
            .transpose()
            .map_err(|error: zipeek_core::HskLevelError| invalid(error.to_string()))?;
        if row.simplified.trim().is_empty()
            || row.pinyin.trim().is_empty()
            || hsk2_level.is_none() && hsk3_level.is_none()
        {
            return Err(invalid("blank label or missing HSK membership".to_owned()));
        }
        let part_of_speech = row
            .part_of_speech
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter(|token| !token.is_empty())
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error: zipeek_core::PartOfSpeechError| invalid(error.to_string()))?;
        Ok(Word {
            id,
            simplified: row.simplified,
            traditional: row.traditional,
            pinyin: row.pinyin,
            pinyin_numeric: row.pinyin_numeric,
            english: row.english,
            part_of_speech,
            hsk2_level,
            hsk3_level,
            frequency: row.frequency,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> serde_json::Value {
        serde_json::from_str(include_str!("../../../tests/fixtures/word-row.json")).unwrap()
    }

    #[test]
    fn typed_rows_preserve_source_labels_glosses_and_null_metadata() {
        let word = Word::try_from(serde_json::from_value::<WordRow>(fixture()).unwrap()).unwrap();
        assert_eq!(word.pinyin, "xué xí");
        assert_eq!(word.english.as_deref(), Some("to study|to learn"));
        assert_eq!(word.hsk3_level, Some(zipeek_core::Hsk3Level::SevenToNine));
        assert_eq!((word.hsk2_level, word.frequency), (None, None));
        assert_eq!(
            word.part_of_speech,
            [
                zipeek_core::PartOfSpeech::Noun,
                zipeek_core::PartOfSpeech::Verb
            ]
        );
    }

    #[test]
    fn corrupt_ids_levels_labels_and_pos_are_actionable_errors() {
        for (field, value) in [
            ("id", serde_json::json!(0)),
            ("hsk3_level", serde_json::json!(9)),
            ("pinyin", serde_json::json!(" ")),
            ("part_of_speech", serde_json::json!("noun,unknown")),
        ] {
            let mut row = fixture();
            row[field] = value;
            assert!(matches!(
                Word::try_from(serde_json::from_value::<WordRow>(row).unwrap()),
                Err(DbError::InvalidRow { .. })
            ));
        }
    }
}
