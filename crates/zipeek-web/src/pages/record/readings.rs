use std::collections::{BTreeMap, BTreeSet};
use crate::services::database::TextReadingCandidates;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct ReadingGlyph {
    pub character: char,
    pub pinyin: Option<String>,
}

fn marked_syllable(numeric: &str) -> Option<String> {
    let (base, suffix) = numeric.split_at(numeric.len().checked_sub(1)?);
    let tone = suffix.chars().next()?.to_digit(10)? as usize;
    if tone > 5 || base.is_empty() { return None; }
    let normalized = base.to_lowercase().replace("u:", "ü").replace('v', "ü");
    if tone == 0 || tone == 5 { return Some(normalized); }
    let letters: Vec<char> = normalized.chars().collect();
    let target = letters.iter().position(|c| *c == 'a')
        .or_else(|| letters.iter().position(|c| *c == 'e'))
        .or_else(|| normalized.find("ou"))
        .or_else(|| letters.iter().rposition(|c| matches!(c, 'a'|'e'|'i'|'o'|'u'|'ü')))?;
    let vowel = match letters[target] {
        'a' => ["ā", "á", "ǎ", "à"],
        'e' => ["ē", "é", "ě", "è"],
        'i' => ["ī", "í", "ǐ", "ì"],
        'o' => ["ō", "ó", "ǒ", "ò"],
        'u' => ["ū", "ú", "ǔ", "ù"],
        'ü' => ["ǖ", "ǘ", "ǚ", "ǜ"],
        _ => return None,
    };
    let mut result = String::new();
    for (index, character) in letters.into_iter().enumerate() {
        if index == target { result.push_str(vowel[tone - 1]); } else { result.push(character); }
    }
    Some(result)
}

pub(super) fn aligned_syllables(text: &str, numeric: &str) -> Option<Vec<String>> {
    let syllables: Option<Vec<_>> = numeric.split_whitespace().map(marked_syllable).collect();
    let syllables = syllables?;
    (syllables.len() == text.chars().count()).then_some(syllables)
}

pub(super) fn resolve(text: &str, candidates: &TextReadingCandidates) -> Vec<ReadingGlyph> {
    let mut word_readings: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for word in &candidates.words {
        if let Some(reading) = word.pinyin_numeric.as_deref() {
            word_readings.entry(&word.simplified).or_default().insert(reading);
        }
    }
    let mut character_readings: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for reading in &candidates.characters {
        character_readings.entry(&reading.character).or_default().insert(&reading.pinyin_numeric);
    }
    let source: Vec<char> = text.chars().collect();
    let mut result = Vec::with_capacity(source.len());
    let mut index = 0;
    while index < source.len() {
        let mut found: Option<(usize, Vec<String>)> = None;
        for size in (2..=5).rev() {
            if let Some(fragment) = source.get(index..index.saturating_add(size)) {
                let key = fragment.iter().collect::<String>();
                if let Some(readings) = word_readings.get(key.as_str()) {
                    if readings.len() == 1 {
                        if let Some(syllables) = aligned_syllables(&key, readings.iter().next().unwrap()) {
                            found = Some((size, syllables));
                            break;
                        }
                    }
                }
            }
        }
        if let Some((count, syllables)) = found {
            for (character, pinyin) in source[index..index + count].iter().copied().zip(syllables) {
                result.push(ReadingGlyph { character, pinyin: Some(pinyin) });
            }
            index += count;
        } else {
            let character = source[index];
            let key = character.to_string();
            let pinyin = character_readings.get(key.as_str())
                .filter(|readings| readings.len() == 1)
                .and_then(|readings| marked_syllable(readings.iter().next().unwrap()));
            result.push(ReadingGlyph { character, pinyin });
            index += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::database::{CharacterPinyinReading, TextWordReading};

    #[test]
    fn tone_marks_and_neutral_tones() {
        assert_eq!(marked_syllable("lü4"), Some("lǜ".into()));
        assert_eq!(marked_syllable("zhong1"), Some("zhōng".into()));
        assert_eq!(marked_syllable("xue2"), Some("xué".into()));
        assert_eq!(marked_syllable("ma5"), Some("ma".into()));
    }

    #[test]
    fn word_context_beats_ambiguous_character_reading() {
        let candidates = TextReadingCandidates {
            words: vec![TextWordReading { simplified: "银行".into(), pinyin_numeric: Some("yin2 hang2".into()) }],
            characters: vec!["xing2", "hang2"].into_iter().map(|reading| CharacterPinyinReading { character: "行".into(), traditional: "行".into(), pinyin_numeric: reading.into() }).collect(),
        };
        let resolved = resolve("银行。行", &candidates);
        assert_eq!(resolved[1].pinyin.as_deref(), Some("háng"));
        assert_eq!(resolved[3].pinyin, None);
        assert_eq!(resolved[2].pinyin, None);
    }
}
