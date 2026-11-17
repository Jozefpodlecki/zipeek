use std::collections::HashMap;

use zipeek_exercises::character_hunt::Difficulty;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HuntConfig {
    pub difficulty: Difficulty,
    pub rounds: Option<u16>,
    pub vocabulary: VocabularySource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum VocabularySource {
    WordSet(Vec<i32>),
    Hsk { standard: u8, level: u8 },
    Filters(String),
}

impl HuntConfig {
    pub fn from_query(params: &HashMap<String, String>) -> Self {
        let difficulty = params
            .get("difficulty")
            .and_then(|value| Difficulty::parse(value))
            .unwrap_or_default();
        let rounds = params
            .get("rounds")
            .and_then(|value| value.parse::<u16>().ok())
            .filter(|count| *count > 0);
        let vocabulary = if let Some(ids) = params.get("set_ids") {
            VocabularySource::WordSet(
                ids.split(',')
                    .map(str::trim)
                    .map(str::parse::<i32>)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|id| *id > 0)
                    .collect(),
            )
        } else {
            match params.get("source").map(String::as_str) {
                None | Some("hsk") => VocabularySource::Hsk {
                    standard: params
                        .get("standard")
                        .and_then(|value| value.parse::<u8>().ok())
                        .unwrap_or(3),
                    level: params
                        .get("level")
                        .and_then(|value| value.parse::<u8>().ok())
                        .unwrap_or(1),
                },
                Some(filters) => VocabularySource::Filters(filters.to_owned()),
            }
        };
        Self {
            difficulty,
            rounds,
            vocabulary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_ids_have_priority_over_dictionary_filters() {
        let params = HashMap::from([
            ("set_ids".to_owned(), "3,5".to_owned()),
            ("source".to_owned(), "hsk".to_owned()),
        ]);
        let config = HuntConfig::from_query(&params);
        assert_eq!(config.vocabulary, VocabularySource::WordSet(vec![3, 5]));
    }

    #[test]
    fn invalid_round_limit_cannot_start_zero_rounds() {
        let params = HashMap::from([("rounds".to_owned(), "0".to_owned())]);
        assert_eq!(HuntConfig::from_query(&params).rounds, None);
    }

    #[test]
    fn difficulty_is_decoded_in_one_place() {
        let params = HashMap::from([("difficulty".to_owned(), "hardcore".to_owned())]);
        assert_eq!(
            HuntConfig::from_query(&params).difficulty,
            Difficulty::Hardcore
        );
    }
    #[test]
    fn malformed_word_set_ids_do_not_partially_change_the_vocabulary() {
        let params = HashMap::from([("set_ids".to_owned(), "1,invalid,3".to_owned())]);
        assert_eq!(
            HuntConfig::from_query(&params).vocabulary,
            VocabularySource::WordSet(Vec::new())
        );
        let params = HashMap::from([("set_ids".to_owned(), "-1,2".to_owned())]);
        assert_eq!(
            HuntConfig::from_query(&params).vocabulary,
            VocabularySource::WordSet(vec![2])
        );
    }
}
