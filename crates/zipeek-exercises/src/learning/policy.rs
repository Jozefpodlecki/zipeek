#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MasteryPolicy {
    pub minimum_distinct_success_days: usize,
    pub minimum_delayed_success_gap_days: i64,
    pub extended_success_days: usize,
}

impl Default for MasteryPolicy {
    fn default() -> Self {
        Self {
            minimum_distinct_success_days: 7,
            minimum_delayed_success_gap_days: 7,
            extended_success_days: 14,
        }
    }
}

impl MasteryPolicy {
    pub const VERSION: &'static str = "mastery-v1";

    pub fn validate(self) -> bool {
        self.minimum_distinct_success_days > 0
            && self.minimum_delayed_success_gap_days > 0
            && self.extended_success_days >= self.minimum_distinct_success_days
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillState {
    Unassessed,
    InsufficientEvidence,
    Learning,
    Learned,
    NeedsReview,
    Inapplicable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_mastery_thresholds_are_rejected() {
        let valid = MasteryPolicy::default();
        assert!(valid.validate());
        assert!(
            !MasteryPolicy {
                minimum_distinct_success_days: 0,
                ..valid
            }
            .validate()
        );
        assert!(
            !MasteryPolicy {
                minimum_delayed_success_gap_days: 0,
                ..valid
            }
            .validate()
        );
        assert!(
            !MasteryPolicy {
                extended_success_days: 6,
                ..valid
            }
            .validate()
        );
        assert!(
            MasteryPolicy {
                extended_success_days: 7,
                ..valid
            }
            .validate()
        );
    }

    #[test]
    fn custom_policy_has_consistent_success_thresholds() {
        let baseline = MasteryPolicy::default();
        assert!(baseline.validate());
        let larger = MasteryPolicy {
            minimum_distinct_success_days: 15,
            extended_success_days: 14,
            ..baseline
        };
        assert!(!larger.validate());
        let same = MasteryPolicy {
            minimum_distinct_success_days: 14,
            extended_success_days: 14,
            ..baseline
        };
        assert!(same.validate());
    }

    #[test]
    fn mastery_policy_rejects_thresholds_without_a_positive_delayed_interval() {
        let baseline = MasteryPolicy::default();
        let invalid = MasteryPolicy {
            minimum_distinct_success_days: 1,
            minimum_delayed_success_gap_days: -1,
            extended_success_days: 1,
        };
        assert!(!invalid.validate());
        assert!(
            MasteryPolicy {
                minimum_distinct_success_days: 1,
                minimum_delayed_success_gap_days: 1,
                extended_success_days: 1,
            }
            .validate()
        );
        assert!(baseline.validate());
    }
}
