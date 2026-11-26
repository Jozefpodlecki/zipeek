use super::{
    DailyProgress, KnowledgeCategory, KnowledgeDimension, KnowledgeEntityId, KnowledgeState,
    PolicyError, RelapsePhase, ScoringPolicy,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssessmentOutcome {
    Correct,
    Incorrect,
    Uncertain,
    NotAssessed,
    Invalid,
    EvaluationFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssessmentEvent {
    pub attempt_id: String,
    pub policy_epoch: u64,
    pub entity: KnowledgeEntityId,
    pub dimension: KnowledgeDimension,
    pub policy_day: i64,
    pub context_key: Option<String>,
    pub reading_key: Option<String>,
    pub outcome: AssessmentOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnowledgeError {
    InvalidPolicy(PolicyError),
    DifferentAssessmentTarget,
    DifferentPolicyEpoch,
    EarlierPolicyDay,
    InvalidAttemptId,
    MissingOrInvalidContext,
    InvalidDimension,
    MissingRequiredReadings,
    InvalidReading,
    InconsistentState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
    DuplicateAttempt,
    IneligibleContext,
    Ignored,
    CountedSuccess,
    DayDisqualified,
    DebtAdded,
    RelapseStarted,
    Qualified,
    DebtRepaid,
    RelapseRecovered,
    Comeback,
}

pub struct KnowledgeEngine {
    policy: ScoringPolicy,
}

impl KnowledgeEngine {
    pub fn new(policy: ScoringPolicy) -> Result<Self, KnowledgeError> {
        Ok(Self {
            policy: policy.validate().map_err(KnowledgeError::InvalidPolicy)?,
        })
    }

    pub fn apply(
        &self,
        state: &mut KnowledgeState,
        event: AssessmentEvent,
    ) -> Result<Transition, KnowledgeError> {
        if state.entity != event.entity || state.dimension != event.dimension {
            return Err(KnowledgeError::DifferentAssessmentTarget);
        }
        if !state
            .entity
            .category
            .dimensions()
            .contains(&state.dimension)
        {
            return Err(KnowledgeError::InvalidDimension);
        }
        if state.policy_epoch != event.policy_epoch {
            return Err(KnowledgeError::DifferentPolicyEpoch);
        }
        if event.attempt_id.trim().is_empty() || event.attempt_id.len() > 256 {
            return Err(KnowledgeError::InvalidAttemptId);
        }
        if state.chunks > self.policy.required_days
            || state.debt > ScoringPolicy::MAXIMUM_DEBT_UNITS
            || (state.phase == RelapsePhase::Recovering
                && state.chunks != self.policy.required_days - 1)
            || (state.phase == RelapsePhase::Mastered && state.chunks != self.policy.required_days)
            || (state.phase == RelapsePhase::Acquiring && state.chunks == self.policy.required_days)
        {
            return Err(KnowledgeError::InconsistentState);
        }
        if state.seen_attempts.contains(&event.attempt_id) {
            return Ok(Transition::DuplicateAttempt);
        }
        if let Some(day) = &state.day {
            if event.policy_day < day.day {
                return Err(KnowledgeError::EarlierPolicyDay);
            }
        }
        if event.outcome == AssessmentOutcome::Correct {
            let Some(context) = event.context_key.as_ref() else {
                return Err(KnowledgeError::MissingOrInvalidContext);
            };
            if context.trim().is_empty() || context.len() > 256 {
                return Err(KnowledgeError::MissingOrInvalidContext);
            }
        }
        let polyphonic = state.entity.category == KnowledgeCategory::Hanzi
            && state.dimension == KnowledgeDimension::Pronunciation;
        if polyphonic {
            if state.required_readings.is_empty()
                || state
                    .required_readings
                    .iter()
                    .any(|s| s.trim().is_empty() || s.len() > 256)
            {
                return Err(KnowledgeError::MissingRequiredReadings);
            }
            if event.outcome == AssessmentOutcome::Correct {
                if !event
                    .reading_key
                    .as_ref()
                    .is_some_and(|r| state.required_readings.contains(r))
                {
                    return Err(KnowledgeError::InvalidReading);
                }
            }
        } else if event.reading_key.is_some() || !state.required_readings.is_empty() {
            return Err(KnowledgeError::InvalidReading);
        }
        if state
            .day
            .as_ref()
            .is_none_or(|day| day.day != event.policy_day)
        {
            let required_successes = self.policy.daily_quota(state.debt).max(if polyphonic {
                u16::try_from(state.required_readings.len()).unwrap_or(u16::MAX)
            } else {
                0
            });
            state.day = Some(DailyProgress {
                day: event.policy_day,
                required_successes,
                successes: 0,
                disqualified: false,
                qualified: false,
                debt_generated: false,
                recovery_streak: 0,
                comeback_triggered: false,
                seen_contexts: BTreeSet::new(),
                reading_successes: BTreeMap::new(),
            });
        }
        state.seen_attempts.insert(event.attempt_id.clone());
        state.observations.push(event.clone());
        match event.outcome {
            AssessmentOutcome::Correct => self.correct(
                state,
                event.context_key.expect("validated context"),
                event.reading_key,
            ),
            AssessmentOutcome::Incorrect => Ok(self.incorrect(state)),
            AssessmentOutcome::Uncertain
            | AssessmentOutcome::NotAssessed
            | AssessmentOutcome::Invalid
            | AssessmentOutcome::EvaluationFailure => Ok(Transition::Ignored),
        }
    }

    fn correct(
        &self,
        state: &mut KnowledgeState,
        context: String,
        reading: Option<String>,
    ) -> Result<Transition, KnowledgeError> {
        let day = state.day.as_mut().expect("initialized day");
        if !day.seen_contexts.insert(context) {
            return Ok(Transition::IneligibleContext);
        }
        if day.qualified {
            return Ok(Transition::Ignored);
        }
        day.successes = day.successes.saturating_add(1);
        if let Some(reading) = reading {
            let count = day.reading_successes.entry(reading).or_default();
            *count = count.saturating_add(1);
        }
        let mut comeback = false;
        if day.disqualified {
            if state.phase == RelapsePhase::Recovering {
                return Ok(Transition::Ignored);
            }
            day.recovery_streak = day.recovery_streak.saturating_add(1);
            if day.recovery_streak < 100 {
                return Ok(Transition::CountedSuccess);
            }
            day.disqualified = false;
            day.comeback_triggered = true;
            comeback = true;
        }
        if day.successes < day.required_successes
            || !Self::reading_coverage(day, &state.required_readings)
        {
            return Ok(if comeback {
                Transition::Comeback
            } else {
                Transition::CountedSuccess
            });
        }
        day.qualified = true;
        if state.phase == RelapsePhase::Recovering {
            state.debt -= 1;
            if state.debt == 0 {
                state.chunks = self.policy.required_days;
                state.phase = RelapsePhase::Mastered;
                return Ok(Transition::RelapseRecovered);
            }
            return Ok(Transition::DebtRepaid);
        }
        if state.chunks < self.policy.required_days {
            state.chunks += 1;
            if state.chunks == self.policy.required_days {
                state.phase = RelapsePhase::Mastered;
            }
        }
        if state.debt != 0 {
            state.debt -= 1;
            return Ok(if comeback {
                Transition::Comeback
            } else {
                Transition::DebtRepaid
            });
        }
        Ok(if comeback {
            Transition::Comeback
        } else {
            Transition::Qualified
        })
    }

    fn reading_coverage(day: &DailyProgress, readings: &BTreeSet<String>) -> bool {
        if readings.is_empty() {
            return true;
        }
        let total = u32::from(day.required_successes);
        let n = readings.len() as u32;
        let minimum = total / n;
        let remainder = total % n;
        if readings
            .iter()
            .any(|r| u32::from(*day.reading_successes.get(r).unwrap_or(&0)) < minimum)
        {
            return false;
        }
        let extras = readings
            .iter()
            .filter(|r| u32::from(*day.reading_successes.get(*r).unwrap_or(&0)) > minimum)
            .count() as u32;
        extras >= remainder
    }

    fn incorrect(&self, state: &mut KnowledgeState) -> Transition {
        let day = state.day.as_mut().expect("initialized day");
        if !day.qualified {
            day.disqualified = true;
            day.recovery_streak = 0;
        }
        if day.debt_generated {
            return Transition::DayDisqualified;
        }
        if state.phase == RelapsePhase::Mastered {
            state.chunks -= 1;
            state.phase = RelapsePhase::Recovering;
            state.debt = ScoringPolicy::MAXIMUM_DEBT_UNITS;
            day.debt_generated = true;
            return Transition::RelapseStarted;
        }
        if state.chunks >= self.policy.debt_eligibility_days {
            state.debt = state
                .debt
                .saturating_add(1)
                .min(ScoringPolicy::MAXIMUM_DEBT_UNITS);
            day.debt_generated = true;
            return Transition::DebtAdded;
        }
        Transition::DayDisqualified
    }
}
