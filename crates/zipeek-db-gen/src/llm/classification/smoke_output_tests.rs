use super::{batch_token_budget, json_with_trailing_periods};

#[test]
fn recovers_only_trailing_periods_after_complete_json() {
    assert_eq!(
        json_with_trailing_periods(" {\"results\":[]}.. ").as_deref(),
        Some("{\"results\":[]}")
    );
    assert!(json_with_trailing_periods("system\n{\"results\":[]}").is_none());
    assert!(json_with_trailing_periods("{\"results\":[]} extra").is_none());
    assert!(json_with_trailing_periods("{\"results\":[]}...").is_none());
    assert!(json_with_trailing_periods("{\"results\":[]}").is_none());
    assert_eq!(batch_token_budget(100), 16384);
    assert!(batch_token_budget(2) >= 2048);
}
