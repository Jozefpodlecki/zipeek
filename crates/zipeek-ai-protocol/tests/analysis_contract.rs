use zipeek_ai_protocol::sentence_analysis::{json_schema, messages, parse};

#[test]
fn complex_sentence_fixtures_follow_contract_in_both_directions() {
    for raw in [
        include_str!("../../zipeek-ai-client/tests/fixtures/complex_zh_en.json"),
        include_str!("../../zipeek-ai-client/tests/fixtures/complex_en_zh.json"),
    ] {
        let fixture: serde_json::Value = serde_json::from_str(raw).unwrap();
        let source = fixture["source"].as_str().unwrap();
        let direction = fixture["direction"].as_str().unwrap();
        let analysis = parse(&fixture["analysis"].to_string(), source, direction).unwrap();
        assert!(!analysis.grammar.is_empty());
        assert!(!analysis.vocabulary.is_empty());
        let system = messages(source, direction)[0]["content"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(system.contains("Hanyu Pinyin"));
        assert!(system.contains("exactly ONE JSON object"));
        assert!(system.contains("ALL TEN keys"));
    }
}

#[test]
fn strict_deserialization_rejects_missing_arrays_and_invented_properties() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../zipeek-ai-client/tests/fixtures/complex_zh_en.json"
    ))
    .unwrap();
    let mut missing = fixture["analysis"].clone();
    missing.as_object_mut().unwrap().remove("nuance");
    assert!(
        parse(
            &missing.to_string(),
            fixture["source"].as_str().unwrap(),
            "zh-en"
        )
        .is_err()
    );

    let mut invented = fixture["analysis"].clone();
    invented["unverified_evidence"] = serde_json::json!("fabricated");
    assert!(
        parse(
            &invented.to_string(),
            fixture["source"].as_str().unwrap(),
            "zh-en"
        )
        .is_err()
    );

    let mut ungrounded = fixture["analysis"].clone();
    ungrounded["grammar"][0]["example_fragment"] = serde_json::json!("从来没有出现在原文中的词组");
    assert!(
        parse(
            &ungrounded.to_string(),
            fixture["source"].as_str().unwrap(),
            "zh-en"
        )
        .is_err()
    );
}

#[test]
fn json_schema_and_model_agree_on_required_fields() {
    let schema = json_schema();
    let required = schema["required"].as_array().unwrap();
    let fields = schema["properties"].as_object().unwrap();
    assert_eq!(required.len(), fields.len());
    for field in required {
        assert!(fields.contains_key(field.as_str().unwrap()));
    }
}
