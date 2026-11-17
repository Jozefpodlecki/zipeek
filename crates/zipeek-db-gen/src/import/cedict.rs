use std::collections::{BTreeMap, BTreeSet};

use zipeek_core::{Word, WordId};

use super::glosses::serialize_glosses;
use super::{HskImport, SourceSense};

use zipeek_core::normalize_for_comparison;

use super::ImportError;

#[derive(Debug)]
pub(super) struct CedictReading {
    pub line: usize,
    pub traditional: String,
    pub pinyin: String,
    pub glosses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterReading {
    pub character: String,
    pub traditional: String,
    pub pinyin_numeric: String,
    pub source_line: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CedictExpansion {
    pub added_words: usize,
    pub skipped_variants: usize,
    pub character_readings: Vec<CharacterReading>,
}

#[derive(Debug)]
pub struct CedictIndex {
    readings: BTreeMap<(String, String), Vec<CedictReading>>,
    entry_count: usize,
}

impl CedictIndex {
    pub fn parse(bytes: &[u8]) -> Result<Self, ImportError> {
        let text = std::str::from_utf8(bytes).map_err(ImportError::CedictUtf8)?;
        let mut readings: BTreeMap<_, Vec<_>> = BTreeMap::new();
        let mut entry_count = 0;
        for (index, original) in text.lines().enumerate() {
            let line = original.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let number = index + 1;
            let invalid = |reason| ImportError::CedictLine {
                line: number,
                original: original.to_owned(),
                reason,
            };
            let (headwords, remainder) = line
                .split_once('[')
                .ok_or_else(|| invalid("missing opening pinyin bracket"))?;
            let (pinyin, meanings) = remainder
                .split_once(']')
                .ok_or_else(|| invalid("missing closing pinyin bracket"))?;
            let headwords: Vec<_> = headwords.split_whitespace().collect();
            if headwords.len() != 2 || pinyin.trim().is_empty() {
                return Err(invalid(
                    "expected traditional, simplified and nonempty pinyin",
                ));
            }
            let meanings = meanings
                .trim()
                .strip_prefix('/')
                .and_then(|text| text.strip_suffix('/'))
                .ok_or_else(|| invalid("meanings must be enclosed in slashes"))?;
            let glosses = meanings
                .split('/')
                .map(str::trim)
                .filter(|gloss| !gloss.is_empty())
                .map(str::to_owned)
                .collect();
            let key = (headwords[1].to_owned(), normalize_for_comparison(pinyin));
            readings.entry(key).or_default().push(CedictReading {
                line: number,
                traditional: headwords[0].to_owned(),
                pinyin: pinyin.trim().to_owned(),
                glosses,
            });
            entry_count += 1;
        }
        Ok(Self {
            readings,
            entry_count,
        })
    }

    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub(super) fn lookup(&self, simplified: &str, pinyin: &str) -> &[CedictReading] {
        self.readings
            .get(&(simplified.to_owned(), normalize_for_comparison(pinyin)))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

pub fn expand_with_cedict(
    imported: &mut HskImport,
    bytes: &[u8],
) -> Result<CedictExpansion, ImportError> {
    let index = CedictIndex::parse(bytes)?;
    let mut keys: BTreeSet<(String, String)> = imported
        .words
        .iter()
        .map(|word| {
            (
                word.simplified.clone(),
                normalize_for_comparison(word.pinyin_numeric.as_deref().unwrap_or(&word.pinyin)),
            )
        })
        .collect();
    let mut expansion = CedictExpansion::default();
    let mut seen_characters = BTreeSet::new();
    for ((simplified, pinyin), candidates) in &index.readings {
        if simplified.trim().is_empty() || !super::validation::valid_cedict_reading(pinyin) {
            continue;
        }
        for candidate in candidates {
            if simplified.chars().count() == 1 && candidate.traditional.chars().count() == 1 {
                let character = simplified.clone();
                let key = (character.clone(), pinyin.clone());
                if seen_characters.insert(key) {
                    expansion.character_readings.push(CharacterReading {
                        character,
                        traditional: candidate.traditional.clone(),
                        pinyin_numeric: pinyin.clone(),
                        source_line: candidate.line as i32,
                    });
                }
            }
        }
        if keys.contains(&(simplified.clone(), pinyin.clone())) {
            continue;
        }
        let Some(candidate) = candidates.iter().find(|candidate| {
            !candidate.glosses.is_empty()
                && !candidate
                    .glosses
                    .iter()
                    .any(|gloss| gloss.to_ascii_lowercase().starts_with("variant of"))
        }) else {
            expansion.skipped_variants += 1;
            continue;
        };
        let next = imported.words.len() + 1;
        let id = i64::try_from(next)
            .ok()
            .and_then(|value| WordId::try_from(value).ok())
            .ok_or(ImportError::IdOverflow { row: next })?;
        let english = serialize_glosses(
            candidate.glosses.iter().map(String::as_str),
            candidate.line,
            simplified,
            &mut imported.diagnostics,
        );
        let Some(english) = english else {
            continue;
        };
        let source_glosses = super::senses::source_glosses(&candidate.glosses);
        for (position, (source_gloss_order, gloss)) in source_glosses.into_iter().enumerate() {
            imported.senses.push(SourceSense {
                word_id: id,
                sense_order: (position + 1) as i32,
                gloss,
                source: "cc-cedict",
                source_ref: candidate.line as i32,
                source_gloss_order,
            });
        }
        imported.words.push(Word {
            id,
            simplified: simplified.clone(),
            traditional: Some(candidate.traditional.clone()),
            pinyin: candidate.pinyin.clone(),
            pinyin_numeric: Some(pinyin.clone()),
            english: Some(english),
            part_of_speech: Vec::new(),
            hsk2_level: None,
            hsk3_level: None,
            frequency: None,
        });
        keys.insert((simplified.clone(), pinyin.clone()));
        expansion.added_words += 1;
    }
    Ok(expansion)
}
