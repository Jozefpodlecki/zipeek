#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenseQualifier {
    pub kind: &'static str,
    pub code: &'static str,
    pub annotation: String,
}

pub fn extract_qualifiers(gloss: &str) -> (String, Vec<SenseQualifier>) {
    let mut remaining = gloss.trim();
    let mut qualifiers = Vec::new();
    while let Some(close) = remaining.strip_prefix('(').and_then(|text| text.find(')')) {
        let marker = &remaining[..close + 2];
        let qualifier = match marker {
            "(Tw)" | "(Taiwan)" => SenseQualifier {
                kind: "region",
                code: "taiwan",
                annotation: marker.to_owned(),
            },
            "(loanword)" => SenseQualifier {
                kind: "usage",
                code: "loanword",
                annotation: marker.to_owned(),
            },
            "(fig.)" => SenseQualifier {
                kind: "usage",
                code: "figurative",
                annotation: marker.to_owned(),
            },
            "(arch.)" | "(archaic)" => SenseQualifier {
                kind: "register",
                code: "archaic",
                annotation: marker.to_owned(),
            },
            "(lit.)" | "(literary)" => SenseQualifier {
                kind: "register",
                code: "literary",
                annotation: marker.to_owned(),
            },
            "(slang)" => SenseQualifier {
                kind: "register",
                code: "slang",
                annotation: marker.to_owned(),
            },
            "(colloquial)" => SenseQualifier {
                kind: "register",
                code: "colloquial",
                annotation: marker.to_owned(),
            },
            _ => break,
        };
        if !qualifiers.iter().any(|existing: &SenseQualifier| {
            existing.kind == qualifier.kind && existing.code == qualifier.code
        }) {
            qualifiers.push(qualifier);
        }
        remaining = remaining[marker.len()..].trim_start();
    }
    if remaining.is_empty() {
        (gloss.to_owned(), Vec::new())
    } else {
        (remaining.to_owned(), qualifiers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unknown_annotations_and_original_text() {
        assert_eq!(
            extract_qualifiers("(biology) cellular growth"),
            ("(biology) cellular growth".to_owned(), vec![])
        );
        assert_eq!(
            extract_qualifiers("button (of a shirt)"),
            ("button (of a shirt)".to_owned(), vec![])
        );
    }

    #[test]
    fn extracts_only_explicit_prefixes() {
        let (display, tags) = extract_qualifiers("(Tw) (loanword) code");
        assert_eq!(display, "code");
        assert_eq!(
            tags.iter()
                .map(|tag| (tag.kind, tag.code))
                .collect::<Vec<_>>(),
            vec![("region", "taiwan"), ("usage", "loanword")]
        );
    }

    #[test]
    fn repeated_region_annotations_do_not_create_duplicate_keys() {
        let (display, tags) = extract_qualifiers("(Tw) (Taiwan) code");
        assert_eq!(display, "code");
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn extracts_known_registers_only_when_explicit() {
        let (display, tags) = extract_qualifiers("(lit.) (slang) term (colloquial)");
        assert_eq!(display, "term (colloquial)");
        assert_eq!(
            tags.iter().map(|tag| tag.code).collect::<Vec<_>>(),
            vec!["literary", "slang"]
        );
    }

    #[test]
    fn preserves_unrecognized_suffix() {
        let (display, tags) = extract_qualifiers("(Tw) (unspecified) code");
        assert_eq!(display, "(unspecified) code");
        assert_eq!(tags.len(), 1);
    }
}
