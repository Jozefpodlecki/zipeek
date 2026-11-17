use super::*;

fn word(id: i64, simplified: &str, pinyin: &str, english: Option<&str>) -> Word {
    Word {
        id: WordId::try_from(id).unwrap(),
        simplified: simplified.to_owned(),
        traditional: None,
        pinyin: pinyin.to_owned(),
        pinyin_numeric: None,
        english: english.map(str::to_owned),
        part_of_speech: Vec::new(),
        hsk2_level: None,
        hsk3_level: None,
        frequency: None,
    }
}

#[test]
fn only_usable_vocabulary_is_presented() {
    let words = vec![
        word(1, "生产", "shēngchǎn", Some("to produce|manufacture")),
        word(2, "生产", "shēngchǎn", Some("production")),
        word(3, "制造", "zhìzào", Some("TO PRODUCE")),
        word(4, "空", "", Some("empty")),
        word(5, "未知", "wèizhī", None),
        word(6, " ", "kōng", Some("blank")),
    ];
    let filtered = distinct_eligible_words(words, true);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].simplified, "生产");
    assert_eq!(meaning(&filtered[0]), Some("to produce"));
}

#[test]
fn identifiers_are_validated_without_panics() {
    assert!(parse_word_ids("").is_err());
    assert!(parse_word_ids("1,,2").is_err());
    assert!(parse_word_ids("not-a-number").is_err());
    assert!(parse_word_ids("-3").is_err());
    assert_eq!(parse_word_ids("1,2,1").unwrap().len(), 2);
    assert!(parse_word_ids(&"1".repeat(16_385)).is_err());
}

#[test]
fn missing_glosses_and_first_gloss_whitespace_are_not_answers() {
    let no_first_gloss = word(1, "词", "cí", Some("  |usable later"));
    assert_eq!(meaning(&no_first_gloss), None);
    let first_gloss = word(2, "词语", "cíyǔ", Some(" expression |term"));
    assert_eq!(meaning(&first_gloss), Some("expression"));
}

#[test]
fn shared_glosses_remain_eligible_outside_matching_exercises() {
    let words = vec![
        word(1, "生产", "shēngchǎn", Some("produce")),
        word(2, "制造", "zhìzào", Some("produce")),
        word(3, "生产", "shēngchǎn", Some("another gloss")),
    ];
    let typing = distinct_eligible_words(words.clone(), false);
    assert_eq!(typing.len(), 2);
    assert_eq!(typing[1].simplified, "制造");
    let mosaics = distinct_eligible_words(words, true);
    assert_eq!(mosaics.len(), 1);
}

#[test]
fn oversized_word_set_selections_are_rejected_before_database_queries() {
    let ids = (1..=1_001)
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    assert!(parse_word_ids(&ids).is_err());
    let allowed = (1..=1_000)
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(parse_word_ids(&allowed).unwrap().len(), 1_000);
}

#[test]
fn mixed_quiz_words_have_unique_visible_meanings() {
    let words = vec![
        word(1, "学习", "xuéxí", Some("to study|learn")),
        word(2, "读书", "dúshū", Some("To Study")),
        word(3, "写字", "xiězì", Some("write characters")),
        word(4, "看书", "kànshū", Some("read books")),
        word(5, "听力", "tīnglì", Some("listening comprehension")),
    ];
    let eligible = distinct_eligible_words(words, true);
    assert_eq!(eligible.len(), 4);
    assert!(!eligible.iter().any(|word| word.simplified == "读书"));
}

#[test]
fn surrounding_hanzi_whitespace_does_not_duplicate_a_visible_choice() {
    let words = vec![
        word(1, "你好", "nǐhǎo", Some("hello")),
        word(2, " 你好 ", "nǐhǎo", Some("greeting")),
        word(3, "谢谢", "xièxie", Some("thanks")),
    ];
    let eligible = distinct_eligible_words(words, false);
    assert_eq!(eligible.len(), 2);
    assert_eq!(eligible[0].simplified, "你好");
}

#[test]
fn ai_generation_preparation_does_not_keep_unreadable_words() {
    let words = vec![
        word(1, "苹果", "píngguǒ", Some("apple")),
        word(2, "香蕉", "", Some("banana")),
        word(3, "梨", "lí", Some(" ")),
        word(4, "桃", "táo", Some("peach")),
    ];
    let prepared = distinct_eligible_words(words, false);
    assert_eq!(prepared.len(), 2);
    assert_eq!(prepared[0].simplified, "苹果");
    assert_eq!(prepared[1].simplified, "桃");
}

#[test]
fn builder_prompt_displays_one_synonymous_gloss_without_mutating_dictionary_data() {
    let entry = word(1, "不仅仅", "bù jǐn jǐn", Some("not only; not just|merely"));
    assert_eq!(concise_prompt_meaning(&entry), Some("not only"));
    assert_eq!(meaning(&entry), Some("not only; not just"));
    assert_eq!(entry.english.as_deref(), Some("not only; not just|merely"));
}
