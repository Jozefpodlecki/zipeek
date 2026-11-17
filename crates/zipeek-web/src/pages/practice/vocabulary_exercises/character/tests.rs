use super::*;
use zipeek_core::WordId;

fn word(id: i64, hanzi: &str) -> Word {
    Word {
        id: WordId::try_from(id).unwrap(),
        simplified: hanzi.into(),
        traditional: None,
        pinyin: "shēngchǎn".into(),
        pinyin_numeric: None,
        english: Some("production".into()),
        part_of_speech: Vec::new(),
        hsk2_level: None,
        hsk3_level: None,
        frequency: None,
    }
}

#[test]
fn repeated_target_characters_receive_distinct_tile_ids() {
    let bank = builder_bank(&word(1, "生生"), &[word(2, "发展变化")], 11);
    let indices = bank
        .iter()
        .enumerate()
        .filter(|(_, ch)| **ch == '生')
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 2);
    assert_eq!(assembled_answer(&bank, &indices), Some("生生".to_owned()));
}

#[test]
fn selection_never_reuses_a_tile_or_exceeds_capacity() {
    let first = add_tile(&[], 0, 3, 2);
    assert_eq!(first, vec![0]);
    assert_eq!(add_tile(&first, 0, 3, 2), first);
    assert_eq!(add_tile(&first, 3, 3, 2), first);
    let full = add_tile(&first, 1, 3, 2);
    assert_eq!(add_tile(&full, 2, 3, 2), full);
    assert_eq!(remove_tile(&full, 0), vec![1]);
    assert_eq!(remove_tile(&full, 99), full);
}

#[test]
fn invalid_or_duplicate_tile_indices_cannot_form_answers() {
    assert_eq!(assembled_answer(&['生', '产'], &[0, 0]), None);
    assert_eq!(assembled_answer(&['生', '产'], &[8]), None);
    assert_eq!(
        assembled_answer(&['生', '产'], &[0, 1]),
        Some("生产".into())
    );
}

#[test]
fn distractors_do_not_duplicate_existing_characters() {
    let bank = builder_bank(
        &word(1, "产品"),
        &[word(2, "产产品生生生"), word(3, "发展")],
        11,
    );
    assert_eq!(
        bank.iter().filter(|character| **character == '产').count(),
        1
    );
    assert_eq!(
        bank.iter().filter(|character| **character == '生').count(),
        1
    );
    assert!(bank.len() <= 5);
}

#[test]
fn different_instance_seeds_produce_repeatable_but_varied_tile_order() {
    let target = word(1, "生产");
    let candidates = vec![word(2, "发展变化行业")];
    let one = builder_bank(&target, &candidates, 17);
    let again = builder_bank(&target, &candidates, 17);
    let other = builder_bank(&target, &candidates, 42);
    assert_eq!(one, again);
    assert_eq!(one.len(), other.len());
    assert_eq!(
        one.iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        other
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
    );
    assert!(one.contains(&'生') && one.contains(&'产'));
}

#[test]
fn readings_prefer_the_current_word_for_polyphonic_characters() {
    let mut preferred = word(1, "银行");
    preferred.pinyin = "yín háng".into();
    let mut distractor = word(2, "行走");
    distractor.pinyin = "xíng zǒu".into();
    assert_eq!(
        reading_for_character('行', &preferred, &[distractor.clone()]),
        "háng"
    );
    assert_eq!(
        reading_for_character('走', &preferred, &[distractor]),
        "zǒu"
    );
}

#[test]
fn readings_do_not_guess_syllable_boundaries() {
    let mut malformed = word(1, "银行");
    malformed.pinyin = "yínháng".into();
    assert_eq!(reading_for_character('行', &malformed, &[]), "");
}
