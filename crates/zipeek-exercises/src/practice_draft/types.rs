use crate::character_hunt::Difficulty as HuntDifficulty;
use std::collections::BTreeSet;
use zipeek_core::{HskLevel, HskSystem, practice::ExerciseVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HskBand {
    pub system: u8,
    pub level: u8,
}

impl HskBand {
    pub fn new(system: u8, level: u8) -> Result<Self, DraftError> {
        let supported = HskSystem::from_number(system)
            .is_some_and(|system| HskLevel::try_from((system, i32::from(level))).is_ok());
        if supported {
            Ok(Self { system, level })
        } else {
            Err(DraftError::InvalidHskBand { system, level })
        }
    }

    pub fn from_level(level: HskLevel) -> Self {
        let system = level.system().number();
        Self {
            system,
            level: level.as_i32() as u8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryFilters {
    pub hsk: BTreeSet<HskBand>,
    pub categories: BTreeSet<String>,
    pub parts_of_speech: BTreeSet<String>,
}

impl DictionaryFilters {
    pub fn matches(&self, candidate: &VocabularyCandidate<'_>) -> bool {
        (self.hsk.is_empty() || candidate.hsk.iter().any(|band| self.hsk.contains(band)))
            && (self.categories.is_empty()
                || candidate
                    .categories
                    .iter()
                    .any(|category| self.categories.contains(*category)))
            && (self.parts_of_speech.is_empty()
                || candidate
                    .parts_of_speech
                    .iter()
                    .any(|pos| self.parts_of_speech.contains(*pos)))
    }

    pub fn add_category(&mut self, category: &str) -> Result<bool, DraftError> {
        let normalized = category.trim().to_lowercase();
        if normalized.is_empty()
            || normalized.len() > 64
            || !normalized
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
        {
            return Err(DraftError::EmptyCategory);
        }
        Ok(self.categories.insert(normalized))
    }

    pub fn remove_category(&mut self, category: &str) -> bool {
        self.categories.remove(&category.trim().to_lowercase())
    }
}

impl Default for DictionaryFilters {
    fn default() -> Self {
        Self {
            hsk: [HskBand::new(3, 1).expect("default HSK band is valid")]
                .into_iter()
                .collect(),
            categories: BTreeSet::new(),
            parts_of_speech: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabularyCandidate<'a> {
    pub hsk: &'a [HskBand],
    pub categories: &'a [&'a str],
    pub parts_of_speech: &'a [&'a str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VocabularySource {
    PersonalSet { reference: String },
    Dictionary(DictionaryFilters),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionLength {
    Endless,
    Fixed(u16),
}

impl SessionLength {
    pub fn resumable_rounds(&self) -> Option<usize> {
        match self {
            Self::Fixed(count) if (1..=20).contains(count) => Some(usize::from(*count)),
            Self::Endless | Self::Fixed(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationOptions {
    pub direction: TranslationDirection,
    pub use_word_bank: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationDirection {
    ChineseToEnglish,
    EnglishToChinese,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatchingColumnOrder {
    #[default]
    Standard,
    Reversed,
}

impl MatchingColumnOrder {
    pub const fn query_value(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Reversed => "reversed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeDraft {
    pub exercise_ids: BTreeSet<String>,
    pub vocabulary: VocabularySource,
    pub length: SessionLength,
    pub translation: TranslationOptions,
    pub matching_formats: Vec<ExerciseVariant>,
    pub matching_column_order: MatchingColumnOrder,
    pub multiple_choice_formats: Vec<ExerciseVariant>,
    pub translation_count: u16,
    pub hunt_difficulty: HuntDifficulty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftError {
    NoExercises,
    UnknownExercise(String),
    EmptyPersonalSetReference,
    NoDictionaryFilters,
    InvalidHskBand { system: u8, level: u8 },
    EmptyCategory,
    InvalidSessionLength,
    UnsupportedAssessment(String),
    IncompatibleFormat(ExerciseVariant),
    DuplicateExerciseFormat(ExerciseVariant),
    MissingExerciseFormat(String),
    InvalidTranslationCount(u16),
}

#[cfg(test)]
mod local_session_length_tests {
    use super::SessionLength;

    #[test]
    fn authored_tile_sessions_require_bounded_positive_round_counts() {
        for count in [1, 5, 10, 20] {
            assert_eq!(
                SessionLength::Fixed(count).resumable_rounds(),
                Some(count as usize)
            );
        }
        for count in [0, 21, u16::MAX] {
            assert_eq!(SessionLength::Fixed(count).resumable_rounds(), None);
        }
        assert_eq!(SessionLength::Endless.resumable_rounds(), None);
    }
}
