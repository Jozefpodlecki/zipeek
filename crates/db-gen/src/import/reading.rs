use super::{
    ImportDiagnostic,
    diagnostics::{ImportDiagnosticKind as Kind, record},
    levels,
};
use zipeek_core::{Hsk2Level, Hsk3Level, PartOfSpeech};

pub(super) struct Reading {
    pub simplified: String,
    pub traditional: Option<String>,
    pub pinyin: String,
    pub numeric: String,
    pub english: Option<String>,
    pub pos: Vec<PartOfSpeech>,
    pub old: Option<Hsk2Level>,
    pub new: Option<Hsk3Level>,
    pub frequency: Option<i32>,
    pub original: String,
    pub entry: usize,
}

impl Reading {
    pub fn merge(
        &mut self,
        incoming: Self,
        form_index: usize,
        diagnostics: &mut Vec<ImportDiagnostic>,
    ) {
        let entry = incoming.entry;
        let simplified = &incoming.simplified;
        record(
            diagnostics,
            Kind::RepeatedReading,
            entry,
            simplified,
            format!(
                "form {} repeats numeric reading {:?}; retaining first form",
                form_index + 1,
                incoming.numeric
            ),
        );
        if self.pinyin != incoming.pinyin {
            record(
                diagnostics,
                Kind::DisplayDrift,
                entry,
                simplified,
                format!(
                    "numeric {:?}: retained {:?}, ignored {:?}",
                    incoming.numeric, self.pinyin, incoming.pinyin
                ),
            );
        }
        if self.english.is_none() {
            self.english = incoming.english;
        }
        self.pos.extend(incoming.pos);
        self.pos.sort_unstable_by_key(|value| value.as_str());
        self.pos.dedup();
        levels::merge(
            &mut self.old,
            incoming.old,
            "HSK 2.0",
            entry,
            simplified,
            diagnostics,
        );
        levels::merge(
            &mut self.new,
            incoming.new,
            "HSK 3.0",
            entry,
            simplified,
            diagnostics,
        );
        if let (Some(first), Some(next)) = (self.frequency, incoming.frequency) {
            if first != next {
                record(
                    diagnostics,
                    Kind::FrequencyConflict,
                    entry,
                    simplified,
                    format!("retained first rank {first}, ignored {next}"),
                );
            }
        } else if self.frequency.is_none() {
            self.frequency = incoming.frequency;
        }
    }
}
