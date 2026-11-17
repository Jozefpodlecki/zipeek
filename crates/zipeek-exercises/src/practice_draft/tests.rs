use super::*;
use std::collections::BTreeSet;
use zipeek_core::practice::ExerciseVariant;

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
    assert_eq!(
        draft.toggle_exercise("unknown"),
        Err(DraftError::UnknownExercise("unknown".into()))
    );
    draft.toggle_exercise("sentence-translation").unwrap();
    draft.toggle_exercise("speaking").unwrap();
    assert_eq!(
        draft.configuration_tabs(),
        ["general", "multiple-choice", "translation", "speaking"]
    );
    draft.toggle_exercise("sentence-translation").unwrap();
    assert_eq!(
        draft.configuration_tabs(),
        ["general", "multiple-choice", "speaking"]
    );
    draft.toggle_exercise("multiple-choice").unwrap();
    draft.toggle_exercise("speaking").unwrap();
    assert_eq!(draft.validate(), Err(DraftError::NoExercises));
}

#[test]
fn unscored_recording_is_not_treated_as_scored_practice() {
    let mut draft = PracticeDraft::default();
    draft.toggle_exercise("speaking").unwrap();
    assert_eq!(draft.validate(), Ok(()));
    assert_eq!(
        draft.validate_for_scored_session(),
        Err(DraftError::UnsupportedAssessment("speaking".into()))
    );
}

#[test]
fn standalone_image_and_listening_are_valid_but_not_mixed_session_exercises() {
    for exercise in ["image-vocabulary", "listening-choice", "character-hunt"] {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        draft.toggle_exercise(exercise).unwrap();
        assert_eq!(draft.validate(), Ok(()));
        assert_eq!(
            draft.validate_for_scored_session(),
            Err(DraftError::UnsupportedAssessment(exercise.to_owned())),
        );
    }
}

#[test]
fn character_hunt_is_configured_in_the_draft() {
    let mut draft = PracticeDraft::default();
    assert_eq!(
        draft.hunt_difficulty,
        crate::character_hunt::Difficulty::Medium
    );
    draft.toggle_exercise("character-hunt").unwrap();
    assert!(draft.configuration_tabs().contains(&"hunt"));
    draft.hunt_difficulty = crate::character_hunt::Difficulty::Hardcore;
    assert_eq!(draft.hunt_difficulty.size(), 5);
}

