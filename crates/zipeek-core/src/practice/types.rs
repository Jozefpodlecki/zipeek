use crate::{Hsk3Level, HskLevel, PartOfSpeech};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExerciseVariant {
    EngPinyin,
    PinyinHanzi,
    HanziEng,
    HanziToMeaning,
    MeaningToHanzi,
    MeaningToPinyin,
}

impl ExerciseVariant {
    pub const fn seed_token(self) -> &'static str {
        match self {
            Self::EngPinyin => "eng-pinyin",
            Self::PinyinHanzi => "pinyin-hanzi",
            Self::HanziEng => "hanzi-eng",
            Self::HanziToMeaning => "hanzi-to-meaning",
            Self::MeaningToHanzi => "meaning-to-hanzi",
            Self::MeaningToPinyin => "meaning-to-pinyin",
        }
    }

    pub const fn is_mcq(self) -> bool {
        matches!(
            self,
            Self::HanziToMeaning | Self::MeaningToHanzi | Self::MeaningToPinyin
        )
    }

    pub const fn requires_english(self) -> bool {
        !matches!(self, Self::PinyinHanzi)
    }

    pub fn route_path(self) -> String {
        format!(
            "/practice/{}{}",
            if self.is_mcq() { "mcq/" } else { "" },
            self.seed_token()
        )
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
