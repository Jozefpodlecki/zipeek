use std::collections::BTreeSet;

#[derive(Clone, Debug, Default)]
pub enum RevealState {
    #[default]
    Unassessed,
    GrammarCompleted,
    Vocabulary {
        member_ids: BTreeSet<String>,
        known_ids: BTreeSet<String>,
    },
}
impl RevealState {
    pub fn grammar_cover(&self) -> f32 {
        if matches!(self, Self::GrammarCompleted) {
            0.0
        } else {
            1.0
        }
    }
    pub fn vocabulary_cover(&self) -> f32 {
        match self {
            Self::Vocabulary {
                member_ids,
                known_ids,
            } if !member_ids.is_empty() => {
                let known = member_ids.intersection(known_ids).count();
                1.0 - known as f32 / member_ids.len() as f32
            }
            _ => 1.0,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unassessed_and_guided_lessons_remain_opaque() {
        let unassessed = RevealState::Unassessed;
        assert_eq!(unassessed.grammar_cover(), 1.0);
        assert_eq!(unassessed.vocabulary_cover(), 1.0);
    }
    #[test]
    fn grammar_only_reveals_with_verified_completion() {
        assert_eq!(RevealState::GrammarCompleted.grammar_cover(), 0.0);
        assert_eq!(RevealState::GrammarCompleted.vocabulary_cover(), 1.0);
    }
    #[test]
    fn word_coverage_counts_unique_members_and_ignores_unrelated_words() {
        let ids = ["a", "b", "c", "d"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let known = ["b", "d", "elsewhere"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let state = RevealState::Vocabulary {
            member_ids: ids,
            known_ids: known,
        };
        assert_eq!(state.vocabulary_cover(), 0.5);
        assert_eq!(state.grammar_cover(), 1.0);
    }
    #[test]
    fn empty_group_remains_hidden() {
        let state = RevealState::Vocabulary {
            member_ids: BTreeSet::new(),
            known_ids: BTreeSet::new(),
        };
        assert_eq!(state.vocabulary_cover(), 1.0);
    }
}
