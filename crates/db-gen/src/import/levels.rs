use zipeek_core::{Hsk2Level, Hsk3Level};

use super::{
    ImportDiagnostic,
    diagnostics::{ImportDiagnosticKind, record},
};

pub(super) fn memberships(
    tokens: &[String],
    entry: usize,
    simplified: &str,
    diagnostics: &mut Vec<ImportDiagnostic>,
) -> (Option<Hsk2Level>, Option<Hsk3Level>) {
    let mut old = None;
    let mut new = None;
    for token in tokens {
        let old_level = match token.as_str() {
            "old-1" => Some(Hsk2Level::One),
            "old-2" => Some(Hsk2Level::Two),
            "old-3" => Some(Hsk2Level::Three),
            "old-4" => Some(Hsk2Level::Four),
            "old-5" => Some(Hsk2Level::Five),
            "old-6" => Some(Hsk2Level::Six),
            _ => None,
        };
        let new_level = match token.as_str() {
            "new-1" => Some(Hsk3Level::One),
            "new-2" => Some(Hsk3Level::Two),
            "new-3" => Some(Hsk3Level::Three),
            "new-4" => Some(Hsk3Level::Four),
            "new-5" => Some(Hsk3Level::Five),
            "new-6" => Some(Hsk3Level::Six),
            "new-7" | "new-7+" => Some(Hsk3Level::SevenToNine),
            _ => None,
        };
        if old_level.is_none() && new_level.is_none() {
            record(
                diagnostics,
                ImportDiagnosticKind::UnknownLevel,
                entry,
                simplified,
                format!("ignored level token {token:?}"),
            );
        }
        merge(
            &mut old,
            old_level,
            "HSK 2.0",
            entry,
            simplified,
            diagnostics,
        );
        merge(
            &mut new,
            new_level,
            "HSK 3.0",
            entry,
            simplified,
            diagnostics,
        );
    }
    (old, new)
}

pub(super) fn merge<T: Ord + Copy + std::fmt::Debug>(
    current: &mut Option<T>,
    incoming: Option<T>,
    system: &str,
    entry: usize,
    simplified: &str,
    diagnostics: &mut Vec<ImportDiagnostic>,
) {
    if let Some(value) = incoming {
        if let Some(previous) = current {
            if *previous != value {
                record(
                    diagnostics,
                    ImportDiagnosticKind::LevelConflict,
                    entry,
                    simplified,
                    format!("{system}: {previous:?} versus {value:?}; keeping lowest"),
                );
                *previous = (*previous).min(value);
            }
        } else {
            *current = Some(value);
        }
    }
}
