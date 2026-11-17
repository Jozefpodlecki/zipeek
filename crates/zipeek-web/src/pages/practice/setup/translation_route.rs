use zipeek_exercises::practice_draft::{TranslationDirection, VocabularySource};

pub(super) fn source(vocabulary: &VocabularySource) -> String {
    match vocabulary {
        VocabularySource::PersonalSet { reference } => format!("saved:{reference}"),
        VocabularySource::Dictionary(filters) => {
            crate::services::practice_filter_source::encode(filters)
        }
    }
}

pub(super) const fn direction(direction: TranslationDirection) -> &'static str {
    match direction {
        TranslationDirection::ChineseToEnglish => "zh-to-en",
        TranslationDirection::EnglishToChinese => "en-to-zh",
        TranslationDirection::Mixed => "mixed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_direction_is_encoded_without_losing_the_choice() {
        assert_eq!(
            direction(TranslationDirection::ChineseToEnglish),
            "zh-to-en"
        );
        assert_eq!(
            direction(TranslationDirection::EnglishToChinese),
            "en-to-zh"
        );
        assert_eq!(direction(TranslationDirection::Mixed), "mixed");
    }
}
