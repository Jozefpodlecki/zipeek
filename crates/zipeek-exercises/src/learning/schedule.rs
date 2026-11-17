use super::mastery::SkillProjection;
use super::policy::SkillState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewSuggestion {
    pub due_at_ms: Option<i64>,
    pub interval_days: Option<u32>,
    pub scheduler_version: &'static str,
}

pub fn suggest_review(projection: &SkillProjection, latest_review_ms: i64) -> ReviewSuggestion {
    const DAY_MS: i64 = 86_400_000;
    let interval = match projection.state {
        SkillState::Unassessed | SkillState::Inapplicable => None,
        SkillState::NeedsReview => Some(1_u32),
        SkillState::InsufficientEvidence => None,
        SkillState::Learning | SkillState::Learned => Some(match projection.successful_days {
            0..=1 => 1,
            2..=3 => 3,
            4..=6 => 7,
            _ => 14,
        }),
    };
    ReviewSuggestion {
        due_at_ms: interval.and_then(|days| latest_review_ms.checked_add(i64::from(days) * DAY_MS)),
        interval_days: interval,
        scheduler_version: "scheduler-v1",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn projection(state: SkillState, successful_days: usize) -> SkillProjection {
        SkillProjection {
            state,
            successful_days,
            qualifying_successes: 0,
            qualifying_failures: 0,
            delayed_successes: 0,
            longest_success_gap_days: 0,
            best_calendar_streak_days: 0,
            latest_failure_ms: None,
            latest_success_ms: None,
            policy_version: "mastery-v1",
        }
    }

    #[test]
    fn unassessed_and_unqualified_skills_are_not_scheduled() {
        for state in [
            SkillState::Unassessed,
            SkillState::Inapplicable,
            SkillState::InsufficientEvidence,
        ] {
            let suggestion = suggest_review(&projection(state, 0), 1_000);
            assert_eq!(suggestion.interval_days, None);
            assert_eq!(suggestion.due_at_ms, None);
        }
    }

    #[test]
    fn due_dates_are_checked_for_overflow() {
        let review = suggest_review(&projection(SkillState::NeedsReview, 0), i64::MAX);
        assert_eq!(review.interval_days, Some(1));
        assert_eq!(review.due_at_ms, None);
        let review = suggest_review(&projection(SkillState::Learning, 4), 0);
        assert_eq!(review.interval_days, Some(7));
        assert_eq!(review.due_at_ms, Some(7 * 86_400_000));
    }
    #[test]
    fn increasing_assessment_days_produces_a_non_decreasing_review_interval() {
        let mut previous = 0;
        for days in 0..=15 {
            let suggestion = suggest_review(&projection(SkillState::Learning, days), 0);
            let interval = suggestion.interval_days.unwrap();
            assert!(interval >= previous);
            previous = interval;
        }
    }
}
