mod detail;
mod items;
mod page;
mod view;
pub(super) use page::{GrammarExplorer, matching_patterns};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grammar_fixture_has_no_fake_authoritative_identifiers_or_assessment() {
        let rows = super::super::data::demo_grammar_for_tests();
        assert_eq!(rows.len(), 550);
        for row in rows.iter() {
            assert!(row.canonical_id.is_none());
            assert!(row.course_id.is_none());
            assert!(row.assessed_attempts.is_none());
            assert!(!row.example.is_empty());
            assert!(!row.teaching_review_status.is_empty());
        }
    }

    #[test]
    fn grammar_fixture_ids_are_unique() {
        let rows = super::super::data::demo_grammar_for_tests();
        let ids = rows
            .iter()
            .map(|item| item.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), 550);
        assert!(
            matching_patterns(&rows, "subject")
                .iter()
                .any(|item| item.id == "g001-p01")
        );
    }
}
