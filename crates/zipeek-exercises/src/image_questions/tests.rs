use std::collections::BTreeSet;

use super::{ImageQuestionError, ImageQuestionMode, VisualWord, prepare};

fn words() -> Vec<VisualWord> {
    let fixtures = [
        ("猫", "cat"),
        ("狗", "dog"),
        ("苹果", "apple"),
        ("自行车", "bicycle"),
        ("汽车", "car"),
        ("房子", "house"),
        ("花", "flower"),
        ("树", "tree"),
    ];
    fixtures
        .into_iter()
        .enumerate()
        .map(|(index, (hanzi, english))| VisualWord {
            id: index as i32 + 1,
            hanzi: hanzi.to_owned(),
            english: english.to_owned(),
        })
        .collect()
}

#[test]
fn one_of_three_is_reproducible_and_grades_single_selection() {
    let mut words = words();
    let question = prepare(&words, ImageQuestionMode::OneOfThree, 42).unwrap();
    words.reverse();
    assert_eq!(
        question,
        prepare(&words, ImageQuestionMode::OneOfThree, 42).unwrap()
    );
    assert_eq!(question.options.len(), 3);
    assert_eq!(question.correct_ids.len(), 1);
    assert_eq!(question.grade(&question.correct_ids), Ok(true));
    assert_eq!(
        question.grade(&BTreeSet::new()),
        Err(ImageQuestionError::InvalidSelection)
    );
    let wrong = question
        .options
        .iter()
        .find(|option| !question.correct_ids.contains(&option.id))
        .unwrap();
    assert_eq!(question.grade(&BTreeSet::from([wrong.id])), Ok(false));
    assert_eq!(
        question.grade(&BTreeSet::from([
            wrong.id,
            *question.correct_ids.first().unwrap()
        ])),
        Err(ImageQuestionError::InvalidSelection)
    );
}

#[test]
fn multi_select_requires_every_correct_word_and_no_distractors() {
    let question = prepare(&words(), ImageQuestionMode::SelectAll, 9).unwrap();
    assert_eq!(question.correct_ids.len(), 2);
    assert_eq!(question.grade(&question.correct_ids), Ok(true));
    let one = BTreeSet::from([*question.correct_ids.first().unwrap()]);
    assert_eq!(question.grade(&one), Ok(false));
    let extra = question
        .options
        .iter()
        .find(|option| !question.correct_ids.contains(&option.id))
        .unwrap()
        .id;
    let mut over_selected = question.correct_ids.clone();
    over_selected.insert(extra);
    assert_eq!(question.grade(&over_selected), Ok(false));
    assert_eq!(
        question.grade(&BTreeSet::from([999])),
        Err(ImageQuestionError::InvalidSelection)
    );
}

#[test]
fn rejects_undersized_and_duplicate_pools() {
    assert_eq!(
        prepare(&words()[..2], ImageQuestionMode::OneOfThree, 1),
        Err(ImageQuestionError::NotEnoughVisualWords)
    );
    let mut pool = words();
    pool.push(pool[0].clone());
    assert_eq!(
        prepare(&pool, ImageQuestionMode::SelectAll, 1),
        Err(ImageQuestionError::DuplicateWordId)
    );
}

#[test]
fn refuses_ambiguous_glosses_and_requires_distinct_visual_choices() {
    let mut pool = words();
    pool[0].english = "classifier for animals".to_owned();
    pool[1].english = "cat".to_owned();
    pool[2].english = "CAT".to_owned();
    pool[3].english = "object 3".to_owned();
    pool[4].english = "variant of a noun".to_owned();
    pool[5].english = "surname Smith".to_owned();
    let question = prepare(&pool, ImageQuestionMode::OneOfThree, 3).unwrap();
    assert!(
        question
            .options
            .iter()
            .all(|option| option.english != "object 3")
    );
    assert!(
        question
            .options
            .iter()
            .all(|option| !option.english.starts_with("classifier"))
    );
    assert_eq!(
        question
            .options
            .iter()
            .filter(|option| option.english.eq_ignore_ascii_case("cat"))
            .count(),
        1
    );
    assert_eq!(
        prepare(&pool, ImageQuestionMode::SelectAll, 3),
        Err(ImageQuestionError::NotEnoughVisualWords)
    );
}

#[test]
fn generated_prompt_only_names_correct_visual_subjects() {
    let question = prepare(&words(), ImageQuestionMode::SelectAll, 23).unwrap();
    for option in &question.options {
        if question.correct_ids.contains(&option.id) {
            assert!(question.prompt.contains(&option.english));
        }
    }
    assert!(question.prompt.ends_with("no text"));
}

#[test]
fn duplicate_identifier_is_rejected_even_if_one_gloss_is_ineligible() {
    let mut pool = words();
    let mut duplicate = pool[0].clone();
    duplicate.english = "classifier for cats".to_owned();
    pool.push(duplicate);
    assert_eq!(
        prepare(&pool, ImageQuestionMode::OneOfThree, 11),
        Err(ImageQuestionError::DuplicateWordId)
    );
}

#[test]
fn corrupted_generated_question_is_rejected_before_grading() {
    let mut question = prepare(&words(), ImageQuestionMode::OneOfThree, 3).unwrap();
    assert!(question.validate().is_ok());
    question.correct_ids = BTreeSet::from([999]);
    assert_eq!(
        question.validate(),
        Err(ImageQuestionError::InvalidQuestion)
    );
    assert_eq!(
        question.grade(&BTreeSet::from([999])),
        Err(ImageQuestionError::InvalidQuestion)
    );
}

#[test]
fn corrupted_choice_text_is_rejected_even_if_answer_ids_are_valid() {
    let mut question = prepare(&words(), ImageQuestionMode::OneOfThree, 3).unwrap();
    question.options[0].english = "description".repeat(30);
    assert_eq!(
        question.validate(),
        Err(ImageQuestionError::InvalidQuestion)
    );
    let mut question = prepare(&words(), ImageQuestionMode::OneOfThree, 3).unwrap();
    question.options[0].hanzi = "字".repeat(37);
    assert_eq!(
        question.validate(),
        Err(ImageQuestionError::InvalidQuestion)
    );
}

#[test]
fn ineligible_overlong_hanzi_cannot_enter_image_question() {
    let mut candidates = words();
    candidates[0].hanzi = "词".repeat(40);
    let question = prepare(&candidates, ImageQuestionMode::OneOfThree, 11).unwrap();
    assert!(!question.options.iter().any(|choice| choice.id == 1));
}
