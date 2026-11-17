use std::collections::BTreeSet;

use zipeek_core::{Word, WordId};
use zipeek_exercises::character_hunt::Candidate;

use super::config::VocabularySource;
use crate::contexts::database::Dictionary;
use crate::services::practice_filter_source;

pub(super) async fn load_candidates(
    dictionary: &Dictionary,
    source: &VocabularySource,
) -> Result<Vec<Candidate>, String> {
    let database = dictionary.database().map_err(|error| error.to_string())?;
    let words = match source {
        VocabularySource::WordSet(ids) => {
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let keys = ids
                .iter()
                .map(|id| WordId::try_from(i64::from(*id)).map_err(|error| error.to_string()))
                .collect::<Result<BTreeSet<_>, _>>()?;
            database
                .words_by_ids(&keys)
                .await
                .map_err(|error| error.to_string())?
        }
        VocabularySource::Hsk { standard, level } => database
            .game_words_for_hsk(*standard, *level)
            .await
            .map_err(|error| error.to_string())?,
        VocabularySource::Filters(encoded) => {
            let filters = practice_filter_source::decode(encoded)?;
            database
                .character_hunt_words_for_filters(&filters)
                .await
                .map_err(|error| error.to_string())?
        }
    };
    Ok(words.into_iter().filter_map(candidate).collect())
}

fn candidate(word: Word) -> Option<Candidate> {
    let hanzi = word.simplified.trim();
    let count = hanzi.chars().count();
    if !(2..=3).contains(&count) {
        return None;
    }

    let syllables: Vec<String> = word.pinyin.split_whitespace().map(str::to_owned).collect();
    let readings = (syllables.len() == count).then_some(syllables);
    let meaning = word
        .english
        .as_deref()
        .and_then(|glosses| glosses.split('|').next())
        .map(str::trim)
        .filter(|gloss| !gloss.is_empty())
        .map(str::to_owned);
    let candidate = Candidate {
        hanzi: hanzi.to_owned(),
        readings,
        meaning,
    };
    candidate.is_eligible().then_some(candidate)
}
