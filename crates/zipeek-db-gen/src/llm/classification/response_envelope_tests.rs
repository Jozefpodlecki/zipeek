use super::*;

#[test]
fn accepts_known_reasoning_and_markdown_wrappers() {
    let wrapped =
        "<think>internal analysis</think>\n```json\n{\"pos\":[\"noun\"],\"categories\":[]}\n```";
    assert_eq!(parse_model_response(wrapped).unwrap().pos, vec!["noun"]);
    assert!(parse_model_response("<think>unfinished").is_err());
}

#[test]
fn still_rejects_incorrect_labels_or_arbitrary_prefix() {
    assert!(parse_model_response("Answer: {\"pos\":[\"noun\"],\"categories\":[]}").is_err());
    assert!(parse_model_response("{\"pos\":[\"invented\"],\"categories\":[]}").is_err());
}
