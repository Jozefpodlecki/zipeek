use super::*;

#[test]
fn quick_exercise_selection_preserves_other_defaults() {
    let original = Rc::new(PracticeSetupFlow::with_source(None));
    assert!(original.draft.exercise_ids.is_empty());
    let updated = original.reduce(PracticeSetupAction::ToggleExercise("matching"));
    assert!(updated.draft.has_exercise("matching"));
    assert!(!updated.draft.has_exercise("multiple-choice"));
}

#[test]
fn quick_path_filters_advanced_exercises_without_reselecting_defaults() {
    let initial = Rc::new(PracticeSetupFlow::with_source(None));
    let empty = initial;
    assert!(empty.draft.exercise_ids.is_empty());
    let selected = empty.reduce(PracticeSetupAction::SelectPath(false));
    assert!(selected.draft.exercise_ids.is_empty());
}

#[test]
fn rapid_hsk_actions_read_current_reducer_state() {
    let initial = Rc::new(PracticeSetupFlow::with_source(None));
    let first = initial.reduce(PracticeSetupAction::ToggleHskBand(
        HskBand::new(2, 2).expect("valid band"),
    ));
    let second = first.reduce(PracticeSetupAction::ToggleHskBand(
        HskBand::new(2, 3).expect("valid band"),
    ));
    if let VocabularySource::Dictionary(filters) = &second.draft.vocabulary {
        assert!(
            filters
                .hsk
                .contains(&HskBand::new(2, 2).expect("valid band"))
        );
        assert!(
            filters
                .hsk
                .contains(&HskBand::new(2, 3).expect("valid band"))
        );
    }
}

#[test]
fn matching_pair_count_is_validated_and_retained() {
    let initial = Rc::new(PracticeSetupFlow::with_source(None));
    let seven = initial.reduce(PracticeSetupAction::SetMatchingPairs(7));
    assert_eq!(seven.exercise_options.matching_pairs, 7);
    let invalid = seven.reduce(PracticeSetupAction::SetMatchingPairs(8));
    assert_eq!(invalid.exercise_options.matching_pairs, 7);
}

#[test]
fn configuration_tabs_are_selected_only_from_enabled_exercises() {
    let initial = Rc::new(PracticeSetupFlow::with_source(None));
    let unchanged = initial
        .clone()
        .reduce(PracticeSetupAction::SelectTab("matching"));
    assert!(unchanged == initial);

    let matching = initial.reduce(PracticeSetupAction::ToggleExercise("matching"));
    assert_eq!(matching.active_tab, "matching");
    let invalid = matching
        .clone()
        .reduce(PracticeSetupAction::SelectTab("translation"));
    assert!(invalid == matching);

    let pinyin = matching.reduce(PracticeSetupAction::ToggleExercise("type-pinyin"));
    assert_eq!(pinyin.active_tab, "type-pinyin");
    let back = pinyin.reduce(PracticeSetupAction::SelectTab("matching"));
    assert_eq!(back.active_tab, "matching");
    let removed = back.reduce(PracticeSetupAction::ToggleExercise("matching"));
    assert_eq!(removed.active_tab, "type-pinyin");
}

#[test]
fn exercise_settings_remain_independent_when_switching_configuration_tab() {
    let initial = Rc::new(PracticeSetupFlow::with_source(None));
    let matching = initial.reduce(PracticeSetupAction::ToggleExercise("matching"));
    let matching = matching.reduce(PracticeSetupAction::SetMatchingPairs(6));
    let pinyin = matching.reduce(PracticeSetupAction::ToggleExercise("type-pinyin"));
    let pinyin = pinyin.reduce(PracticeSetupAction::SetTypePinyinHint(true));
    let matching = pinyin.reduce(PracticeSetupAction::SelectTab("matching"));
    assert_eq!(matching.exercise_options.matching_pairs, 6);
    assert!(matching.exercise_options.type_pinyin.show_hint);
}

