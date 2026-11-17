use super::parse_model_response;

#[test]
fn empty_or_whitespace_only_generation_reports_actionable_error() {
    for text in ["", " ", "\n\t"] {
        let error = parse_model_response(text).unwrap_err().to_string();
        assert!(error.contains("no final text"), "{error}");
    }
}

#[test]
fn valid_json_after_stream_aggregation_is_accepted() {
    let chunks = ["{\"pos\":[\"noun\"],", "\"categories\":[\"food\"]}"];
    let result = parse_model_response(&chunks.concat()).unwrap();
    assert_eq!(result.pos, vec!["noun".to_owned()]);
    assert_eq!(result.categories, vec!["food".to_owned()]);
}
