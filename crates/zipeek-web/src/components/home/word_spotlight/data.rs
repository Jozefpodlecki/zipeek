use crate::contexts::database::Dictionary;
use crate::services::database::{BrowseRelation, DictionaryMeaning};
use std::collections::BTreeSet;
use zipeek_core::Word;

pub(super) async fn load(
    dictionary: &Dictionary,
    day: Option<u32>,
) -> Result<(Word, Vec<DictionaryMeaning>, Vec<BrowseRelation>), String> {
    let database = dictionary.database().map_err(|error| error.to_string())?;
    let exclusions = BTreeSet::new();
    let count = database
        .count_words(&exclusions)
        .await
        .map_err(|error| error.to_string())?;
    if count == 0 {
        return Err("The dictionary contains no HSK words".to_owned());
    }
    let offset = match day {
        Some(day) => day % count,
        None => {
            let mut bytes = [0_u8; 4];
            getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
            u32::from_le_bytes(bytes) % count
        }
    };
    let word = database
        .word_at_offset(offset, &exclusions)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Selected word is unavailable".to_owned())?;
    let mut meanings = database
        .word_meanings(word.id)
        .await
        .map_err(|error| error.to_string())?;
    if meanings.is_empty() {
        if let Some(definition) = database
            .word_display_definition(word.id)
            .await
            .map_err(|error| error.to_string())?
            .filter(|definition| !definition.trim().is_empty())
        {
            meanings.push(DictionaryMeaning {
                sense_order: 1,
                definition,
                categories: Vec::new(),
                qualifiers: Vec::new(),
            });
        }
    }
    let relations = database
        .word_relations(word.id)
        .await
        .map_err(|error| error.to_string())?;
    Ok((word, meanings, relations))
}