#[test]
fn source_choice_is_exclusive_and_invalid_references_fail() {
    let mut draft = PracticeDraft::default();
    draft.vocabulary = VocabularySource::PersonalSet {
        reference: " ".into(),
    };
    assert_eq!(draft.validate(), Err(DraftError::EmptyPersonalSetReference));
    draft.vocabulary = VocabularySource::PersonalSet {
        reference: "saved:0".into(),
    };
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
fn draft_hsk_bands_follow_the_shared_system_catalog() {
    for system in zipeek_core::HskSystem::ALL {
        for level in system.levels() {
            let band = HskBand::from_level(*level);
            assert_eq!(HskBand::new(band.system, band.level), Ok(band));
            assert_eq!(band.level as i32, level.as_i32());
        }
    }
    assert!(HskBand::new(2, 7).is_err());
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
    assert!(matches(
        vec![HskBand::new(3, 6).unwrap()],
        vec!["travel"],
        vec!["noun"]
    ));
    assert!(matches(
        vec![HskBand::new(3, 1).unwrap()],
        vec!["shopping"],
        vec!["noun"]
    ));
    assert!(!matches(
        vec![HskBand::new(2, 6).unwrap()],
        vec!["travel"],
        vec!["noun"]
    ));
    assert!(!matches(
        vec![HskBand::new(3, 6).unwrap()],
        vec!["food"],
        vec!["noun"]
    ));
    assert!(!matches(
        vec![HskBand::new(3, 6).unwrap()],
        vec!["travel"],
        vec!["verb"]
    ));
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
    draft
        .set_format(ExerciseVariant::MatchingPinyinHanzi, true)
        .unwrap();
    draft
        .set_format(ExerciseVariant::MultipleChoiceHanziMeaning, false)
        .unwrap();
    assert_eq!(
        draft.validate_exercise_settings(),
        Err(DraftError::MissingExerciseFormat("multiple-choice".into()))
    );
    draft
        .set_format(ExerciseVariant::MultipleChoiceMeaningHanzi, true)
        .unwrap();
    assert_eq!(draft.validate_exercise_settings(), Ok(()));
    draft.toggle_exercise("matching").unwrap();
    assert!(
        !draft
            .active_formats()
            .contains(&ExerciseVariant::MatchingPinyinHanzi)
    );
    draft.toggle_exercise("matching").unwrap();
    assert!(
        draft
            .active_formats()
            .contains(&ExerciseVariant::MatchingPinyinHanzi)
    );
}

#[test]
fn invalid_family_formats_and_translation_counts_rejected() {
    let mut draft = PracticeDraft::default();
    draft
        .matching_formats
        .push(ExerciseVariant::MultipleChoiceMeaningHanzi);
    assert_eq!(
        draft.validate(),
        Err(DraftError::IncompatibleFormat(
            ExerciseVariant::MultipleChoiceMeaningHanzi
        ))
    );
    draft
        .matching_formats
        .retain(|v| *v != ExerciseVariant::MultipleChoiceMeaningHanzi);
    draft.translation_count = 7;
    assert_eq!(
        draft.validate(),
        Err(DraftError::InvalidTranslationCount(7))
    );
}

#[test]
fn fixed_session_requires_a_positive_count() {
    let mut draft = PracticeDraft::default();
    draft.length = SessionLength::Fixed(0);
    assert_eq!(draft.validate(), Err(DraftError::InvalidSessionLength));
    draft.length = SessionLength::Endless;
    assert_eq!(draft.validate(), Ok(()));
}

#[test]
fn matching_defaults_to_hanzi_meaning_and_preserves_column_order() {
    use super::MatchingColumnOrder;
    let mut draft = PracticeDraft::default();
    assert_eq!(
        draft.matching_formats,
        vec![ExerciseVariant::MatchingHanziMeaning]
    );
    assert_eq!(draft.matching_column_order, MatchingColumnOrder::Standard);
    draft.matching_column_order = MatchingColumnOrder::Reversed;
    assert_eq!(draft.matching_column_order.query_value(), "reversed");
    assert!(draft.validate().is_ok());
}

#[test]
fn vocabulary_exercises_have_distinct_configuration_tabs_and_remain_standalone() {
    for (id, expected_tab) in [
        ("type-pinyin", "type-pinyin"),
        ("character-builder", "character-builder"),
        ("word-mosaics", "word-mosaics"),
        ("fill-blank", "fill-blank"),
        ("audio-dictation", "audio-dictation"),
        ("sentence-builder", "sentence-builder"),
        ("sentence-transformation", "sentence-transformation"),
        ("spot-mistake", "spot-mistake"),
    ] {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        draft.toggle_exercise(id).unwrap();
        assert_eq!(draft.validate(), Ok(()));
        assert_eq!(draft.configuration_tabs(), ["general", expected_tab]);
        assert_eq!(
            draft.validate_for_scored_session(),
            Err(DraftError::UnsupportedAssessment(id.to_owned()))
        );
        draft.toggle_exercise(id).unwrap();
        assert_eq!(draft.validate(), Err(DraftError::NoExercises));
    }
}

#[test]
fn all_exercises_can_coexist_in_a_draft() {
    let mut draft = PracticeDraft::default();
    for id in crate::definition::PRACTICE_PLAYLIST_ORDER {
        if !draft.has_exercise(id) {
            draft.toggle_exercise(id).unwrap();
        }
    }
    assert_eq!(
        draft.playlist_exercises().len(),
        crate::definition::PRACTICE_PLAYLIST_ORDER.len()
    );
}

#[test]
fn local_vocabulary_session_support_requires_fixed_count_and_hosted_types() {
    let mut draft = PracticeDraft::default();
    draft.exercise_ids = [
        "type-pinyin".to_owned(),
        "character-builder".to_owned(),
        "word-mosaics".to_owned(),
    ]
    .into_iter()
    .collect();
    assert!(draft.supports_local_vocabulary_session());
    draft.length = SessionLength::Endless;
    assert!(!draft.supports_local_vocabulary_session());
    draft.length = SessionLength::Fixed(5);
    draft.exercise_ids.insert("sentence-translation".into());
    assert!(!draft.supports_local_vocabulary_session());
}

#[test]
fn resumable_length_rejects_unbounded_and_out_of_range_rounds() {
    assert_eq!(SessionLength::Fixed(5).resumable_rounds(), Some(5));
    assert_eq!(SessionLength::Fixed(20).resumable_rounds(), Some(20));
    assert_eq!(SessionLength::Fixed(0).resumable_rounds(), None);
    assert_eq!(SessionLength::Fixed(21).resumable_rounds(), None);
    assert_eq!(SessionLength::Endless.resumable_rounds(), None);
}

#[test]
fn authored_practice_modes_share_resumable_sessions_without_fake_scoring() {
    let mut draft = PracticeDraft::default();
    draft.exercise_ids.clear();
    for id in [
        "fill-blank",
        "audio-dictation",
        "sentence-builder",
        "sentence-transformation",
        "spot-mistake",
        "type-pinyin",
    ] {
        draft.toggle_exercise(id).unwrap();
    }
    assert!(draft.supports_local_vocabulary_session());
    assert_eq!(draft.playlist_exercises().len(), 6);
    assert_eq!(
        draft.validate_for_scored_session(),
        Err(DraftError::UnsupportedAssessment(
            "audio-dictation".to_owned()
        ))
    );
    assert_eq!(draft.validate(), Ok(()));
    draft.toggle_exercise("matching").unwrap();
    assert!(draft.supports_local_vocabulary_session());
}

#[test]
fn mixed_matching_listening_and_multiple_choice_have_uuid_hosts() {
    let mut draft = PracticeDraft::default();
    draft.exercise_ids.clear();
    for id in [
        "matching",
        "multiple-choice",
        "listening-choice",
        "speaking",
    ] {
        draft.toggle_exercise(id).unwrap();
    }
    assert!(draft.supports_local_vocabulary_session());
    draft.toggle_exercise("sentence-translation").unwrap();
    assert!(draft.supports_local_vocabulary_session());
    draft.toggle_exercise("sentence-translation").unwrap();
    draft.length = SessionLength::Endless;
    assert!(!draft.supports_local_vocabulary_session());
}

#[test]
fn single_fixed_exercise_uses_the_same_uuid_host_as_mixed_sessions() {
    for id in [
        "matching",
        "multiple-choice",
        "listening-choice",
        "speaking",
    ] {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        draft.toggle_exercise(id).unwrap();
        assert!(draft.supports_local_vocabulary_session());
        draft.length = SessionLength::Endless;
        assert!(!draft.supports_local_vocabulary_session());
    }
}

#[test]
fn invalid_dictionary_category_tokens_fail_before_session_persistence() {
    let mut filters = DictionaryFilters::default();
    let too_long = "too-long-".repeat(20);
    for category in [too_long.as_str(), "travel|noun", "a,b", ""] {
        assert!(filters.add_category(category).is_err());
    }
    assert_eq!(filters.add_category(" TRAVEL "), Ok(true));
    assert!(filters.categories.contains("travel"));
}

#[test]
fn character_hunt_can_run_in_a_fixed_resumable_mixed_session() {
    let mut draft = PracticeDraft::default();
    draft.exercise_ids.clear();
    draft.toggle_exercise("character-hunt").unwrap();
    draft.hunt_difficulty = crate::character_hunt::Difficulty::Hardcore;
    assert!(draft.supports_local_vocabulary_session());
    draft.toggle_exercise("matching").unwrap();
    assert!(draft.supports_local_vocabulary_session());
    draft.toggle_exercise("image-vocabulary").unwrap();
    assert!(draft.supports_local_vocabulary_session());
}

#[test]
fn ai_generated_exercises_have_fixed_uuid_launch_paths() {
    for id in ["sentence-translation", "image-vocabulary"] {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        draft.toggle_exercise(id).unwrap();
        assert!(draft.supports_local_vocabulary_session());
        assert_eq!(draft.validate(), Ok(()));
        draft.length = SessionLength::Endless;
        assert!(!draft.supports_local_vocabulary_session());
    }
}

#[test]
fn duplicate_selected_quiz_formats_fail_validation() {
    let mut draft = PracticeDraft::default();
    draft
        .multiple_choice_formats
        .push(ExerciseVariant::MultipleChoiceHanziMeaning);
    assert_eq!(
        draft.validate_exercise_settings(),
        Err(DraftError::DuplicateExerciseFormat(
            ExerciseVariant::MultipleChoiceHanziMeaning
        ))
    );
}

#[test]
fn endless_mode_rejects_exercises_without_a_repeatable_host() {
    let mut draft = PracticeDraft::default();
    draft.length = SessionLength::Endless;
    assert!(draft.supports_endless_session());
    draft.exercise_ids.insert("matching".into());
    assert!(draft.supports_endless_session());
    draft.exercise_ids.insert("fill-blank".into());
    assert!(!draft.supports_endless_session());
    assert_eq!(draft.validate(), Err(DraftError::InvalidSessionLength));
    draft.length = SessionLength::Fixed(5);
    assert_eq!(draft.validate(), Ok(()));
}

#[test]
fn translation_only_persists_its_own_finite_or_endless_count() {
    let mut draft = PracticeDraft::default();
    draft.exercise_ids = ["sentence-translation".to_owned()].into_iter().collect();
    for count in [0, 5, 10] {
        draft.translation_count = count;
        draft.length = SessionLength::Endless;
        assert!(draft.translation_session_only());
        assert!(draft.supports_local_vocabulary_session());
        assert_eq!(draft.validate(), Ok(()));
    }
    draft.toggle_exercise("matching").unwrap();
    assert!(!draft.translation_session_only());
    assert!(!draft.supports_local_vocabulary_session());
}
