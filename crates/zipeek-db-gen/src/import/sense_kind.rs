#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SenseKind {
    Lexical,
    UsageReference,
    CrossReference,
}

impl SenseKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Lexical => "lexical",
            Self::UsageReference => "usage_reference",
            Self::CrossReference => "cross_reference",
        }
    }
}

pub(super) fn classify(gloss: &str, relation_only: bool) -> SenseKind {
    if relation_only {
        return SenseKind::CrossReference;
    }
    if gloss.trim().to_ascii_lowercase().starts_with("used in ") {
        SenseKind::UsageReference
    } else {
        SenseKind::Lexical
    }
}

#[cfg(test)]
mod tests {
    use super::{SenseKind, classify};

    #[test]
    fn reference_only_readings_are_not_lexical_senses() {
        assert_eq!(
            classify("used in 上聲|上声[shang3 sheng1]", false),
            SenseKind::UsageReference
        );
        assert_eq!(
            classify("used in 丁丁[zheng1 zheng1]", false),
            SenseKind::UsageReference
        );
        assert_eq!(
            classify("(bound form) up; upper; above; previous", false),
            SenseKind::Lexical
        );
        assert_eq!(
            classify("to attend (class or university)", false),
            SenseKind::Lexical
        );
        assert_eq!(classify("variant of 上", true), SenseKind::CrossReference);
    }
}
