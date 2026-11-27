use super::{
    DictionaryFilters, DraftError, HskBand, MatchingColumnOrder, PracticeDraft, SessionLength,
    TranslationDirection, TranslationOptions, VocabularySource,
};
use crate::definition::{DEFINITIONS, definition};
use zipeek_core::practice::ExerciseVariant;

impl Default for PracticeDraft {
    fn default() -> Self {
        Self {
            exercise_ids: ["multiple-choice".to_owned()].into_iter().collect(),
            vocabulary: VocabularySource::Dictionary(DictionaryFilters::default()),
            length: SessionLength::Fixed(5),
            matching_formats: vec![ExerciseVariant::MatchingHanziMeaning],
            matching_column_order: MatchingColumnOrder::default(),
            multiple_choice_formats: vec![ExerciseVariant::MultipleChoiceHanziMeaning],
            translation_count: 5,
            hunt_difficulty: crate::character_hunt::Difficulty::Medium,
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

    pub fn supports_endless_session(&self) -> bool {
        !self.exercise_ids.is_empty()
            && self
                .exercise_ids
                .iter()
                .all(|id| crate::definition::supports_resumable_session_host(id))
    }

    pub fn supports_local_vocabulary_session(&self) -> bool {
        !self.exercise_ids.is_empty()
            && self
                .exercise_ids
                .iter()
                .all(|id| crate::definition::supports_resumable_session_host(id))
            && (self.translation_session_only()
                || matches!(self.length, SessionLength::Endless)
                || self.length.resumable_rounds().is_some())
    }

    pub fn translation_session_only(&self) -> bool {
        self.exercise_ids.len() == 1
            && self.has_exercise("sentence-translation")
            && matches!(self.translation_count, 0 | 5 | 10 | 20)
    }

    pub fn playlist_exercises(&self) -> Vec<&'static str> {
        crate::definition::PRACTICE_PLAYLIST_ORDER
            .into_iter()
            .filter(|id| self.has_exercise(id))
            .collect()
    }

    pub fn has_exercise(&self, id: &str) -> bool {
        self.exercise_ids.contains(id)
    }

    pub fn set_format(
        &mut self,
        variant: ExerciseVariant,
        enabled: bool,
    ) -> Result<(), DraftError> {
        let family = if variant.is_mcq() {
            &mut self.multiple_choice_formats
        } else {
            &mut self.matching_formats
        };
        if enabled {
            if !family.contains(&variant) {
                family.push(variant);
            }
        } else {
            family.retain(|saved| *saved != variant);
        }
        Ok(())
    }

    pub fn selected_formats(&self) -> Vec<ExerciseVariant> {
        ExerciseVariant::CANONICAL
            .into_iter()
            .filter(|variant| {
                self.matching_formats.contains(variant)
                    || self.multiple_choice_formats.contains(variant)
            })
            .collect()
    }

    pub fn active_formats(&self) -> Vec<ExerciseVariant> {
        self.selected_formats()
            .into_iter()
            .filter(|variant| {
                if variant.is_mcq() {
                    self.has_exercise("multiple-choice")
                } else {
                    self.has_exercise("matching")
                }
            })
            .collect()
    }

    pub fn validate_exercise_settings(&self) -> Result<(), DraftError> {
        for variant in &self.matching_formats {
            if variant.is_mcq() {
                return Err(DraftError::IncompatibleFormat(*variant));
            }
        }
        for variant in &self.multiple_choice_formats {
            if !variant.is_mcq() {
                return Err(DraftError::IncompatibleFormat(*variant));
            }
        }
        for formats in [&self.matching_formats, &self.multiple_choice_formats] {
            for (index, variant) in formats.iter().enumerate() {
                if formats[index + 1..].contains(variant) {
                    return Err(DraftError::DuplicateExerciseFormat(*variant));
                }
            }
        }
        if self.has_exercise("matching") && self.matching_formats.is_empty() {
            return Err(DraftError::MissingExerciseFormat("matching".into()));
        }
        if self.has_exercise("multiple-choice") && self.multiple_choice_formats.is_empty() {
            return Err(DraftError::MissingExerciseFormat("multiple-choice".into()));
        }
        if !matches!(self.translation_count, 0 | 5 | 10 | 20) {
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
            ("story-builder", "story-builder"),
            ("reading-comprehension", "reading-comprehension"),
            ("conversation-practice", "conversation-practice"),
            ("speaking", "speaking"),
            ("listening-choice", "listening"),
            ("image-vocabulary", "image"),
            ("character-hunt", "hunt"),
            ("type-pinyin", "type-pinyin"),
            ("character-builder", "character-builder"),
            ("word-mosaics", "word-mosaics"),
            ("fill-blank", "fill-blank"),
            ("audio-dictation", "audio-dictation"),
            ("sentence-builder", "sentence-builder"),
            ("sentence-transformation", "sentence-transformation"),
            ("spot-mistake", "spot-mistake"),
            ("write-hanzi", "write-hanzi"),
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
        if matches!(self.length, SessionLength::Fixed(0))
            || (matches!(self.length, SessionLength::Endless)
                && !self.supports_endless_session()
                && !self.translation_session_only())
        {
            return Err(DraftError::InvalidSessionLength);
        }
        Ok(())
    }

    pub fn validate_for_scored_session(&self) -> Result<(), DraftError> {
        self.validate()?;
        for id in &self.exercise_ids {
            if matches!(
                id.as_str(),
                "speaking"
                    | "listening-choice"
                    | "image-vocabulary"
                    | "character-hunt"
                    | "type-pinyin"
                    | "character-builder"
                    | "word-mosaics"
                    | "fill-blank"
                    | "audio-dictation"
                    | "sentence-builder"
                    | "sentence-transformation"
                    | "spot-mistake"
                    | "write-hanzi"
            ) {
                return Err(DraftError::UnsupportedAssessment(id.clone()));
            }
        }
        Ok(())
    }
}
