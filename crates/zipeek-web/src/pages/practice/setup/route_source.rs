use std::collections::HashMap;

pub(super) fn preselected_source(query: Option<&HashMap<String, String>>) -> String {
    query
        .and_then(|query| query.get("source"))
        .filter(|source| {
            source
                .strip_prefix("saved:")
                .and_then(|id| id.parse::<crate::services::word_sets::WordSetId>().ok())
                .is_some()
        })
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_source_requires_a_valid_saved_set_id() {
        for valid in [
            "saved:00112233445566778899aabbccddeeff",
            "saved:ffeeddccbbaa99887766554433221100",
        ] {
            let query = HashMap::from([("source".to_owned(), valid.to_owned())]);
            assert_eq!(preselected_source(Some(&query)), valid);
        }
        for invalid in ["saved:", "saved:nope", "saved:-1", "saved:0", "other:0"] {
            let query = HashMap::from([("source".to_owned(), invalid.to_owned())]);
            assert!(preselected_source(Some(&query)).is_empty());
        }
    }
}
