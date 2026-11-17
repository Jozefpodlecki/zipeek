use std::collections::BTreeMap;

use zipeek_core::{Word, WordId};

use super::{
    ImportDiagnostic, ImportError,
    diagnostics::{ImportDiagnosticKind as Kind, record},
    glosses::glosses,
    reading::Reading,
    senses::{SourceSense, source_glosses},
    source,
    validation::{validate_entry, validated_transcriptions},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HskImport {
    pub words: Vec<Word>,
    pub senses: Vec<SourceSense>,
    pub diagnostics: Vec<ImportDiagnostic>,
    pub source_entries: usize,
    pub rejected_entries: usize,
    pub character_readings: Vec<super::CharacterReading>,
}

pub fn parse_hsk(bytes: &[u8]) -> Result<HskImport, ImportError> {
    let entries = source::parse(bytes)?;
    let source_entries = entries.len();
    let mut rejected_entries = 0;
    let mut diagnostics = Vec::new();
    let mut readings: BTreeMap<(String, String), Reading> = BTreeMap::new();
    for (index, (source, original)) in entries.into_iter().enumerate() {
        let entry = index + 1;
        let Some(metadata) = validate_entry(&source, entry, &mut diagnostics) else {
            rejected_entries += 1;
            continue;
        };
        let simplified = &source.simplified;
        let mut accepted_forms = 0;
        for (form_index, form) in source.forms.into_iter().enumerate() {
            let Some((pinyin, numeric)) =
                validated_transcriptions(&form, form_index, entry, simplified, &mut diagnostics)
            else {
                continue;
            };
            accepted_forms += 1;
            let key = (simplified.clone(), numeric.clone());
            let english = glosses(&form, entry, simplified, &mut diagnostics);
            let incoming = Reading {
                simplified: simplified.clone(),
                traditional: form.traditional.clone(),
                pinyin: pinyin.clone(),
                numeric: numeric.clone(),
                english,
                source_glosses: source_glosses(&form.meanings),
                pos: metadata.pos.clone(),
                old: metadata.old,
                new: metadata.new,
                frequency: metadata.frequency,
                original: original.clone(),
                entry,
            };
            if let Some(previous) = readings.get_mut(&key) {
                previous.merge(incoming, form_index, &mut diagnostics);
            } else {
                readings.insert(key, incoming);
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
    let (words, senses) = finalize_readings(readings, &mut diagnostics)?;
    let repeated_readings = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind == Kind::RepeatedReading)
        .count();
    tracing::info!(
        repeated_readings,
        other_diagnostics = diagnostics.len() - repeated_readings,
        "HSK import diagnostic summary"
    );
    Ok(HskImport {
        words,
        senses,
        diagnostics,
        source_entries,
        rejected_entries,
        character_readings: Vec::new(),
    })
}

fn finalize_readings(
    readings: BTreeMap<(String, String), Reading>,
    diagnostics: &mut Vec<ImportDiagnostic>,
) -> Result<(Vec<Word>, Vec<SourceSense>), ImportError> {
    let mut display_keys = BTreeMap::new();
    let mut words = Vec::with_capacity(readings.len());
    let mut senses = Vec::new();
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
                diagnostics,
                Kind::MissingEnglish,
                reading.entry,
                &reading.simplified,
                format!(
                    "no primary gloss for {:?}; fallback required",
                    reading.numeric
                ),
            );
        }
        let row = index + 1;
        let id = i64::try_from(row)
            .ok()
            .and_then(|value| WordId::try_from(value).ok())
            .ok_or(ImportError::IdOverflow { row })?;
        for (position, (source_gloss_order, gloss)) in reading.source_glosses.iter().enumerate() {
            senses.push(SourceSense {
                word_id: id,
                sense_order: (position + 1) as i32,
                gloss: gloss.clone(),
                source: "hsk-primary",
                source_ref: reading.entry as i32,
                source_gloss_order: *source_gloss_order,
            });
        }
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
    Ok((words, senses))
}
