use super::{
    CedictIndex, HskImport, ImportError,
    diagnostics::{ImportDiagnosticKind as Kind, record},
    glosses::serialize_glosses,
    senses::{SourceSense, source_glosses},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FallbackSummary {
    pub filled: usize,
    pub unresolved: usize,
    pub skipped_variants: usize,
}

pub fn apply_cedict_fallback(
    imported: &mut HskImport,
    bytes: &[u8],
) -> Result<FallbackSummary, ImportError> {
    let mut summary = FallbackSummary::default();
    if imported.words.iter().all(|word| word.english.is_some()) {
        return Ok(summary);
    }
    let index = CedictIndex::parse(bytes)?;
    for word in &mut imported.words {
        if word.english.is_some() {
            continue;
        }
        let pinyin = word
            .pinyin_numeric
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&word.pinyin);
        let word_id = word.id.as_i32() as usize;
        for candidate in index.lookup(&word.simplified, pinyin) {
            if candidate
                .glosses
                .iter()
                .any(|gloss| gloss.to_ascii_lowercase().starts_with("variant of"))
            {
                summary.skipped_variants += 1;
                record(
                    &mut imported.diagnostics,
                    Kind::FallbackVariantSkipped,
                    word_id,
                    &word.simplified,
                    format!(
                        "word id {word_id}: ignored CC-CEDICT variant entry at line {}",
                        candidate.line
                    ),
                );
                continue;
            }
            let english = serialize_glosses(
                candidate.glosses.iter().map(String::as_str),
                word_id,
                &word.simplified,
                &mut imported.diagnostics,
            );
            if let Some(english) = english {
                word.english = Some(english);
                for (index, (source_gloss_order, gloss)) in source_glosses(&candidate.glosses).into_iter().enumerate() {
                    imported.senses.push(SourceSense {
                        word_id: word.id,
                        sense_order: (index + 1) as i32,
                        gloss,
                        source: "cc-cedict-fallback",
                        source_ref: candidate.line as i32,
                        source_gloss_order,
                    });
                }
                summary.filled += 1;
                record(
                    &mut imported.diagnostics,
                    Kind::FallbackApplied,
                    word_id,
                    &word.simplified,
                    format!(
                        "word id {word_id}: CC-CEDICT line {}, reading {pinyin:?}",
                        candidate.line
                    ),
                );
                break;
            }
        }
        if word.english.is_none() {
            summary.unresolved += 1;
            record(
                &mut imported.diagnostics,
                Kind::FallbackUnavailable,
                word_id,
                &word.simplified,
                format!(
                    "word id {word_id}: no usable CC-CEDICT gloss for {pinyin:?}; retained NULL"
                ),
            );
        }
    }
    tracing::info!(
        filled = summary.filled,
        unresolved = summary.unresolved,
        skipped_variants = summary.skipped_variants,
        "CC-CEDICT fallback complete"
    );
    Ok(summary)
}
