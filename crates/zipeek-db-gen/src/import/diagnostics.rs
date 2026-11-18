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
        ImportDiagnosticKind::RepeatedReading | ImportDiagnosticKind::GlossPipeReplacement => {
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
    fn routine_diagnostics_are_retained_without_warning_level_logging() {
        let mut diagnostics = Vec::new();
        record(
            &mut diagnostics,
            ImportDiagnosticKind::RepeatedReading,
            5,
            "子",
            "duplicate",
        );
        record(
            &mut diagnostics,
            ImportDiagnosticKind::GlossPipeReplacement,
            6,
            "骆驼",
            "normalized separator",
        );
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].detail, "duplicate");
        assert_eq!(diagnostics[1].kind, ImportDiagnosticKind::GlossPipeReplacement);
        assert_eq!(diagnostics[1].detail, "normalized separator");
    }
}
