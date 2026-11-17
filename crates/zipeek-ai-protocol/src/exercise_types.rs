pub use zipeek_exercises::definition::{
    DEFINITIONS, ExerciseDefinition, VerdictAuthority, definition,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn scoring_authority_is_owned_by_domain_definition() {
        assert_eq!(
            definition("matching").unwrap().verdict_authority,
            VerdictAuthority::Deterministic
        );
        assert_eq!(
            definition("sentence-translation")
                .unwrap()
                .verdict_authority,
            VerdictAuthority::TranslationEvaluator
        );
        let definition_ids: HashSet<_> = DEFINITIONS.iter().map(|item| item.id).collect();
        let playlist_ids: HashSet<_> = zipeek_exercises::definition::PRACTICE_PLAYLIST_ORDER
            .into_iter()
            .collect();
        assert_eq!(
            definition_ids.len(),
            DEFINITIONS.len(),
            "duplicate exercise definition"
        );
        assert_eq!(
            playlist_ids.len(),
            zipeek_exercises::definition::PRACTICE_PLAYLIST_ORDER.len(),
            "duplicate playlist entry"
        );
        assert_eq!(
            definition_ids, playlist_ids,
            "exercise catalog and playlist differ"
        );
        assert_eq!(
            definition("write-hanzi").unwrap().verdict_authority,
            VerdictAuthority::PracticeOnly
        );
        assert_eq!(
            definition("spot-mistake").unwrap().verdict_authority,
            VerdictAuthority::PracticeOnly
        );
        assert_eq!(
            definition("sentence-builder").unwrap().verdict_authority,
            VerdictAuthority::Deterministic
        );
        assert_eq!(
            definition("speaking").unwrap().verdict_authority,
            VerdictAuthority::PracticeOnly
        );
    }
}
