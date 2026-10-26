use std::collections::BTreeMap;

use zipeek_core::{Hsk2Level, Hsk3Level, PartOfSpeech, Word, WordId};

use super::{
    ImportDiagnostic, ImportError,
    diagnostics::{ImportDiagnosticKind as Kind, record},
    levels,
    source::{self, SourceForm},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HskImport {
    pub words: Vec<Word>,
    pub diagnostics: Vec<ImportDiagnostic>,
    pub source_entries: usize,
    pub rejected_entries: usize,
}

struct Reading {
    simplified: String,
    traditional: Option<String>,
    pinyin: String,
    numeric: String,
    english: Option<String>,
    pos: Vec<PartOfSpeech>,
    old: Option<Hsk2Level>,
    new: Option<Hsk3Level>,
    frequency: Option<i32>,
    original: String,
    entry: usize,
}

pub fn parse_hsk(bytes: &[u8]) -> Result<HskImport, ImportError> {
    let entries = source::parse(bytes)?;
    let source_entries = entries.len();
    let mut rejected_entries = 0;
    let mut diagnostics = Vec::new();
    let mut readings: BTreeMap<(String, String), Reading> = BTreeMap::new();
    for (index, (source, original)) in entries.into_iter().enumerate() {
        let entry = index + 1;
        let simplified = &source.simplified;
        let pos = match PartOfSpeech::from_source_codes(source.pos.iter().map(String::as_str)) {
            Ok(pos) => pos,
            Err(_) => {
                let unknown: Vec<_> = source
                    .pos
                    .iter()
                    .filter(|code| PartOfSpeech::from_source_code(code).is_err())
                    .collect();
                record(
                    &mut diagnostics,
                    Kind::RejectedEntry,
                    entry,
                    simplified,
                    format!(
                        "unknown PoS codes {unknown:?}; rejecting all {} forms",
                        source.forms.len()
                    ),
                );
                rejected_entries += 1;
                continue;
            }
        };
        let (old, new) = levels::memberships(&source.level, entry, simplified, &mut diagnostics);
        let frequency = match source.frequency.map(i32::try_from).transpose() {
            Ok(value) => value,
            Err(_) => {
                record(
                    &mut diagnostics,
                    Kind::RejectedEntry,
                    entry,
                    simplified,
                    format!(
                        "frequency {:?} is outside SQL INTEGER range",
                        source.frequency
                    ),
                );
                rejected_entries += 1;
                continue;
            }
        };
        if simplified.trim().is_empty()
            || (old.is_none() && new.is_none())
            || source.forms.is_empty()
        {
            record(
                &mut diagnostics,
                Kind::RejectedEntry,
                entry,
                simplified,
                "missing headword, HSK membership or forms",
            );
            rejected_entries += 1;
            continue;
        }
        let mut accepted_forms = 0;
        for (form_index, form) in source.forms.into_iter().enumerate() {
            let Some(pinyin) = form
                .transcriptions
                .pinyin
                .as_ref()
                .filter(|text| !text.trim().is_empty())
            else {
                record(
                    &mut diagnostics,
                    Kind::RejectedForm,
                    entry,
                    simplified,
                    format!("form {}: missing or blank display pinyin", form_index + 1),
                );
                continue;
            };
            let Some(numeric) = form
                .transcriptions
                .numeric
                .as_ref()
                .filter(|text| !text.trim().is_empty())
            else {
                record(
                    &mut diagnostics,
                    Kind::RejectedForm,
                    entry,
                    simplified,
                    format!("form {}: missing or blank numeric pinyin", form_index + 1),
                );
                continue;
            };
            accepted_forms += 1;
            let key = (simplified.clone(), numeric.clone());
            let english = glosses(&form, entry, simplified, &mut diagnostics);
            if let Some(previous) = readings.get_mut(&key) {
                record(
                    &mut diagnostics,
                    Kind::RepeatedReading,
                    entry,
                    simplified,
                    format!(
                        "form {} repeats numeric reading {numeric:?}; retaining first form",
                        form_index + 1
                    ),
                );
                if previous.pinyin != *pinyin {
                    record(
                        &mut diagnostics,
                        Kind::DisplayDrift,
                        entry,
                        simplified,
                        format!(
                            "numeric {numeric:?}: retained {:?}, ignored {pinyin:?}",
                            previous.pinyin
                        ),
                    );
                }
                if previous.english.is_none() {
                    previous.english = english;
                }
                previous.pos.extend(pos.iter().copied());
                previous.pos.sort_unstable_by_key(|value| value.as_str());
                previous.pos.dedup();
                levels::merge(
                    &mut previous.old,
                    old,
                    "HSK 2.0",
                    entry,
                    simplified,
                    &mut diagnostics,
                );
                levels::merge(
                    &mut previous.new,
                    new,
                    "HSK 3.0",
                    entry,
                    simplified,
                    &mut diagnostics,
                );
                if let (Some(first), Some(next)) = (previous.frequency, frequency) {
                    if first != next {
                        record(
                            &mut diagnostics,
                            Kind::FrequencyConflict,
                            entry,
                            simplified,
                            format!("retained first rank {first}, ignored {next}"),
                        );
                    }
                } else if previous.frequency.is_none() {
                    previous.frequency = frequency;
                }
            } else {
                readings.insert(
                    key,
                    Reading {
                        simplified: simplified.clone(),
                        traditional: form.traditional.clone(),
                        pinyin: pinyin.clone(),
                        numeric: numeric.clone(),
                        english,
                        pos: pos.clone(),
                        old,
                        new,
                        frequency,
                        original: original.clone(),
                        entry,
                    },
                );
            }
        }
        if accepted_forms == 0 {
            rejected_entries += 1;
            record(
                &mut diagnostics,
                Kind::RejectedEntry,
                entry,
                simplified,
                "all forms lack required pinyin",
            );
        }
    }
    let mut display_keys = BTreeMap::new();
    let mut words = Vec::with_capacity(readings.len());
    for (index, reading) in readings.into_values().enumerate() {
        let key = (reading.simplified.clone(), reading.pinyin.clone());
        if let Some(first) = display_keys.insert(key, reading.original.clone()) {
            let error = ImportError::DisplayKeyConflict {
                simplified: reading.simplified,
                pinyin: reading.pinyin,
                first,
                second: reading.original,
            };
            tracing::error!(%error, "Conflicting source rows; refusing generation");
            return Err(error);
        }
        if reading.english.is_none() {
            record(
                &mut diagnostics,
                Kind::MissingEnglish,
                reading.entry,
                &reading.simplified,
                format!(
                    "no primary gloss for {:?}; retained NULL pending A04 fallback",
                    reading.numeric
                ),
            );
        }
        let row = index + 1;
        let id = i64::try_from(row)
            .ok()
            .and_then(|value| WordId::try_from(value).ok())
            .ok_or(ImportError::IdOverflow { row })?;
        words.push(Word {
            id,
            simplified: reading.simplified,
            traditional: reading.traditional,
            pinyin: reading.pinyin,
            pinyin_numeric: Some(reading.numeric),
            english: reading.english,
            part_of_speech: reading.pos,
            hsk2_level: reading.old,
            hsk3_level: reading.new,
            frequency: reading.frequency,
        });
    }
    Ok(HskImport {
        words,
        diagnostics,
        source_entries,
        rejected_entries,
    })
}

fn glosses(
    form: &SourceForm,
    entry: usize,
    simplified: &str,
    diagnostics: &mut Vec<ImportDiagnostic>,
) -> Option<String> {
    let values: Vec<_> = form
        .meanings
        .iter()
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
        .map(|text| {
            let replaced = text.replace('|', "｜");
            if text != replaced {
                record(
                    diagnostics,
                    Kind::GlossPipeReplacement,
                    entry,
                    simplified,
                    format!("original {text:?}; replaced {replaced:?}"),
                );
            }
            replaced
        })
        .collect();
    (!values.is_empty()).then(|| values.join("|"))
}
