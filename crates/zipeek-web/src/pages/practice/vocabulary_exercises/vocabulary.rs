use std::collections::{BTreeSet, HashMap};

use zipeek_core::{Word, WordId};

use crate::{contexts::database::Dictionary, services::practice_filter_source};

pub(super) fn meaning(word: &Word) -> Option<&str> {
    word.english
        .as_deref()
        .and_then(|english| english.split('|').next())
        .map(str::trim)
        .filter(|gloss| !gloss.is_empty())
}

pub(super) fn concise_prompt_meaning(word: &Word) -> Option<&str> {
    meaning(word)
        .and_then(|gloss| gloss.split(';').next())
        .map(str::trim)
        .filter(|gloss| !gloss.is_empty())
}

fn eligible(word: &Word) -> bool {
    !word.simplified.trim().is_empty() && !word.pinyin.trim().is_empty() && meaning(word).is_some()
}

pub(super) fn distinct_eligible_words(words: Vec<Word>, unique_meanings: bool) -> Vec<Word> {
    let mut hanzi = BTreeSet::new();
    let mut meanings = BTreeSet::new();
    words
        .into_iter()
        .filter(eligible)
        .filter(|word| hanzi.insert(word.simplified.trim().to_owned()))
        .filter(|word| {
            !unique_meanings || meanings.insert(meaning(word).unwrap_or_default().to_lowercase())
        })
        .collect()
}

pub(super) fn parse_word_ids(raw: &str) -> Result<BTreeSet<WordId>, String> {
    if raw.len() > 16_384 {
        return Err("Word Set selection is too long".to_owned());
    }
    if raw.is_empty() {
        return Err("The selected Word Set is empty".to_owned());
    }
    let ids = raw
        .split(',')
        .map(|id| {
            let id = id
                .parse::<i64>()
                .map_err(|_| "Invalid Word Set identifier")?;
            WordId::try_from(id).map_err(|_| "Invalid Word Set identifier".to_owned())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if ids.is_empty() {
        return Err("The selected Word Set is empty".to_owned());
    }
    if ids.len() > 1_000 {
        return Err("The selected Word Set exceeds the 1,000-word practice limit".to_owned());
    }
    Ok(ids)
}

pub(crate) async fn load_words(
    dictionary: &Dictionary,
    query: &HashMap<String, String>,
    unique_meanings: bool,
) -> Result<Vec<Word>, String> {
    if query.contains_key("set_ids") && query.contains_key("source") {
        return Err("Select one vocabulary source".to_owned());
    }
    let database = dictionary.database().map_err(|error| error.to_string())?;
    let words = if let Some(raw) = query.get("set_ids") {
        let ids = parse_word_ids(raw)?;
        database
            .practice_eligible_words_by_ids(&ids)
            .await
            .map_err(|error| error.to_string())?
    } else if let Some(source) = query.get("source") {
        let filters = practice_filter_source::decode(source)?;
        database
            .practice_eligible_words_for_filters(&filters, 128)
            .await
            .map_err(|error| error.to_string())?
    } else {
        return Err("Choose a vocabulary source in Practice setup".to_owned());
    };
    Ok(distinct_eligible_words(words, unique_meanings))
}

#[cfg(test)]
mod tests;
