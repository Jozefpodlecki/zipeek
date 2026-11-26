use super::*;

fn words() -> Vec<QuizWord> {
    (1..=10)
        .map(|id| QuizWord {
            id,
            hanzi: format!("字{id}"),
            meaning: format!("meaning {id}"),
            pinyin: format!("pin{id}"),
        })
        .collect()
}

#[test]
fn stable_preparation_is_independent_of_input_order() {
    let first = prepare(&words(), 17).unwrap();
    let mut reversed = words();
    reversed.reverse();
    assert_eq!(first, prepare(&reversed, 17).unwrap());
    assert_eq!(first.words.len(), 4);
    assert_eq!(first.grade(first.answer_id), Ok(true));
    assert!(first.words.iter().any(|word| word.id == first.answer_id));
}

#[test]
fn rejects_duplicate_ids_and_ambiguous_visible_answers() {
    let mut duplicate = words();
    duplicate.push(duplicate[0].clone());
    assert_eq!(prepare(&duplicate, 2), Err(QuizError::DuplicateIds));
    let mut ambiguous = words()[..4].to_vec();
    ambiguous[1].meaning = ambiguous[0].meaning.clone();
    assert_eq!(
        prepare(&ambiguous, 2),
        Err(QuizError::InsufficientDistinctWords)
    );
    ambiguous = words()[..4].to_vec();
    ambiguous[1].hanzi = ambiguous[0].hanzi.clone();
    assert_eq!(
        prepare(&ambiguous, 2),
        Err(QuizError::InsufficientDistinctWords)
    );
}

#[test]
fn visible_round_choices_are_normalized_before_display() {
    let mut candidates = words();
    candidates[0].hanzi = "  字1 ".into();
    candidates[0].meaning = "  meaning 1 ".into();
    let round = prepare(&candidates, 19).unwrap();
    assert!(
        round
            .words
            .iter()
            .all(|word| { word.hanzi == word.hanzi.trim() && word.meaning == word.meaning.trim() })
    );
}

#[test]
fn grading_rejects_unknown_choices() {
    let round = prepare(&words(), 14).unwrap();
    assert_eq!(round.grade(-1), Err(QuizError::InvalidSelection));
    assert_eq!(
        round.pair_is_correct(-1, round.answer_id),
        Err(QuizError::InvalidSelection)
    );
    for word in &round.words {
        assert_eq!(round.pair_is_correct(word.id, word.id), Ok(true));
    }
}

#[test]
fn oversized_vocabulary_cannot_become_an_unreadable_quiz_option() {
    let mut candidates = words()[..4].to_vec();
    candidates[0].meaning = "x".repeat(161);
    assert_eq!(
        prepare(&candidates, 1),
        Err(QuizError::InsufficientDistinctWords)
    );
    candidates[0].meaning = "valid".into();
    assert!(prepare(&candidates, 1).is_ok());
}

#[test]
fn every_quiz_format_presents_the_approved_prompt_and_option_fields() {
    use zipeek_core::practice::ExerciseVariant;
    for format in ExerciseVariant::ALL {
        let round = prepare_for_format(&words(), 41, format).unwrap();
        let answer = round.answer();
        let prompt = round.prompt_for(answer);
        let option = round.option_for(answer);
        let (expected_prompt, expected_option) = match format {
            ExerciseVariant::MatchingEnglishPinyin => (&answer.meaning, &answer.pinyin),
            ExerciseVariant::MatchingPinyinHanzi => (&answer.pinyin, &answer.hanzi),
            ExerciseVariant::MatchingHanziMeaning => (&answer.hanzi, &answer.meaning),
            ExerciseVariant::MatchingAudioHanzi => (&answer.hanzi, &answer.hanzi),
            ExerciseVariant::MatchingAudioMeaning => (&answer.hanzi, &answer.meaning),
            ExerciseVariant::MultipleChoiceHanziMeaning => (&answer.hanzi, &answer.meaning),
            ExerciseVariant::MultipleChoiceMeaningHanzi => (&answer.meaning, &answer.hanzi),
            ExerciseVariant::MultipleChoiceMeaningPinyin => (&answer.meaning, &answer.pinyin),
        };
        assert_eq!(prompt, expected_prompt);
        assert_eq!(option, expected_option);
        assert_eq!(round.grade(answer.id), Ok(true));
    }
}

