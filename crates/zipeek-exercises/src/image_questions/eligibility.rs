use std::collections::BTreeSet;

use super::model::{ImageQuestionError, VisualWord};

fn visual_name(text: &str) -> Option<String> {
    let primary = text.split([';', '|', '/']).next()?.trim();
    let primary = primary.strip_prefix("to ").unwrap_or(primary);
    let name = primary
        .trim_matches(|character: char| matches!(character, '(' | ')' | '.' | ','))
        .trim();
    if name.is_empty()
        || name.len() > 42
        || name
            .chars()
            .any(|character| character.is_control() || character.is_ascii_digit())
    {
        return None;
    }
    let lowercase = name.to_ascii_lowercase();
    const NON_VISUAL: [&str; 9] = [
        "classifier",
        "particle",
        "measure word",
        "surname",
        "prefix",
        "suffix",
        "variant of",
        "abbreviation",
        "see also",
    ];
    if NON_VISUAL.iter().any(|marker| lowercase.contains(marker)) {
        return None;
    }
    Some(name.to_owned())
}

pub(super) fn eligible_words(words: &[VisualWord]) -> Result<Vec<VisualWord>, ImageQuestionError> {
    let mut ids = BTreeSet::new();
    for word in words.iter().filter(|word| word.id > 0) {
        if !ids.insert(word.id) {
            return Err(ImageQuestionError::DuplicateWordId);
        }
    }

    let mut eligible = words
        .iter()
        .filter_map(|word| {
            if word.id <= 0 || word.hanzi.trim().is_empty() || word.hanzi.chars().count() > 36 {
                return None;
            }
            let english = visual_name(&word.english)?;
            Some(VisualWord {
                id: word.id,
                hanzi: word.hanzi.trim().to_owned(),
                english,
            })
        })
        .collect::<Vec<_>>();
    eligible.sort_by_key(|word| word.id);

    let mut seen_glosses = BTreeSet::new();
    eligible.retain(|word| seen_glosses.insert(word.english.to_lowercase()));
    Ok(eligible)
}
