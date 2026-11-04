#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ImportDiagnosticKind {
    RejectedEntry,
    RejectedForm,
    UnknownPartOfSpeech,
    UnknownLevel,
    LevelConflict,
    RepeatedReading,
    DisplayDrift,
    FrequencyConflict,
    GlossPipeReplacement,
    MissingEnglish,
    FallbackApplied,
    FallbackUnavailable,
    FallbackVariantSkipped,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ImportDiagnostic {
    pub kind: ImportDiagnosticKind,
    pub entry: usize,
    pub simplified: String,
    pub detail: String,
}

pub(super) fn record(
    diagnostics: &mut Vec<ImportDiagnostic>,
    kind: ImportDiagnosticKind,
    entry: usize,
    simplified: &str,
    detail: impl Into<String>,
) {
    let detail = detail.into();
    match kind {
        ImportDiagnosticKind::RepeatedReading => {
            tracing::debug!(?kind, entry, simplified, detail, "HSK import diagnostic");
        }
        _ => {
            tracing::warn!(?kind, entry, simplified, detail, "HSK import diagnostic");
        }
    }
    diagnostics.push(ImportDiagnostic {
        kind,
        entry,
        simplified: simplified.to_owned(),
        detail,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_reading_diagnostics_are_still_retained() {
        let mut diagnostics = Vec::new();
        record(&mut diagnostics, ImportDiagnosticKind::RepeatedReading, 5, "子", "duplicate");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].detail, "duplicate");
    }
}
