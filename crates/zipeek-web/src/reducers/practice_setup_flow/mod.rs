use std::rc::Rc;

mod exercise;
mod navigation;
mod vocabulary;

use yew::Reducible;
use zipeek_core::practice::ExerciseVariant;
use zipeek_exercises::character_hunt::Difficulty as HuntDifficulty;
use zipeek_exercises::practice_draft::{
    HskBand, MatchingColumnOrder, PracticeDraft, SessionLength, TranslationDirection,
    VocabularySource,
};

use crate::{
    components::practice::setup::practice_flow_navigation::PracticeStep,
    models::{
        AnswerPolicy, AudioPreference, ListeningHintVisibility, PracticeExerciseOptions,
        WritingGuidance,
    },
};

pub(crate) const QUICK_EXERCISES: [&str; 8] = [
    "write-hanzi",
    "listening-choice",
    "matching",
    "multiple-choice",
    "type-pinyin",
    "character-builder",
    "word-mosaics",
    "character-hunt",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExerciseStage {
    Choose,
    Configure,
}

#[derive(Clone, PartialEq)]
pub(crate) struct PracticeSetupFlow {
    pub draft: PracticeDraft,
    pub hsk_standard: u8,
    pub step: PracticeStep,
    pub furthest: PracticeStep,
    pub exercise_stage: ExerciseStage,
    pub advanced: bool,
    pub active_tab: &'static str,
    pub start_failed: bool,
    pub exercise_options: PracticeExerciseOptions,
}

pub(crate) enum PracticeSetupAction {
    SelectPath(bool),
    LoadProfile {
        draft: PracticeDraft,
        options: PracticeExerciseOptions,
        advanced: bool,
    },
    SelectStep(PracticeStep),
    SelectExerciseStage(ExerciseStage),
    Back,
    Continue,
    ToggleExercise(&'static str),
    SelectTab(&'static str),
    SetVocabulary(VocabularySource),
    ToggleHskBand(HskBand),
    SelectHskStandard(u8),
    ToggleFormat(ExerciseVariant),
    SetMatchingColumnOrder(MatchingColumnOrder),
    SetMatchingPairs(u8),
    SetPassageSentences(u8),
    SetDialogueTurns(u8),
    SetNarrativeGrammar(u8),
    SetMatchingAiGenerated(bool),
    SetMatchingShowPinyin(bool),
    SetWritingGuidance(WritingGuidance),
    SetWritingAnimation(bool),
    SetWritingAssistance(crate::models::StrokeAssistance),
    SetWritingPinyin(bool),
    SetWritingMeaning(bool),
    SetWritingAudio(bool),
    SetListeningPinyin(ListeningHintVisibility),
    SetListeningMeaning(ListeningHintVisibility),
    SetListeningAutoplay(bool),
    SetTransformationPattern(u8),
    SetTransformationAnswerMode(zipeek_exercises::sentence_transformation::AnswerMode),
    SetLength(SessionLength),
    SetTranslationCount(u16),
    SetTranslationDirection(TranslationDirection),
    SetHuntDifficulty(HuntDifficulty),
    StartFailed(bool),
    SetCharacterBuilderRetry(bool),
    SetTypePinyinRetry(bool),
    SetCharacterBuilderAudio(bool),
    SetCharacterBuilderAutoCheck(bool),
    SetSentenceBuilderAutoCheck(bool),
    SetSentenceBuilderRetry(bool),
    SetTypePinyinHint(bool),
}

impl PracticeSetupFlow {
    fn preferred_configuration_tab(&self) -> &'static str {
        self.draft
            .configuration_tabs()
            .into_iter()
            .find(|tab| *tab != "general")
            .unwrap_or("matching")
    }

    pub fn with_source(source: Option<String>) -> Self {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        draft.length = SessionLength::Fixed(5);
        if let Some(reference) = source
            .as_deref()
            .and_then(|value| value.strip_prefix("saved:"))
        {
            if reference
                .parse::<crate::services::word_sets::WordSetId>()
                .is_ok()
            {
                draft.vocabulary = VocabularySource::PersonalSet {
                    reference: reference.to_owned(),
                };
            }
        }
        Self {
            draft,
            hsk_standard: 3,
            step: PracticeStep::Path,
            furthest: PracticeStep::Path,
            exercise_stage: ExerciseStage::Choose,
            advanced: false,
            active_tab: "matching",
            start_failed: false,
            exercise_options: PracticeExerciseOptions::default(),
        }
    }
}

impl Reducible for PracticeSetupFlow {
    type Action = PracticeSetupAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        next.start_failed = false;
        match action {
            PracticeSetupAction::LoadProfile {
                draft,
                options,
                advanced,
            } => {
                next.hsk_standard = match &draft.vocabulary {
                    VocabularySource::Dictionary(filters) => {
                        let two = filters.hsk.iter().any(|band| band.system == 2);
                        let three = filters.hsk.iter().any(|band| band.system == 3);
                        if two && three {
                            0
                        } else if two {
                            2
                        } else {
                            3
                        }
                    }
                    _ => 3,
                };
                next.draft = draft;
                next.exercise_options = options;
                next.advanced = advanced;
                next.step = PracticeStep::Summary;
                next.furthest = PracticeStep::Summary;
                next.exercise_stage = ExerciseStage::Choose;
                next.active_tab = next.preferred_configuration_tab();
            }
            PracticeSetupAction::SelectPath(advanced) => {
                if !advanced {
                    next.draft
                        .exercise_ids
                        .retain(|id| QUICK_EXERCISES.contains(&id.as_str()));
                    if matches!(next.draft.length, SessionLength::Endless)
                        && !next.draft.supports_endless_session()
                    {
                        next.draft.length = SessionLength::Fixed(5);
                    }
                    next.active_tab = next.preferred_configuration_tab();
                }
                next.advanced = advanced;
                next.step = PracticeStep::Exercises;
                next.furthest = PracticeStep::Exercises;
                next.exercise_stage = ExerciseStage::Choose;
            }
            PracticeSetupAction::SelectStep(step) => {
                if !navigation::select_step(&mut next, step) {
                    return self;
                }
            }
            PracticeSetupAction::SelectExerciseStage(stage) => {
                if !navigation::select_exercise_stage(&mut next, stage) {
                    return self;
                }
            }
            PracticeSetupAction::Back => navigation::back(&mut next),
            PracticeSetupAction::Continue => {
                if !navigation::continue_to_next_step(&mut next) {
                    return self;
                }
            }
            PracticeSetupAction::ToggleExercise(id) => {
                if zipeek_exercises::definition::definition(id).is_none()
                    || (!next.advanced && !QUICK_EXERCISES.contains(&id))
                {
                    return self;
                }
                if let Err(error) = next.draft.toggle_exercise(id) {
                    log::error!("Could not toggle practice exercise: {error:?}");
                    return self;
                }
                if matches!(next.draft.length, SessionLength::Endless)
                    && !next.draft.supports_endless_session()
                {
                    next.draft.length = SessionLength::Fixed(5);
                }
                if next.draft.has_exercise(id) {
                    next.active_tab = exercise::configuration_tab(id);
                } else if !next.draft.configuration_tabs().contains(&next.active_tab) {
                    next.active_tab = next.preferred_configuration_tab();
                }
            }
            PracticeSetupAction::SelectTab(tab) => {
                if !next.draft.configuration_tabs().contains(&tab) {
                    return self;
                }
                next.active_tab = tab;
            }
            PracticeSetupAction::SetVocabulary(source) => next.draft.vocabulary = source,
            PracticeSetupAction::ToggleHskBand(band) => {
                vocabulary::toggle_hsk_band(&mut next.draft, band);
            }
            PracticeSetupAction::SelectHskStandard(system) => {
                if !matches!(system, 0 | 2 | 3) {
                    return self;
                }
                if system != 0 && !vocabulary::select_hsk_standard(&mut next.draft, system) {
                    return self;
                }
                next.hsk_standard = system;
            }
            PracticeSetupAction::ToggleFormat(variant) => {
                let enabled = !next.draft.selected_formats().contains(&variant);
                if let Err(error) = next.draft.set_format(variant, enabled) {
                    log::error!("Could not update question format: {error:?}");
                    return self;
                }
            }
            PracticeSetupAction::SetMatchingColumnOrder(order) => {
                next.draft.matching_column_order = order;
            }
            PracticeSetupAction::SetPassageSentences(count) => {
                if (5..=10).contains(&count) {
                    next.exercise_options.narrative.passage_sentences = count;
                }
            }
            PracticeSetupAction::SetNarrativeGrammar(index) => {
                if index == 0
                    || zipeek_exercises::sentence_transformation::item_for_seed(0, index).is_some()
                {
                    next.exercise_options.narrative.grammar_pattern_index = index;
                }
            }
            PracticeSetupAction::SetDialogueTurns(count) => {
                if (5..=10).contains(&count) {
                    next.exercise_options.narrative.dialogue_turns = count;
                }
            }
            PracticeSetupAction::SetMatchingAiGenerated(enabled) => {
                next.exercise_options.matching_ai_generated = enabled;
            }
            PracticeSetupAction::SetMatchingShowPinyin(enabled) => {
                next.exercise_options.matching_show_pinyin = enabled;
            }
            PracticeSetupAction::SetMatchingPairs(count) => {
                if (2..=7).contains(&count) {
                    next.exercise_options.matching_pairs = count;
                }
            }
            PracticeSetupAction::SetWritingGuidance(guidance) => {
                next.exercise_options.write_hanzi.guidance = guidance;
            }
            PracticeSetupAction::SetWritingAssistance(assistance) => {
                next.exercise_options.write_hanzi.assistance = assistance;
            }
            PracticeSetupAction::SetWritingAnimation(animation) => {
                next.exercise_options.write_hanzi.animation = animation;
            }
            PracticeSetupAction::SetWritingPinyin(show) => {
                next.exercise_options.write_hanzi.show_pinyin = show;
            }
            PracticeSetupAction::SetWritingMeaning(show) => {
                next.exercise_options.write_hanzi.show_meaning = show;
            }
            PracticeSetupAction::SetWritingAudio(enabled) => {
                next.exercise_options.write_hanzi.playable_audio = enabled;
            }
            PracticeSetupAction::SetListeningPinyin(visibility) => {
                next.exercise_options.listening_choice.pinyin = visibility;
            }
            PracticeSetupAction::SetListeningMeaning(visibility) => {
                next.exercise_options.listening_choice.meaning = visibility;
            }
            PracticeSetupAction::SetListeningAutoplay(enabled) => {
                next.exercise_options.listening_choice.autoplay = enabled;
            }
            PracticeSetupAction::SetTransformationPattern(index) => {
                if usize::from(index) <= zipeek_exercises::sentence_transformation::ITEMS.len() {
                    next.exercise_options.sentence_transformation.pattern_index = index;
                }
            }
            PracticeSetupAction::SetTransformationAnswerMode(mode) => {
                next.exercise_options.sentence_transformation.answer_mode = mode;
            }
            PracticeSetupAction::SetLength(length) => {
                if matches!(length, SessionLength::Endless)
                    && !next.draft.supports_endless_session()
                {
                    return self;
                }
                next.draft.length = length;
            }
            PracticeSetupAction::SetTranslationCount(count) => {
                if !matches!(count, 0 | 5 | 10 | 20) {
                    return self;
                }
                next.draft.translation_count = count;
                next.draft.length = if count == 0 {
                    SessionLength::Endless
                } else {
                    SessionLength::Fixed(count)
                };
            }
            PracticeSetupAction::SetTranslationDirection(direction) => {
                next.draft.translation.direction = direction;
            }
            PracticeSetupAction::SetHuntDifficulty(difficulty) => {
                next.draft.hunt_difficulty = difficulty
            }
            PracticeSetupAction::StartFailed(value) => next.start_failed = value,
            PracticeSetupAction::SetCharacterBuilderRetry(value) => {
                next.exercise_options.character_builder.answer_policy = if value {
                    AnswerPolicy::RetryUntilCorrect
                } else {
                    AnswerPolicy::Advance
                };
            }
            PracticeSetupAction::SetTypePinyinRetry(value) => {
                next.exercise_options.type_pinyin.answer_policy = if value {
                    AnswerPolicy::RetryUntilCorrect
                } else {
                    AnswerPolicy::Advance
                };
            }
            PracticeSetupAction::SetSentenceBuilderAutoCheck(value) => {
                next.exercise_options.sentence_builder.automatic_check = value;
            }
            PracticeSetupAction::SetSentenceBuilderRetry(value) => {
                next.exercise_options.sentence_builder.retry_incorrect = value;
            }
            PracticeSetupAction::SetCharacterBuilderAutoCheck(value) => {
                next.exercise_options.character_builder.automatic_check = value;
            }
            PracticeSetupAction::SetTypePinyinHint(value) => {
                next.exercise_options.type_pinyin.show_hint = value;
            }
            PracticeSetupAction::SetCharacterBuilderAudio(value) => {
                next.exercise_options.character_builder.audio = if value {
                    AudioPreference::Manual
                } else {
                    AudioPreference::Hidden
                };
            }
        }
        if next == *self { self } else { Rc::new(next) }
    }
}

#[cfg(test)]
mod tests;
