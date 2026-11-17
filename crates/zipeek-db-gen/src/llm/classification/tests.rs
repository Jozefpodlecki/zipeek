use super::*;
use zipeek_core::WordId;

fn sense() -> SourceSense {
    SourceSense {
        word_id: WordId::try_from(10).unwrap(),
        sense_order: 1,
        gloss: "train ticket".into(),
        source: "hsk-primary",
        source_ref: 20,
        source_gloss_order: 1,
    }
}

#[test]
fn structured_schemas_cover_single_and_multi_sense_without_unbounded_labels() {
    let single = classification_schema(false);
    let batch = classification_schema(true);
    assert_eq!(
        single["properties"]["pos"]["items"]["enum"]
            .as_array()
            .unwrap()
            .len(),
        POS.len()
    );
    assert_eq!(
        single["properties"]["categories"]["items"]["enum"]
            .as_array()
            .unwrap()
            .len(),
        CATEGORIES.len()
    );
    assert_eq!(single["additionalProperties"], false);
    assert_eq!(
        batch["properties"]["results"]["items"]["additionalProperties"],
        false
    );
    assert!(
        batch["properties"]["results"]["items"]["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("word_id"))
    );
    let output = classification_output(false);
    assert!(
        output
            .parse_complete_output(r#"{"pos":["noun"],"categories":["food"]}"#)
            .is_ok()
    );
    assert!(
        output
            .parse_complete_output("Assistant\nLet me reason first")
            .is_err()
    );
    assert!(
        output
            .parse_complete_output(r#"{"pos":["fake"],"categories":[]}"#)
            .is_err()
    );
}

#[test]
fn multi_response_validates_exact_ids_and_taxonomy() {
    use zipeek_core::WordId;
    let id = WordId::try_from(7).unwrap();
    let word = Word {
        id,
        simplified: "超市".into(),
        traditional: None,
        pinyin: "chāo shì".into(),
        pinyin_numeric: None,
        english: None,
        part_of_speech: vec![],
        hsk2_level: None,
        hsk3_level: None,
        frequency: None,
    };
    let sense = SourceSense {
        word_id: id,
        sense_order: 1,
        gloss: "supermarket".into(),
        source: "test",
        source_ref: 1,
        source_gloss_order: 1,
    };
    let expected = vec![(word, sense)];
    let good =
        r#"{"results":[{"word_id":7,"sense_order":1,"pos":["noun"],"categories":["shopping"]}]}"#;
    assert_eq!(
        parse_batch_response(good, &expected).unwrap()[0].pos,
        vec!["noun".to_owned()]
    );
    for bad in [
        r#"{"results":[]}"#,
        r#"{"results":[{"word_id":8,"sense_order":1,"pos":["noun"],"categories":[]}]}"#,
        r#"{"results":[{"word_id":7,"sense_order":1,"pos":["noun"],"categories":[]},{"word_id":7,"sense_order":1,"pos":["noun"],"categories":[]}]}"#,
        r#"{"results":[{"word_id":7,"sense_order":1,"pos":["new_label"],"categories":[]}]}"#,
        r#"{"results":[{"word_id":7,"sense_order":1,"pos":["noun"],"categories":[],"extra":false}]}"#,
    ] {
        assert!(parse_batch_response(bad, &expected).is_err(), "{bad}");
    }
}

#[test]
fn model_output_rejects_unknown_and_duplicate_labels() {
    assert!(parse_model_response(r#"{"pos":["noun"],"categories":["travel"]}"#).is_ok());
    assert!(parse_model_response(r#"{"pos":["silly"],"categories":[]}"#).is_err());
    assert!(parse_model_response(r#"{"pos":["noun","noun"],"categories":[]}"#).is_err());
    assert!(parse_model_response(r#"{"pos":[],"categories":["food"]}"#).is_err());
    assert!(parse_model_response(r#"{"pos":["unclassified","noun"],"categories":[]}"#).is_err());
    assert!(parse_model_response("```json\n{}\n```").is_err());
}

#[test]
fn classification_instructions_are_separate_from_entry_data() {
    let id = WordId::try_from(10).unwrap();
    let word = Word {
        id,
        simplified: "车票".into(),
        traditional: None,
        pinyin: "chē piào".into(),
        pinyin_numeric: None,
        english: Some("ticket".into()),
        part_of_speech: vec![],
        hsk2_level: None,
        hsk3_level: None,
        frequency: None,
    };
    let instructions = system_prompt();
    let data = prompt(&word, &sense());
    assert!(instructions.contains("noun"));
    assert!(instructions.contains("travel"));
    assert!(instructions.contains("categories may be empty"));
    assert!(!instructions.contains("车票"));
    assert!(data.contains("车票"));
    assert!(data.contains("train ticket"));
    assert!(!data.contains("Allowed category codes"));
}

#[test]
fn input_mutation_invalidates_cache() {
    let original = sense();
    let record = CachedClassification {
        word_id: 10,
        sense_order: 1,
        input_digest: digest(&original),
        model: "local".into(),
        prompt_version: PROMPT_VERSION.into(),
        taxonomy_version: TAXONOMY_VERSION,
        labels: parse_model_response(r#"{"pos":["noun"],"categories":["travel"]}"#).unwrap(),
    };
    record.validate(&original).unwrap();
    let mut old_prompt = record.clone();
    old_prompt.prompt_version = "sense-classification-v2".into();
    assert!(old_prompt.validate(&original).is_err());
    let mut changed = original;
    changed.gloss = "different gloss".into();
    assert!(record.validate(&changed).is_err());
}
