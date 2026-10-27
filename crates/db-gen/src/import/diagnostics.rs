#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ImportDiagnosticKind {
    RejectedEntry,
    RejectedForm,
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
    tracing::warn!(?kind, entry, simplified, detail, "HSK import diagnostic");
    diagnostics.push(ImportDiagnostic {
        kind,
        entry,
        simplified: simplified.to_owned(),
        detail,
    });
}