#[test]
fn ambiguous_pinyin_options_are_not_playable_in_pinyin_formats() {
    use zipeek_core::practice::ExerciseVariant;
    let mut candidates = words()[..4].to_vec();
    candidates[0].pinyin = candidates[1].pinyin.clone();
    assert_eq!(
        prepare_for_format(&candidates, 8, ExerciseVariant::MultipleChoiceMeaningPinyin),
        Err(QuizError::InsufficientDistinctWords)
    );
    assert!(
        prepare_for_format(&candidates, 8, ExerciseVariant::MultipleChoiceMeaningHanzi).is_ok()
    );
}

#[test]
fn pair_attempts_distinguish_errors_repeats_and_wrong_choices() {
    let round = prepare(&words(), 8).unwrap();
    let left = round.words[0].id;
    let right = round.words[1].id;
    let mut matched = BTreeSet::new();
    assert_eq!(
        round.evaluate_pair(&matched, left, right),
        Ok(PairAttempt::Incorrect)
    );
    assert_eq!(
        round.evaluate_pair(&matched, left, left),
        Ok(PairAttempt::Correct)
    );
    assert_eq!(
        round.evaluate_pair(&matched, -1, right),
        Err(QuizError::InvalidSelection)
    );
    assert_eq!(
        round.evaluate_pair(&matched, left, -1),
        Err(QuizError::InvalidSelection)
    );
    matched.insert(left);
    assert_eq!(
        round.evaluate_pair(&matched, left, left),
        Ok(PairAttempt::AlreadyMatched)
    );
    assert_eq!(
        round.evaluate_pair(&matched, left, right),
        Ok(PairAttempt::AlreadyMatched)
    );
    assert_eq!(
        round.evaluate_pair(&matched, right, right),
        Ok(PairAttempt::Correct)
    );
}

#[test]
fn matching_round_size_is_configurable_but_multiple_choice_is_four() {
    for count in 2..=7 {
        let result = prepare_for_format_with_count(
            &words(),
            41,
            zipeek_core::practice::ExerciseVariant::MatchingHanziMeaning,
            count,
        )
        .unwrap();
        assert_eq!(result.words.len(), count);
        assert_eq!(
            result,
            prepare_for_format_with_count(
                &words(),
                41,
                zipeek_core::practice::ExerciseVariant::MatchingHanziMeaning,
                count
            )
            .unwrap()
        );
    }
    for count in [0, 1, 8] {
        assert_eq!(
            prepare_for_format_with_count(
                &words(),
                41,
                zipeek_core::practice::ExerciseVariant::MatchingHanziMeaning,
                count
            ),
            Err(QuizError::InvalidSelection)
        );
    }
    assert_eq!(
        prepare_for_format_with_count(
            &words(),
            41,
            zipeek_core::practice::ExerciseVariant::MultipleChoiceHanziMeaning,
            5
        ),
        Err(QuizError::InvalidSelection)
    );
}

#[test]
fn audio_matching_preserves_pair_identity_across_both_answer_modes() {
    use zipeek_core::practice::ExerciseVariant;
    for mode in [ExerciseVariant::MatchingAudioHanzi, ExerciseVariant::MatchingAudioMeaning] {
        let first = prepare_for_format_with_count(&words(), 127, mode, 5).unwrap();
        let second = prepare_for_format_with_count(&words(), 127, mode, 5).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.words.len(), 5);
        for word in &first.words {
            assert_eq!(first.prompt_for(word), word.hanzi);
            assert_eq!(first.pair_is_correct(word.id, word.id), Ok(true));
            assert_eq!(first.option_for(word), if mode == ExerciseVariant::MatchingAudioHanzi {
                word.hanzi.as_str()
            } else {
                word.meaning.as_str()
            });
        }
    }
}
