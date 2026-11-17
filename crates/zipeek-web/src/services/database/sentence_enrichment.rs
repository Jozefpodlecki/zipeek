use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use zipeek_ai_protocol::sentence_analysis::VocabularyObservation;

use super::{Database, DbError};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SentenceDictionaryEntry {
    pub id: i32,
    pub simplified: String,
    pub traditional: Option<String>,
    pub pinyin: String,
    pub english: Option<String>,
    pub hsk2_level: Option<i32>,
    pub hsk3_level: Option<i32>,
    pub frequency: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedVocabulary {
    pub observation: VocabularyObservation,
    pub dictionary: Option<SentenceDictionaryEntry>,
    pub alternatives: usize,
}

fn normalized_pinyin(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn reconcile_vocabulary(
    observations: &[VocabularyObservation],
    entries: Vec<SentenceDictionaryEntry>,
) -> Vec<VerifiedVocabulary> {
    let mut grouped: BTreeMap<String, Vec<SentenceDictionaryEntry>> = BTreeMap::new();
    for entry in entries {
        grouped
            .entry(entry.simplified.clone())
            .or_default()
            .push(entry.clone());
        if let Some(traditional) = &entry.traditional {
            if traditional != &entry.simplified {
                grouped.entry(traditional.clone()).or_default().push(entry);
            }
        }
    }
    observations
        .iter()
        .map(|observation| {
            let mut candidates = grouped.remove(&observation.hanzi).unwrap_or_default();
            let reported = normalized_pinyin(&observation.pinyin);
            candidates.sort_by_key(|entry| {
                (
                    normalized_pinyin(&entry.pinyin) != reported,
                    entry.frequency.is_none(),
                    entry.frequency.unwrap_or(i32::MAX),
                    entry.id,
                )
            });
            let alternatives = candidates.len().saturating_sub(1);
            VerifiedVocabulary {
                observation: observation.clone(),
                dictionary: candidates.into_iter().next(),
                alternatives,
            }
        })
        .collect()
}

impl Database {
    pub async fn verify_sentence_vocabulary(
        &self,
        observations: &[VocabularyObservation],
    ) -> Result<Vec<VerifiedVocabulary>, DbError> {
        let distinct: BTreeSet<&str> = observations
            .iter()
            .map(|item| item.hanzi.as_str())
            .collect();
        if distinct.is_empty() {
            return Ok(reconcile_vocabulary(observations, vec![]));
        }
        let placeholders = vec!["?"; distinct.len()].join(",");
        let sql = format!(
            r#"SELECT
    id,
    simplified,
    traditional,
    pinyin,
    english,
    hsk2_level,
    hsk3_level,
    frequency
FROM words
WHERE simplified IN ({placeholders})
    OR traditional IN ({placeholders})
ORDER BY id ASC"#
        );
        let values: Vec<_> = distinct.iter().map(|value| json!(value)).collect();
        let mut parameters = values.clone();
        parameters.extend(values);
        let rows: Vec<SentenceDictionaryEntry> = self.query(&sql, parameters).await?;
        Ok(reconcile_vocabulary(observations, rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(id: i64, text: &str, pinyin: &str, frequency: Option<i32>) -> SentenceDictionaryEntry {
        SentenceDictionaryEntry {
            id: id as i32,
            simplified: text.to_owned(),
            traditional: None,
            pinyin: pinyin.to_owned(),
            english: Some("dictionary meaning".to_owned()),
            hsk2_level: None,
            hsk3_level: Some(1),
            frequency,
        }
    }
    fn observation(text: &str, pinyin: &str) -> VocabularyObservation {
        VocabularyObservation {
            hanzi: text.to_owned(),
            pinyin: pinyin.to_owned(),
            meaning: "AI meaning".to_owned(),
        }
    }
    #[test]
    fn matches_exact_word_and_keeps_ai_meaning_separate() {
        let result = reconcile_vocabulary(
            &[observation("水", "shuǐ")],
            vec![word(1, "水", "shuǐ", Some(335))],
        );
        assert_eq!(result[0].dictionary.as_ref().unwrap().frequency, Some(335));
        assert_eq!(result[0].observation.meaning, "AI meaning");
    }
    #[test]
    fn missing_candidate_is_not_marked_verified() {
        let result = reconcile_vocabulary(&[observation("一本", "yī běn")], vec![]);
        assert!(result[0].dictionary.is_none());
    }
    #[test]
    fn pinyin_precedes_frequency_when_homographs_ambiguous() {
        let result = reconcile_vocabulary(
            &[observation("行", "háng")],
            vec![
                word(1, "行", "xíng", Some(1)),
                word(2, "行", "háng", Some(400)),
            ],
        );
        assert_eq!(result[0].dictionary.as_ref().unwrap().id, 2);
        assert_eq!(result[0].alternatives, 1);
    }
    #[test]
    fn preserves_ai_order_and_labels_for_repeated_queries() {
        let observations = [observation("借", "jiè"), observation("水", "shuǐ")];
        let result = reconcile_vocabulary(&observations, vec![word(3, "水", "shuǐ", Some(300))]);
        assert_eq!(result[0].observation.hanzi, "借");
        assert!(result[0].dictionary.is_none());
        assert!(result[1].dictionary.is_some());
    }
}