#[test]
fn transformation_setup_rejects_unavailable_patterns_and_supports_all_answer_modes() {
    use zipeek_exercises::sentence_transformation::AnswerMode;
    let initial = std::rc::Rc::new(PracticeSetupFlow::with_source(None));
    let selected = initial.reduce(PracticeSetupAction::SetTransformationPattern(2));
    assert_eq!(
        selected
            .exercise_options
            .sentence_transformation
            .pattern_index,
        2
    );
    let invalid = selected.reduce(PracticeSetupAction::SetTransformationPattern(99));
    assert_eq!(
        invalid
            .exercise_options
            .sentence_transformation
            .pattern_index,
        2
    );
    let both = invalid.reduce(PracticeSetupAction::SetTransformationAnswerMode(
        AnswerMode::Both,
    ));
    assert_eq!(
        both.exercise_options.sentence_transformation.answer_mode,
        AnswerMode::Both
    );
}

#[test]
fn write_hanzi_guidance_and_animation_are_independent_options() {
    use crate::models::WritingGuidance;
    let initial = Rc::new(PracticeSetupFlow::with_source(None));
    assert_eq!(
        initial.exercise_options.write_hanzi.guidance,
        WritingGuidance::StartCue
    );
    assert!(initial.exercise_options.write_hanzi.animation);
    let configured = initial.reduce(PracticeSetupAction::SetWritingGuidance(
        WritingGuidance::None,
    ));
    let configured = configured.reduce(PracticeSetupAction::SetWritingAnimation(false));
    assert_eq!(
        configured.exercise_options.write_hanzi.guidance,
        WritingGuidance::None
    );
    assert!(!configured.exercise_options.write_hanzi.animation);
    let configured = configured.reduce(PracticeSetupAction::SetWritingGuidance(
        WritingGuidance::FullGuide,
    ));
    assert_eq!(
        configured.exercise_options.write_hanzi.guidance,
        WritingGuidance::FullGuide
    );
    assert!(!configured.exercise_options.write_hanzi.animation);
}

#[test]
fn writing_word_prompt_options_are_independent() {
    let initial = Rc::new(PracticeSetupFlow::with_source(None));
    let configured = initial.reduce(PracticeSetupAction::SetWritingPinyin(false));
    let configured = configured.reduce(PracticeSetupAction::SetWritingMeaning(false));
    let configured = configured.reduce(PracticeSetupAction::SetWritingAudio(false));
    let options = configured.exercise_options.write_hanzi;
    assert!(!options.show_pinyin);
    assert!(!options.show_meaning);
    assert!(!options.playable_audio);
    assert!(options.animation);
    assert_eq!(options.guidance, crate::models::WritingGuidance::StartCue);
}

#[test]
fn writing_assistance_configuration_is_independent_and_defaults_to_three() {
    use crate::models::StrokeAssistance;
    let state = Rc::new(PracticeSetupFlow::with_source(None));
    assert_eq!(
        state.exercise_options.write_hanzi.assistance,
        StrokeAssistance::AfterThree
    );
    let state = state.reduce(PracticeSetupAction::SetWritingAssistance(
        StrokeAssistance::AfterFive,
    ));
    assert_eq!(
        state.exercise_options.write_hanzi.assistance,
        StrokeAssistance::AfterFive
    );
    let state = state.reduce(PracticeSetupAction::SetWritingAssistance(
        StrokeAssistance::Off,
    ));
    assert_eq!(
        state.exercise_options.write_hanzi.assistance,
        StrokeAssistance::Off
    );
    assert!(state.exercise_options.write_hanzi.animation);
}

#[test]
fn listening_configuration_is_independent_from_writing() {
    use crate::models::ListeningHintVisibility;
    let initial = Rc::new(PracticeSetupFlow::with_source(None));
    let configured = initial.reduce(PracticeSetupAction::SetListeningPinyin(
        ListeningHintVisibility::Always,
    ));
    let configured = configured.reduce(PracticeSetupAction::SetListeningMeaning(
        ListeningHintVisibility::Never,
    ));
    let configured = configured.reduce(PracticeSetupAction::SetListeningAutoplay(true));
    assert_eq!(
        configured.exercise_options.listening_choice.pinyin,
        ListeningHintVisibility::Always
    );
    assert_eq!(
        configured.exercise_options.listening_choice.meaning,
        ListeningHintVisibility::Never
    );
    assert!(configured.exercise_options.listening_choice.autoplay);
    assert!(configured.exercise_options.write_hanzi.playable_audio);
}
