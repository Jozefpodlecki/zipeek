use super::policy::SkillState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VocabularySkills {
    pub recognition: SkillState,
    pub listening: SkillState,
    pub pronunciation: SkillState,
    pub typing: SkillState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverallState {
    Unassessed,
    Learning,
    ReceptivelyLearned,
    FullyLearned,
    NeedsReview,
    InsufficientEvidence,
}

pub fn overall_state(skills: VocabularySkills) -> OverallState {
    let all = [
        skills.recognition,
        skills.listening,
        skills.pronunciation,
        skills.typing,
    ];
    if all.iter().any(|state| *state == SkillState::NeedsReview) {
        return OverallState::NeedsReview;
    }
    if all.iter().all(|state| *state == SkillState::Unassessed) {
        return OverallState::Unassessed;
    }
    let effective =
        |state: SkillState| matches!(state, SkillState::Learned | SkillState::Inapplicable);
    if all.iter().all(|state| effective(*state)) && all.contains(&SkillState::Learned) {
        return OverallState::FullyLearned;
    }
    if effective(skills.recognition)
        && effective(skills.listening)
        && (skills.recognition == SkillState::Learned || skills.listening == SkillState::Learned)
    {
        return OverallState::ReceptivelyLearned;
    }
    if all
        .iter()
        .any(|state| *state == SkillState::InsufficientEvidence)
    {
        return OverallState::InsufficientEvidence;
    }
    OverallState::Learning
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_lapsed_skill_overrides_otherwise_complete_mastery() {
        let all_learned = VocabularySkills {
            recognition: SkillState::Learned,
            listening: SkillState::Learned,
            pronunciation: SkillState::Learned,
            typing: SkillState::Learned,
        };
        assert_eq!(overall_state(all_learned), OverallState::FullyLearned);
        assert_eq!(
            overall_state(VocabularySkills {
                pronunciation: SkillState::NeedsReview,
                ..all_learned
            }),
            OverallState::NeedsReview
        );
        assert_eq!(
            overall_state(VocabularySkills {
                pronunciation: SkillState::InsufficientEvidence,
                typing: SkillState::Unassessed,
                ..all_learned
            }),
            OverallState::ReceptivelyLearned
        );
    }

    #[test]
    fn receptive_success_does_not_claim_full_mastery() {
        let skills = VocabularySkills {
            recognition: SkillState::Learned,
            listening: SkillState::Learned,
            pronunciation: SkillState::Unassessed,
            typing: SkillState::Unassessed,
        };
        assert_eq!(overall_state(skills), OverallState::ReceptivelyLearned);
        assert_eq!(
            overall_state(VocabularySkills {
                typing: SkillState::Learned,
                ..skills
            }),
            OverallState::ReceptivelyLearned
        );
        assert_eq!(
            overall_state(VocabularySkills {
                typing: SkillState::Learned,
                pronunciation: SkillState::Learned,
                ..skills
            }),
            OverallState::FullyLearned
        );
    }
    #[test]
    fn needs_review_is_not_masked_by_inapplicable_competencies() {
        let skills = VocabularySkills {
            recognition: SkillState::Inapplicable,
            listening: SkillState::NeedsReview,
            pronunciation: SkillState::Inapplicable,
            typing: SkillState::Inapplicable,
        };
        assert_eq!(overall_state(skills), OverallState::NeedsReview);
    }
}
