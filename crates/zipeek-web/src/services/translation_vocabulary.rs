use crate::services::word_sets::WordSetId;
use std::collections::BTreeSet;

use zipeek_ai_protocol::{SentenceVocabulary, starter_set};
use zipeek_core::{Hsk3Level, HskLevel, HskSystem, WordId};

use crate::{
    contexts::database::Dictionary,
    services::{practice_filter_source, word_sets},
};
use zipeek_exercises::practice_draft::DictionaryFilters;

#[derive(Debug, PartialEq, Eq)]
enum VocabularySource {
    Sample(String),
    Saved(WordSetId),
    Hsk(HskLevel),
    Filters(DictionaryFilters),
}

fn parse_source(value: &str) -> Result<VocabularySource, String> {
    if value.starts_with("filters:") {
        return practice_filter_source::decode(value).map(VocabularySource::Filters);
    }
    if let Some(id) = value.strip_prefix("starter:") {
        return starter_set(id)
            .map(|_| VocabularySource::Sample(id.to_owned()))
            .ok_or_else(|| "Unknown sample topic".to_owned());
    }
    if let Some(index) = value.strip_prefix("saved:") {
        return index
            .parse::<WordSetId>()
            .map(VocabularySource::Saved)
            .map_err(|_| "Invalid saved Word Set selection".to_owned());
    }
    if let Some(spec) = value.strip_prefix("hsk:") {
        let (system, number) = spec.split_once(':').ok_or("Invalid HSK selection")?;
        let number = number.parse::<i32>().map_err(|_| "Invalid HSK level")?;
        let system = HskSystem::from_code(system).ok_or("Unsupported HSK standard")?;
        let level = HskLevel::try_from((system, number)).map_err(|_| "Unsupported HSK level")?;
        return Ok(VocabularySource::Hsk(level));
    }
    Err("Choose an HSK level, saved Word Set, or sample topic".to_owned())
}

pub async fn load(
    source: &str,
    dictionary: &Dictionary,
    storage: &web_sys::Storage,
) -> Result<SentenceVocabulary, String> {
    match parse_source(source)? {
        VocabularySource::Sample(id) => {
            let set = starter_set(&id).ok_or("Unknown sample topic".to_owned())?;
            SentenceVocabulary::new(
                set.name.to_owned(),
                set.vocabulary.iter().map(|word| (*word).to_owned()),
            )
        }
        VocabularySource::Saved(id) => {
            let sets = word_sets::read(storage)?;
            let set = word_sets::find(&sets, &id).ok_or("Saved Word Set no longer exists")?;
            let ids = set
                .word_ids
                .iter()
                .copied()
                .take(128)
                .map(|id| {
                    WordId::try_from(i64::from(id)).map_err(|_| format!("Invalid Word ID {id}"))
                })
                .collect::<Result<BTreeSet<_>, _>>()?;
            let db = dictionary.database().map_err(|error| error.to_string())?;
            let words = db
                .words_by_ids(&ids)
                .await
                .map_err(|error| error.to_string())?;
            SentenceVocabulary::new(
                set.name.clone(),
                words.into_iter().map(|word| word.simplified),
            )
        }
        VocabularySource::Filters(filters) => {
            let db = dictionary.database().map_err(|error| error.to_string())?;
            let words = db
                .practice_words_for_filters(&filters, 128)
                .await
                .map_err(|error| error.to_string())?;
            SentenceVocabulary::new(
                "Filtered dictionary vocabulary".to_owned(),
                words.into_iter().map(|word| word.simplified),
            )
        }
        VocabularySource::Hsk(level) => {
            let db = dictionary.database().map_err(|error| error.to_string())?;
            let words = db
                .translation_words_for_hsk(level)
                .await
                .map_err(|error| error.to_string())?;
            SentenceVocabulary::new(
                level.badge_label(),
                words.into_iter().map(|word| word.simplified),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_parser_accepts_known_sources() {
        assert_eq!(
            parse_source("starter:travel").unwrap(),
            VocabularySource::Sample("travel".to_owned())
        );
        assert_eq!(
            parse_source("saved:00112233445566778899aabbccddeeff").unwrap(),
            VocabularySource::Saved("00112233445566778899aabbccddeeff".parse().unwrap())
        );
        assert_eq!(
            parse_source("hsk:3:7").unwrap(),
            VocabularySource::Hsk(HskLevel::Hsk3(Hsk3Level::SevenToNine))
        );
        assert!(matches!(
            parse_source("filters:2.6,3.7|travel|noun"),
            Ok(VocabularySource::Filters(_))
        ));
    }

    #[test]
    fn invalid_sources_never_fall_back_to_unrelated_vocabulary() {
        for invalid in [
            "",
            "travel",
            "starter:missing",
            "saved:-1",
            "saved:garbage",
            "hsk:2:7",
            "hsk:3:8",
            "hsk:4:1",
            "hsk:2:one",
        ] {
            assert!(
                parse_source(invalid).is_err(),
                "Unexpected acceptance: {invalid}"
            );
        }
    }
}
