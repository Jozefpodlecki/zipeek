use super::{
    ImportDiagnostic,
    diagnostics::{ImportDiagnosticKind as Kind, record},
    source::SourceForm,
};

pub(super) fn glosses(
    form: &SourceForm,
    entry: usize,
    simplified: &str,
    diagnostics: &mut Vec<ImportDiagnostic>,
) -> Option<String> {
    serialize_glosses(
        form.meanings.iter().map(String::as_str),
        entry,
        simplified,
        diagnostics,
    )
}

pub(super) fn serialize_glosses<'a>(
    glosses: impl IntoIterator<Item = &'a str>,
    entry: usize,
    simplified: &str,
    diagnostics: &mut Vec<ImportDiagnostic>,
) -> Option<String> {
    let values: Vec<_> = glosses
        .into_iter()
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
