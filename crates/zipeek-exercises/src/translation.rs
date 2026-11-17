#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationDirection {
    ChineseToEnglish,
    EnglishToChinese,
}

impl TranslationDirection {
    pub fn code(self) -> &'static str {
        match self {
            Self::ChineseToEnglish => "zh-to-en",
            Self::EnglishToChinese => "en-to-zh",
        }
    }

    pub fn source_language(self) -> &'static str {
        match self {
            Self::ChineseToEnglish => "Chinese",
            Self::EnglishToChinese => "English",
        }
    }

    pub fn target_language(self) -> &'static str {
        match self {
            Self::ChineseToEnglish => "English",
            Self::EnglishToChinese => "Chinese",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "zh-to-en" => Some(Self::ChineseToEnglish),
            "en-to-zh" => Some(Self::EnglishToChinese),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionSelection {
    Mixed,
    ChineseToEnglish,
    EnglishToChinese,
}

impl DirectionSelection {
    pub fn choose(self, first_is_chinese: bool, exercise_index: u64) -> TranslationDirection {
        match self {
            Self::ChineseToEnglish => TranslationDirection::ChineseToEnglish,
            Self::EnglishToChinese => TranslationDirection::EnglishToChinese,
            Self::Mixed if first_is_chinese == (exercise_index % 2 == 0) => {
                TranslationDirection::ChineseToEnglish
            }
            Self::Mixed => TranslationDirection::EnglishToChinese,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_direction_is_balanced_and_repeatable() {
        let result = (0..6)
            .map(|index| DirectionSelection::Mixed.choose(true, index))
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            vec![
                TranslationDirection::ChineseToEnglish,
                TranslationDirection::EnglishToChinese,
                TranslationDirection::ChineseToEnglish,
                TranslationDirection::EnglishToChinese,
                TranslationDirection::ChineseToEnglish,
                TranslationDirection::EnglishToChinese,
            ]
        );
        assert_eq!(
            DirectionSelection::Mixed.choose(false, 0),
            TranslationDirection::EnglishToChinese
        );
    }

    #[test]
    fn direction_codes_round_trip_without_guessing() {
        for direction in [
            TranslationDirection::ChineseToEnglish,
            TranslationDirection::EnglishToChinese,
        ] {
            assert_eq!(
                TranslationDirection::from_code(direction.code()),
                Some(direction)
            );
            assert_ne!(direction.source_language(), direction.target_language());
        }
        assert_eq!(TranslationDirection::from_code("ZH-TO-EN"), None);
    }

    #[test]
    fn explicit_direction_ignores_round_position() {
        for index in 0..20 {
            assert_eq!(
                DirectionSelection::ChineseToEnglish.choose(false, index),
                TranslationDirection::ChineseToEnglish
            );
            assert_eq!(
                DirectionSelection::EnglishToChinese.choose(true, index),
                TranslationDirection::EnglishToChinese
            );
        }
        assert_eq!(
            TranslationDirection::from_code("en-to-zh"),
            Some(TranslationDirection::EnglishToChinese)
        );
        assert_eq!(TranslationDirection::from_code("invalid"), None);
    }
}
