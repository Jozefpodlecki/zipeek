use std::collections::BTreeSet;

use crate::definition::{DEFINITIONS, definition};
use zipeek_core::practice::ExerciseVariant;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HskBand {
    pub system: u8,
    pub level: u8,
}

impl HskBand {
    pub fn new(system: u8, level: u8) -> Result<Self, DraftError> {
        match (system, level) {
            (2, 1..=6) | (3, 1..=7) => Ok(Self { system, level }),
            _ => Err(DraftError::InvalidHskBand { system, level }),
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
        if normalized.is_empty() {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeDraft {
    pub exercise_ids: BTreeSet<String>,
    pub vocabulary: VocabularySource,
    pub length: SessionLength,
    pub translation: TranslationOptions,
    pub matching_formats: Vec<ExerciseVariant>,
    pub multiple_choice_formats: Vec<ExerciseVariant>,
    pub translation_count: u16,
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
    MissingExerciseFormat(String),
    InvalidTranslationCount(u16),
}

impl Default for PracticeDraft {
    fn default() -> Self {
        Self {
            exercise_ids: ["multiple-choice".to_owned()].into_iter().collect(),
            vocabulary: VocabularySource::Dictionary(DictionaryFilters::default()),
            length: SessionLength::Fixed(5),
            matching_formats: vec![ExerciseVariant::EngPinyin],
            multiple_choice_formats: vec![ExerciseVariant::HanziToMeaning],
            translation_count: 5,
            translation: TranslationOptions {
                direction: TranslationDirection::Mixed,
                use_word_bank: true,
            },
        }
    }
}

impl PracticeDraft {
    pub fn toggle_exercise(&mut self, id: &str) -> Result<(), DraftError> {
        if definition(id).is_none() {
            return Err(DraftError::UnknownExercise(id.to_owned()));
        }
        if !self.exercise_ids.remove(id) {
            self.exercise_ids.insert(id.to_owned());
        }
        Ok(())
    }

    pub fn has_exercise(&self, id: &str) -> bool {
        self.exercise_ids.contains(id)
    }

    pub fn set_format(&mut self, variant: ExerciseVariant, enabled: bool) -> Result<(), DraftError> {
        let family = if variant.is_mcq() {
            &mut self.multiple_choice_formats
        } else {
            &mut self.matching_formats
        };
        if enabled {
            if !family.contains(&variant) { family.push(variant); }
        } else {
            family.retain(|saved| *saved != variant);
        }
        Ok(())
    }

    pub fn selected_formats(&self) -> Vec<ExerciseVariant> {
        [
            ExerciseVariant::HanziToMeaning,
            ExerciseVariant::MeaningToHanzi,
            ExerciseVariant::MeaningToPinyin,
            ExerciseVariant::EngPinyin,
            ExerciseVariant::PinyinHanzi,
            ExerciseVariant::HanziEng,
        ].into_iter().filter(|variant| {
            self.matching_formats.contains(variant) || self.multiple_choice_formats.contains(variant)
        }).collect()
    }

    pub fn active_formats(&self) -> Vec<ExerciseVariant> {
        self.selected_formats().into_iter().filter(|variant| {
            if variant.is_mcq() { self.has_exercise("multiple-choice") }
            else { self.has_exercise("matching") }
        }).collect()
    }

    pub fn validate_exercise_settings(&self) -> Result<(), DraftError> {
        for variant in &self.matching_formats {
            if variant.is_mcq() { return Err(DraftError::IncompatibleFormat(*variant)); }
        }
        for variant in &self.multiple_choice_formats {
            if !variant.is_mcq() { return Err(DraftError::IncompatibleFormat(*variant)); }
        }
        if self.has_exercise("matching") && self.matching_formats.is_empty() {
            return Err(DraftError::MissingExerciseFormat("matching".into()));
        }
        if self.has_exercise("multiple-choice") && self.multiple_choice_formats.is_empty() {
            return Err(DraftError::MissingExerciseFormat("multiple-choice".into()));
        }
        if !matches!(self.translation_count, 0 | 5 | 10) {
            return Err(DraftError::InvalidTranslationCount(self.translation_count));
        }
        Ok(())
    }

    pub fn configuration_tabs(&self) -> Vec<&'static str> {
        let mut tabs = vec!["general"];
        for (id, tab) in [
            ("matching", "matching"),
            ("multiple-choice", "multiple-choice"),
            ("sentence-translation", "translation"),
            ("speaking", "speaking"),
            ("listening-choice", "listening"),
        ] {
            if self.has_exercise(id) {
                tabs.push(tab);
            }
        }
        tabs
    }

    pub fn validate(&self) -> Result<(), DraftError> {
        if self.exercise_ids.is_empty() {
            return Err(DraftError::NoExercises);
        }
        for id in &self.exercise_ids {
            if !DEFINITIONS.iter().any(|definition| definition.id == id) {
                return Err(DraftError::UnknownExercise(id.clone()));
            }
        }
        match &self.vocabulary {
            VocabularySource::PersonalSet { reference } if reference.trim().is_empty() => {
                return Err(DraftError::EmptyPersonalSetReference);
            }
            VocabularySource::Dictionary(filters)
                if filters.hsk.is_empty()
                    && filters.categories.is_empty()
                    && filters.parts_of_speech.is_empty() =>
            {
                return Err(DraftError::NoDictionaryFilters);
            }
            VocabularySource::Dictionary(filters) => {
                for band in &filters.hsk {
                    HskBand::new(band.system, band.level)?;
                }
            }
            _ => {}
        }
        self.validate_exercise_settings()?;
        if matches!(self.length, SessionLength::Fixed(0)) {
            return Err(DraftError::InvalidSessionLength);
        }
        Ok(())
    }

    pub fn validate_for_scored_session(&self) -> Result<(), DraftError> {
        self.validate()?;
        for id in &self.exercise_ids {
            if matches!(id.as_str(), "speaking" | "listening-choice") {
                return Err(DraftError::UnsupportedAssessment(id.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_scored() {
        let draft = PracticeDraft::default();
        assert_eq!(draft.validate(), Ok(()));
        assert_eq!(draft.validate_for_scored_session(), Ok(()));
        assert_eq!(draft.configuration_tabs(), ["general", "multiple-choice"]);
    }

    #[test]
    fn exercise_ids_are_validated_without_a_central_kind_enum() {
        let mut draft = PracticeDraft::default();
        assert_eq!(draft.toggle_exercise("unknown"), Err(DraftError::UnknownExercise("unknown".into())));
        draft.toggle_exercise("sentence-translation").unwrap();
        draft.toggle_exercise("speaking").unwrap();
        assert_eq!(draft.configuration_tabs(), ["general", "multiple-choice", "translation", "speaking"]);
        draft.toggle_exercise("sentence-translation").unwrap();
        assert_eq!(draft.configuration_tabs(), ["general", "multiple-choice", "speaking"]);
        draft.toggle_exercise("multiple-choice").unwrap();
        draft.toggle_exercise("speaking").unwrap();
        assert_eq!(draft.validate(), Err(DraftError::NoExercises));
    }

    #[test]
    fn unscored_recording_is_not_treated_as_scored_practice() {
        let mut draft = PracticeDraft::default();
        draft.toggle_exercise("speaking").unwrap();
        assert_eq!(draft.validate(), Ok(()));
        assert_eq!(draft.validate_for_scored_session(), Err(DraftError::UnsupportedAssessment("speaking".into())));
    }

    #[test]
    fn source_choice_is_exclusive_and_invalid_references_fail() {
        let mut draft = PracticeDraft::default();
        draft.vocabulary = VocabularySource::PersonalSet { reference: " ".into() };
        assert_eq!(draft.validate(), Err(DraftError::EmptyPersonalSetReference));
        draft.vocabulary = VocabularySource::PersonalSet { reference: "saved:0".into() };
        assert_eq!(draft.validate(), Ok(()));
        draft.vocabulary = VocabularySource::Dictionary(DictionaryFilters {
            hsk: BTreeSet::new(),
            categories: BTreeSet::new(),
            parts_of_speech: BTreeSet::new(),
        });
        assert_eq!(draft.validate(), Err(DraftError::NoDictionaryFilters));
    }

    #[test]
    fn hsk_systems_do_not_conflate_advanced_levels() {
        assert!(HskBand::new(2, 6).is_ok());
        assert!(HskBand::new(3, 7).is_ok());
        assert!(HskBand::new(2, 7).is_err());
        assert!(HskBand::new(3, 8).is_err());
        assert!(HskBand::new(4, 1).is_err());
    }

    #[test]
    fn multiple_values_are_or_within_group_and_and_across_groups() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.insert(HskBand::new(3, 6).unwrap());
        filters.add_category(" Travel ").unwrap();
        filters.add_category("shopping").unwrap();
        filters.parts_of_speech.insert("noun".into());
        let matches = |hsk: Vec<HskBand>, categories: Vec<&str>, pos: Vec<&str>| {
            filters.matches(&VocabularyCandidate {
                hsk: &hsk,
                categories: &categories,
                parts_of_speech: &pos,
            })
        };
        assert!(matches(vec![HskBand::new(3, 6).unwrap()], vec!["travel"], vec!["noun"]));
        assert!(matches(vec![HskBand::new(3, 1).unwrap()], vec!["shopping"], vec!["noun"]));
        assert!(!matches(vec![HskBand::new(2, 6).unwrap()], vec!["travel"], vec!["noun"]));
        assert!(!matches(vec![HskBand::new(3, 6).unwrap()], vec!["food"], vec!["noun"]));
        assert!(!matches(vec![HskBand::new(3, 6).unwrap()], vec!["travel"], vec!["verb"]));
    }

    #[test]
    fn category_chips_are_normalized_deduplicated_and_removable() {
        let mut filters = DictionaryFilters::default();
        assert_eq!(filters.add_category(" Communication "), Ok(true));
        assert_eq!(filters.add_category("communication"), Ok(false));
        assert_eq!(filters.add_category("  "), Err(DraftError::EmptyCategory));
        assert!(filters.remove_category(" COMMUNICATION "));
        assert!(!filters.remove_category("communication"));
    }

    #[test]
    fn exercise_settings_are_independent_and_retained_on_deselection() {
        let mut draft = PracticeDraft::default();
        draft.toggle_exercise("matching").unwrap();
        draft.set_format(ExerciseVariant::PinyinHanzi, true).unwrap();
        draft.set_format(ExerciseVariant::HanziToMeaning, false).unwrap();
        assert_eq!(draft.validate_exercise_settings(), Err(DraftError::MissingExerciseFormat("multiple-choice".into())));
        draft.set_format(ExerciseVariant::MeaningToHanzi, true).unwrap();
        assert_eq!(draft.validate_exercise_settings(), Ok(()));
        draft.toggle_exercise("matching").unwrap();
        assert!(!draft.active_formats().contains(&ExerciseVariant::PinyinHanzi));
        draft.toggle_exercise("matching").unwrap();
        assert!(draft.active_formats().contains(&ExerciseVariant::PinyinHanzi));
    }

    #[test]
    fn invalid_family_formats_and_translation_counts_rejected() {
        let mut draft = PracticeDraft::default();
        draft.matching_formats.push(ExerciseVariant::MeaningToHanzi);
        assert_eq!(draft.validate(), Err(DraftError::IncompatibleFormat(ExerciseVariant::MeaningToHanzi)));
        draft.matching_formats.retain(|v| *v != ExerciseVariant::MeaningToHanzi);
        draft.translation_count = 7;
        assert_eq!(draft.validate(), Err(DraftError::InvalidTranslationCount(7)));
    }

    #[test]
    fn fixed_session_requires_a_positive_count() {
        let mut draft = PracticeDraft::default();
        draft.length = SessionLength::Fixed(0);
        assert_eq!(draft.validate(), Err(DraftError::InvalidSessionLength));
        draft.length = SessionLength::Endless;
        assert_eq!(draft.validate(), Ok(()));
    }
}
