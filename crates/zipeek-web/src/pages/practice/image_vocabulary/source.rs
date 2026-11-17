use std::collections::{BTreeSet, HashMap};

use zipeek_core::{Word, WordId};

use crate::contexts::database::Dictionary;
use crate::services::practice_filter_source;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ImageSource {
    Hsk { standard: u8, level: u8 },
    Filters(String),
    WordSet(Vec<i32>),
    Invalid(String),
}

impl ImageSource {
    pub(super) fn from_query(query: &HashMap<String, String>) -> Self {
        if query.contains_key("set_ids") && query.contains_key("source") {
            return Self::Invalid("Select one vocabulary source".to_owned());
        }
        if let Some(raw) = query.get("set_ids") {
            if raw.is_empty() || raw.len() > 16_384 {
                return Self::Invalid("Invalid Word Set selection".to_owned());
            }
            let ids = raw
                .split(',')
                .map(str::parse::<i32>)
                .collect::<Result<Vec<_>, _>>();
            return match ids {
                Ok(ids) if ids.len() <= 1_000 && ids.iter().all(|id| *id > 0) => Self::WordSet(ids),
                _ => Self::Invalid("Invalid Word Set identifiers".to_owned()),
            };
        }
        if let Some(value) = query.get("source") {
            return if value.starts_with("filters:") {
                Self::Filters(value.clone())
            } else {
                Self::Invalid("Invalid vocabulary source".to_owned())
            };
        }
        let standard = match query.get("standard") {
            Some(raw) => match raw.parse::<u8>() {
                Ok(value) => value,
                Err(_) => return Self::Invalid("Invalid HSK standard".to_owned()),
            },
            None => 3,
        };
        let level = match query.get("level") {
            Some(raw) => match raw.parse::<u8>() {
                Ok(value) => value,
                Err(_) => return Self::Invalid("Invalid HSK level".to_owned()),
            },
            None => 1,
        };
        if zipeek_exercises::practice_draft::HskBand::new(standard, level).is_err() {
            return Self::Invalid("Unsupported HSK level".to_owned());
        }
        Self::Hsk { standard, level }
    }

    pub(super) async fn load(&self, dictionary: &Dictionary) -> Result<Vec<Word>, String> {
        if let Self::Invalid(message) = self {
            return Err(message.clone());
        }
        let db = dictionary.database().map_err(|error| error.to_string())?;
        match self {
            Self::Invalid(message) => Err(message.clone()),
            Self::Hsk { standard, level } => db
                .game_words_for_hsk(*standard, *level)
                .await
                .map_err(|error| error.to_string()),
            Self::Filters(encoded) => {
                let filters = practice_filter_source::decode(encoded)?;
                db.practice_eligible_words_for_filters(&filters, 128)
                    .await
                    .map_err(|error| error.to_string())
            }
            Self::WordSet(ids) => {
                if ids.is_empty() {
                    return Err("The selected Word Set contains no words".to_owned());
                }
                let ids = ids
                    .iter()
                    .map(|id| WordId::try_from(i64::from(*id)).map_err(|error| error.to_string()))
                    .collect::<Result<BTreeSet<_>, _>>()?;
                db.practice_eligible_words_by_ids(&ids)
                    .await
                    .map_err(|error| error.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_sources_do_not_fall_back_to_unrelated_vocabulary() {
        for query in [
            HashMap::from([("set_ids".to_owned(), "1,garbage,3".to_owned())]),
            HashMap::from([("set_ids".to_owned(), "1,,3".to_owned())]),
            HashMap::from([("source".to_owned(), "invalid".to_owned())]),
            HashMap::from([("standard".to_owned(), "invalid".to_owned())]),
            HashMap::from([
                ("standard".to_owned(), "3".to_owned()),
                ("level".to_owned(), "99".to_owned()),
            ]),
            HashMap::from([
                ("set_ids".to_owned(), "1".to_owned()),
                ("source".to_owned(), "filters:3.1||".to_owned()),
            ]),
        ] {
            assert!(matches!(
                ImageSource::from_query(&query),
                ImageSource::Invalid(_)
            ));
        }
    }

    #[test]
    fn selected_dictionary_filters_survive_navigation() {
        let query = HashMap::from([("source".to_owned(), "filters:3.1|travel|noun".to_owned())]);
        assert_eq!(
            ImageSource::from_query(&query),
            ImageSource::Filters("filters:3.1|travel|noun".into())
        );
    }

    #[test]
    fn direct_entry_uses_beginner_hsk_and_set_ids_override_it() {
        assert_eq!(
            ImageSource::from_query(&HashMap::new()),
            ImageSource::Hsk {
                standard: 3,
                level: 1
            }
        );
        let query = HashMap::from([("set_ids".to_owned(), "12,8".to_owned())]);
        assert_eq!(
            ImageSource::from_query(&query),
            ImageSource::WordSet(vec![12, 8])
        );
    }
}
