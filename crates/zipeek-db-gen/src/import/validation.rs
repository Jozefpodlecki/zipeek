use super::{
    ImportDiagnostic,
    diagnostics::{ImportDiagnosticKind as Kind, record},
    levels,
    source::{self, SourceForm},
};
use zipeek_core::{Hsk2Level, Hsk3Level, PartOfSpeech};

pub(super) struct EntryMetadata {
    pub pos: Vec<PartOfSpeech>,
    pub old: Option<Hsk2Level>,
    pub new: Option<Hsk3Level>,
    pub frequency: Option<i32>,
}

pub(super) fn validate_entry(
    source: &source::SourceEntry,
    entry: usize,
    diagnostics: &mut Vec<ImportDiagnostic>,
) -> Option<EntryMetadata> {
    let simplified = &source.simplified;
    let mut pos = Vec::new();
    let mut unknown = Vec::new();
    for code in &source.pos {
        match super::pos_mapping::normalize(code) {
            Ok(value) => pos.push(value),
            Err(_) => unknown.push(code.as_str()),
        }
    }
    pos.sort_unstable_by_key(|value| value.as_str());
    pos.dedup();
    if !unknown.is_empty() {
        record(
            diagnostics,
            Kind::UnknownPartOfSpeech,
            entry,
            simplified,
            format!("unknown PoS codes {unknown:?}; retaining entry and known codes"),
        );
    }
    let (old, new) = levels::memberships(&source.level, entry, simplified, diagnostics);
    let frequency = match source.frequency.map(i32::try_from).transpose() {
        Ok(value) => value,
        Err(_) => {
            record(
                diagnostics,
                Kind::RejectedEntry,
                entry,
                simplified,
                format!(
                    "frequency {:?} is outside SQL INTEGER range",
                    source.frequency
                ),
            );
            return None;
        }
    };
    if simplified.trim().is_empty() || (old.is_none() && new.is_none()) || source.forms.is_empty() {
        record(
            diagnostics,
            Kind::RejectedEntry,
            entry,
            simplified,
            "missing headword, HSK membership or forms",
        );
        return None;
    }
    Some(EntryMetadata {
        pos,
        old,
        new,
        frequency,
    })
}

pub(super) fn validated_transcriptions<'a>(
    form: &'a SourceForm,
    form_index: usize,
    entry: usize,
    simplified: &str,
    diagnostics: &mut Vec<ImportDiagnostic>,
) -> Option<(&'a String, &'a String)> {
    let Some(pinyin) = form
        .transcriptions
        .pinyin
        .as_ref()
        .filter(|text| !text.trim().is_empty())
    else {
        record(
            diagnostics,
            Kind::RejectedForm,
            entry,
            simplified,
            format!("form {}: missing or blank display pinyin", form_index + 1),
        );
        return None;
    };
    let Some(numeric) = form
        .transcriptions
        .numeric
        .as_ref()
        .filter(|text| !text.trim().is_empty())
    else {
        record(
            diagnostics,
            Kind::RejectedForm,
            entry,
            simplified,
            format!("form {}: missing or blank numeric pinyin", form_index + 1),
        );
        return None;
    };
    Some((pinyin, numeric))
}

pub(super) fn valid_cedict_reading(normalized: &str) -> bool {
    let mut has_letter = false;
    let mut has_tone_or_vowel = false;
    for character in normalized.chars() {
        if character.is_ascii_alphabetic() {
            has_letter = true;
            if matches!(character, 'a' | 'e' | 'i' | 'o' | 'u' | 'v') {
                has_tone_or_vowel = true;
            }
        } else if matches!(character, '1'..='5') {
            has_tone_or_vowel = true;
        } else {
            return false;
        }
    }
    has_letter && has_tone_or_vowel
}

#[cfg(test)]
mod cedict_reading_tests {
    use super::valid_cedict_reading;

    #[test]
    fn accepts_tones_and_neutral_vowels_without_accepting_invalid_symbols() {
        for accepted in ["yao2gun3yue4", "nv3", "ng5", "ma", "xing2"] {
            assert!(valid_cedict_reading(accepted), "rejected {accepted}");
        }
        for rejected in ["", "r", "ng", "z?2", "ma9", "a+b", "123"] {
            assert!(!valid_cedict_reading(rejected), "accepted {rejected}");
        }
    }
}
