pub use zipeek_exercises::definition::{
    ExerciseDefinition, VerdictAuthority, DEFINITIONS, definition,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoring_authority_is_owned_by_domain_definition() {
        assert_eq!(definition("matching").unwrap().verdict_authority, VerdictAuthority::Deterministic);
        assert_eq!(definition("sentence-translation").unwrap().verdict_authority, VerdictAuthority::TranslationEvaluator);
        assert_eq!(DEFINITIONS.len(), 5);
    }
}
