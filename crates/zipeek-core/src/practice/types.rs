use crate::{Hsk3Level, HskLevel, PartOfSpeech};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExerciseMechanic {
    Matching,
    MultipleChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestionFormat {
    EnglishToPinyin,
    PinyinToHanzi,
    HanziToMeaning,
    MeaningToHanzi,
    MeaningToPinyin,
}

impl QuestionFormat {
    pub const fn label(self) -> &'static str {
        match self {
            Self::EnglishToPinyin => "Meaning → Pinyin",
            Self::PinyinToHanzi => "Pinyin → Hanzi",
            Self::HanziToMeaning => "Hanzi → Meaning",
            Self::MeaningToHanzi => "Meaning → Hanzi",
            Self::MeaningToPinyin => "Meaning → Pinyin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExerciseVariant {
    MatchingEnglishPinyin,
    MatchingPinyinHanzi,
    MatchingHanziMeaning,
    MatchingAudioHanzi,
    MatchingAudioMeaning,
    MultipleChoiceHanziMeaning,
    MultipleChoiceMeaningHanzi,
    MultipleChoiceMeaningPinyin,
}

impl ExerciseVariant {
    pub const ALL: [Self; 8] = [
        Self::MatchingEnglishPinyin,
        Self::MatchingPinyinHanzi,
        Self::MatchingHanziMeaning,
        Self::MatchingAudioHanzi,
        Self::MatchingAudioMeaning,
        Self::MultipleChoiceHanziMeaning,
        Self::MultipleChoiceMeaningHanzi,
        Self::MultipleChoiceMeaningPinyin,
    ];

    pub const CANONICAL: [Self; 8] = [
        Self::MultipleChoiceHanziMeaning,
        Self::MultipleChoiceMeaningHanzi,
        Self::MultipleChoiceMeaningPinyin,
        Self::MatchingEnglishPinyin,
        Self::MatchingPinyinHanzi,
        Self::MatchingHanziMeaning,
        Self::MatchingAudioHanzi,
        Self::MatchingAudioMeaning,
    ];

    pub const fn mechanic(self) -> ExerciseMechanic {
        match self {
            Self::MatchingEnglishPinyin
            | Self::MatchingPinyinHanzi
            | Self::MatchingHanziMeaning
            | Self::MatchingAudioHanzi
            | Self::MatchingAudioMeaning => ExerciseMechanic::Matching,
            Self::MultipleChoiceHanziMeaning
            | Self::MultipleChoiceMeaningHanzi
            | Self::MultipleChoiceMeaningPinyin => ExerciseMechanic::MultipleChoice,
        }
    }

    pub const fn format(self) -> QuestionFormat {
        match self {
            Self::MatchingEnglishPinyin => QuestionFormat::EnglishToPinyin,
            Self::MatchingPinyinHanzi => QuestionFormat::PinyinToHanzi,
            Self::MatchingHanziMeaning | Self::MatchingAudioHanzi | Self::MatchingAudioMeaning | Self::MultipleChoiceHanziMeaning => {
                QuestionFormat::HanziToMeaning
            }
            Self::MultipleChoiceMeaningHanzi => QuestionFormat::MeaningToHanzi,
            Self::MultipleChoiceMeaningPinyin => QuestionFormat::MeaningToPinyin,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::MatchingAudioHanzi => "Audio → Hanzi",
            Self::MatchingAudioMeaning => "Audio → Meaning",
            _ => self.format().label(),
        }
    }

    pub const fn seed_token(self) -> &'static str {
        match self {
            Self::MatchingEnglishPinyin => "eng-pinyin",
            Self::MatchingPinyinHanzi => "pinyin-hanzi",
            Self::MatchingHanziMeaning => "hanzi-eng",
            Self::MatchingAudioHanzi => "audio-hanzi",
            Self::MatchingAudioMeaning => "audio-meaning",
            Self::MultipleChoiceHanziMeaning => "hanzi-to-meaning",
            Self::MultipleChoiceMeaningHanzi => "meaning-to-hanzi",
            Self::MultipleChoiceMeaningPinyin => "meaning-to-pinyin",
        }
    }

    pub fn from_seed_token(token: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|variant| variant.seed_token() == token)
    }

    pub const fn is_mcq(self) -> bool {
        matches!(self.mechanic(), ExerciseMechanic::MultipleChoice)
    }

    pub const fn requires_english(self) -> bool {
        !matches!(self.format(), QuestionFormat::PinyinToHanzi)
    }
}

#[cfg(test)]
mod taxonomy_tests {
    use super::*;

    #[test]
    fn both_mechanics_share_the_same_hanzi_meaning_format() {
        assert_eq!(
            ExerciseVariant::MatchingHanziMeaning.format(),
            QuestionFormat::HanziToMeaning
        );
        assert_eq!(
            ExerciseVariant::MultipleChoiceHanziMeaning.format(),
            QuestionFormat::HanziToMeaning
        );
        assert_ne!(
            ExerciseVariant::MatchingHanziMeaning.mechanic(),
            ExerciseVariant::MultipleChoiceHanziMeaning.mechanic()
        );
        assert_eq!(
            ExerciseVariant::MatchingHanziMeaning.label(),
            ExerciseVariant::MultipleChoiceHanziMeaning.label()
        );
    }

    #[test]
    fn every_format_token_round_trips_without_accepting_unknown_formats() {
        for variant in ExerciseVariant::ALL {
            assert_eq!(
                ExerciseVariant::from_seed_token(variant.seed_token()),
                Some(variant)
            );
        }
        for invalid in ["", "hanzi-eng,eng-pinyin", "not-a-format", "HANZI-ENG"] {
            assert_eq!(ExerciseVariant::from_seed_token(invalid), None);
        }
    }

    #[test]
    fn every_variant_has_a_distinct_stable_token_and_valid_mechanic() {
        let tokens = ExerciseVariant::ALL.map(ExerciseVariant::seed_token);
        for (index, token) in tokens.iter().enumerate() {
            assert!(!token.is_empty());
            assert!(!tokens[..index].contains(token));
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FixedRounds {
    Five,
    #[default]
    Ten,
    Twenty,
}

impl FixedRounds {
    pub const fn count(self) -> u8 {
        match self {
            Self::Five => 5,
            Self::Ten => 10,
            Self::Twenty => 20,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PracticeMode {
    #[default]
    Endless,
    Fixed(FixedRounds),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeFilter {
    pub level: HskLevel,
    pub part_of_speech: Vec<PartOfSpeech>,
}

impl Default for PracticeFilter {
    fn default() -> Self {
        Self {
            level: HskLevel::Hsk3(Hsk3Level::One),
            part_of_speech: Vec::new(),
        }
    }
}

impl PracticeFilter {
    pub fn pos_tokens(&self) -> String {
        let mut tokens: Vec<_> = self
            .part_of_speech
            .iter()
            .map(|value| value.as_str())
            .collect();
        tokens.sort_unstable();
        tokens.dedup();
        tokens.join(",")
    }
}
