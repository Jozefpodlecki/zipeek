use zipeek_core::PartOfSpeech;

pub(super) fn normalize(code: &str) -> Result<PartOfSpeech, zipeek_core::PartOfSpeechError> {
    match code.to_ascii_lowercase().as_str() {
        "qv" | "qt" => Ok(PartOfSpeech::Classifier),
        "mq" => Ok(PartOfSpeech::Numeral),
        "cc" => Ok(PartOfSpeech::Conjunction),
        _ => PartOfSpeech::from_source_code(code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_source_subtypes_are_normalized() {
        for (source, expected) in [
            ("qv", PartOfSpeech::Classifier),
            ("qt", PartOfSpeech::Classifier),
            ("mq", PartOfSpeech::Numeral),
            ("cc", PartOfSpeech::Conjunction),
            ("N", PartOfSpeech::Noun),
        ] {
            assert_eq!(normalize(source).unwrap(), expected);
        }
    }

    #[test]
    fn unexpected_codes_are_not_silently_mapped() {
        assert!(normalize("unknown_code").is_err());
    }
}
