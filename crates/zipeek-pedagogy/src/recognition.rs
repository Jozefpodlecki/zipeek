use std::collections::{BTreeMap, HashMap};

use crate::local_day_number;

const DAY_MS: i64 = 86_400_000;
const SESSION_LIMIT: usize = 4;
const DAILY_SUCCESSES: usize = 10;
const ACQUISITION_DAYS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MistakeAllowance {
    Strict,
    One,
    Two,
}

impl MistakeAllowance {
    pub const fn maximum(self) -> usize {
        match self {
            Self::Strict => 0,
            Self::One => 1,
            Self::Two => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssessmentFormat {
    IndependentRecall,
    ValidatedChoice { validation_id: String },
    SupportingOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEvidence {
    pub session_id: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognitionEvent {
    pub attempt_id: String,
    pub character: char,
    pub session: SessionEvidence,
    pub occurred_at_utc_ms: i64,
    pub timezone_iana: String,
    pub timezone_offset_minutes: i32,
    pub local_date: String,
    pub mistake_allowance: MistakeAllowance,
    pub evaluator_id: String,
    pub assessment_version: String,
    pub correct: bool,
    pub assisted: bool,
    pub format: AssessmentFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DailyStatus {
    InProgress,
    Qualified,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognitionState {
    pub character: char,
    pub streak_days: usize,
    pub acquired_at_utc_ms: Option<i64>,
    pub needs_review: bool,
    pub current_day: Option<String>,
    pub current_day_status: Option<DailyStatus>,
    pub current_day_correct: usize,
    pub current_day_mistakes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaySummary {
    pub character: char,
    pub local_date: String,
    pub valid_correct: usize,
    pub mistakes: usize,
    pub credited_sessions: usize,
    pub status: DailyStatus,
    pub qualified_at_utc_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognitionProjection {
    pub days: Vec<DaySummary>,
    pub states: BTreeMap<char, RecognitionState>,
    pub accepted_attempts: usize,
    pub supporting_attempts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    InvalidAttempt,
    InvalidCalendar,
    InvalidTime,
    ConflictingDuplicate(String),
    FutureEvidence,
    InconsistentDayPolicy,
}

#[derive(Default)]
struct DayRecord {
    correct: usize,
    mistakes: usize,
    per_session: HashMap<String, usize>,
    status: Option<DailyStatus>,
    qualified_at: Option<i64>,
}

impl DayRecord {
    fn record(
        &mut self,
        event: &RecognitionEvent,
        allowance: MistakeAllowance,
        earliest_qualification: Option<i64>,
    ) {
        if self.status.is_some() {
            return;
        }
        if !event.correct {
            self.mistakes += 1;
            if self.mistakes > allowance.maximum() {
                self.status = Some(DailyStatus::Failed);
            }
        } else {
            let from_session = self
                .per_session
                .entry(event.session.session_id.clone())
                .or_default();
            if *from_session < SESSION_LIMIT {
                *from_session += 1;
                self.correct += 1;
            }
        }
        if event.correct
            && self.correct >= DAILY_SUCCESSES
            && self.per_session.len() >= 3
            && earliest_qualification.is_none_or(|at| event.occurred_at_utc_ms >= at)
        {
            self.status = Some(DailyStatus::Qualified);
            self.qualified_at = Some(event.occurred_at_utc_ms);
        }
    }
}

fn eligible(event: &RecognitionEvent) -> bool {
    !event.assisted
        && event.session.completed
        && !event.evaluator_id.trim().is_empty()
        && !event.assessment_version.trim().is_empty()
        && match &event.format {
            AssessmentFormat::IndependentRecall => true,
            AssessmentFormat::ValidatedChoice { validation_id } => !validation_id.trim().is_empty(),
            AssessmentFormat::SupportingOnly => false,
        }
}

fn validate(event: &RecognitionEvent) -> Result<i64, EvidenceError> {
    if event.attempt_id.trim().is_empty()
        || event.session.session_id.trim().is_empty()
        || event.timezone_iana.trim().is_empty()
        || event.character.is_whitespace()
        || !(0x3400..=0x9fff).contains(&(event.character as u32))
    {
        return Err(EvidenceError::InvalidAttempt);
    }
    let day = local_day_number(&event.local_date).map_err(|_| EvidenceError::InvalidCalendar)?;
    if !(-840..=840).contains(&event.timezone_offset_minutes) {
        return Err(EvidenceError::InvalidTime);
    }
    let offset = i64::from(event.timezone_offset_minutes) * 60_000;
    let local_ms = event
        .occurred_at_utc_ms
        .checked_add(offset)
        .ok_or(EvidenceError::InvalidTime)?;
    if local_ms.div_euclid(DAY_MS) != day {
        return Err(EvidenceError::InvalidTime);
    }
    Ok(day)
}

pub fn project_recognition(
    events: &[RecognitionEvent],
    as_of_utc_ms: i64,
) -> Result<RecognitionProjection, EvidenceError> {
    let mut distinct = HashMap::<&str, &RecognitionEvent>::new();
    let mut day_policies = BTreeMap::<(char, i64), MistakeAllowance>::new();
    for event in events {
        let day = validate(event)?;
        match day_policies.insert((event.character, day), event.mistake_allowance) {
            Some(previous) if previous != event.mistake_allowance => {
                return Err(EvidenceError::InconsistentDayPolicy);
            }
            _ => {}
        }
        if event.occurred_at_utc_ms > as_of_utc_ms {
            return Err(EvidenceError::FutureEvidence);
        }
        match distinct.get(event.attempt_id.as_str()) {
            Some(previous) if *previous != event => {
                return Err(EvidenceError::ConflictingDuplicate(
                    event.attempt_id.clone(),
                ));
            }
            Some(_) => {}
            None => {
                distinct.insert(&event.attempt_id, event);
            }
        }
    }
    let mut ordered: Vec<_> = distinct.into_values().collect();
    ordered.sort_by(|a, b| {
        (a.occurred_at_utc_ms, &a.attempt_id).cmp(&(b.occurred_at_utc_ms, &b.attempt_id))
    });

    let mut states = BTreeMap::<char, RecognitionState>::new();
    let mut grouped = BTreeMap::<char, BTreeMap<i64, Vec<&RecognitionEvent>>>::new();
    let mut supporting_attempts = 0;
    let mut day_summaries = Vec::new();
    for event in ordered.iter().copied() {
        states
            .entry(event.character)
            .or_insert_with(|| RecognitionState {
                character: event.character,
                streak_days: 0,
                acquired_at_utc_ms: None,
                needs_review: false,
                current_day: None,
                current_day_status: None,
                current_day_correct: 0,
                current_day_mistakes: 0,
            });
        if !eligible(event) {
            supporting_attempts += 1;
            continue;
        }
        let day = validate(event)?;
        grouped
            .entry(event.character)
            .or_default()
            .entry(day)
            .or_default()
            .push(event);
    }

    for (character, days) in grouped {
        let state = states
            .get_mut(&character)
            .expect("recorded character has a state");
        let mut last_qualified_day: Option<i64> = None;
        let mut last_qualified_ms: Option<i64> = None;
        for (day, day_events) in days {
            if state.acquired_at_utc_ms.is_none()
                && last_qualified_day.is_some_and(|previous| day != previous + 1)
            {
                state.streak_days = 0;
                last_qualified_day = None;
                last_qualified_ms = None;
            }
            let mut record = DayRecord::default();
            for event in &day_events {
                record.record(
                    event,
                    event.mistake_allowance,
                    last_qualified_ms.and_then(|at| at.checked_add(DAY_MS)),
                );
            }
            if state.acquired_at_utc_ms.is_none() {
                match record.status {
                    Some(DailyStatus::Failed) => {
                        state.streak_days = 0;
                        last_qualified_day = None;
                        last_qualified_ms = None;
                    }
                    Some(DailyStatus::Qualified) => {
                        state.streak_days += 1;
                        last_qualified_day = Some(day);
                        last_qualified_ms = record.qualified_at;
                        if state.streak_days == ACQUISITION_DAYS {
                            state.acquired_at_utc_ms = record.qualified_at;
                        }
                    }
                    _ => {}
                }
            }
            let local_date = day_events
                .first()
                .expect("nonempty day events")
                .local_date
                .clone();
            day_summaries.push(DaySummary {
                character,
                local_date: local_date.clone(),
                valid_correct: record.correct,
                mistakes: record.mistakes,
                credited_sessions: record.per_session.len(),
                status: record.status.unwrap_or(DailyStatus::InProgress),
                qualified_at_utc_ms: record.qualified_at,
            });
            state.current_day = Some(local_date);
            state.current_day_status = Some(record.status.unwrap_or(DailyStatus::InProgress));
            state.current_day_correct = record.correct;
            state.current_day_mistakes = record.mistakes;
        }
        if state.acquired_at_utc_ms.is_none() {
            let latest_offset = ordered
                .iter()
                .rev()
                .find(|event| event.character == character)
                .map_or(0, |event| i64::from(event.timezone_offset_minutes) * 60_000);
            let as_of_day = as_of_utc_ms
                .saturating_add(latest_offset)
                .div_euclid(DAY_MS);
            if last_qualified_day.is_some_and(|day| as_of_day > day + 1) {
                state.streak_days = 0;
            }
        }
    }
    for event in &ordered {
        if eligible(event) {
            if let Some(state) = states.get_mut(&event.character) {
                if state
                    .acquired_at_utc_ms
                    .is_some_and(|at| event.occurred_at_utc_ms > at)
                {
                    state.needs_review = !event.correct;
                }
            }
        }
    }
    Ok(RecognitionProjection {
        days: day_summaries,
        states,
        accepted_attempts: ordered.len() - supporting_attempts,
        supporting_attempts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_DAY: i64 = 20_000;

    fn observation(day: i64, session: usize, ordinal: usize, correct: bool) -> RecognitionEvent {
        let date = {
            let mut year = 2024;
            let mut month = 10;
            let mut day_of_month = 4 + (day - BASE_DAY) as u32;
            while day_of_month > 31 {
                day_of_month -= 31;
                month += 1;
            }
            if month > 12 {
                year += 1;
                month -= 12;
            }
            format!("{year:04}-{month:02}-{day_of_month:02}")
        };
        assert_eq!(local_day_number(&date), Ok(day));
        RecognitionEvent {
            attempt_id: format!("{day}-{session}-{ordinal}"),
            character: '学',
            session: SessionEvidence {
                session_id: format!("{day}-session-{session}"),
                completed: true,
            },
            occurred_at_utc_ms: day * DAY_MS + 12 * 3_600_000 + ordinal as i64 * 1_000,
            timezone_iana: "Etc/UTC".into(),
            timezone_offset_minutes: 0,
            local_date: date,
            mistake_allowance: MistakeAllowance::Strict,
            evaluator_id: "test-assessor".into(),
            assessment_version: "recognition-v1".into(),
            correct,
            assisted: false,
            format: AssessmentFormat::IndependentRecall,
        }
    }

    fn qualifying_day(day: i64) -> Vec<RecognitionEvent> {
        (0..10).map(|i| observation(day, i / 4, i, true)).collect()
    }

    fn state(events: &[RecognitionEvent]) -> RecognitionState {
        project_recognition(
            events,
            events
                .iter()
                .map(|event| event.occurred_at_utc_ms)
                .max()
                .unwrap()
                + 60_000,
        )
        .unwrap()
        .states
        .remove(&'学')
        .unwrap()
    }

    #[test]
    fn ten_across_three_sessions_qualify_once() {
        let result = state(&qualifying_day(BASE_DAY));
        assert_eq!(result.streak_days, 1);
        assert_eq!(result.current_day_status, Some(DailyStatus::Qualified));
        assert_eq!(result.current_day_correct, 10);
    }

    #[test]
    fn two_sessions_and_cap_do_not_qualify() {
        let events: Vec<_> = (0..10)
            .map(|i| observation(BASE_DAY, i / 5, i, true))
            .collect();
        let result = state(&events);
        assert_eq!(result.current_day_correct, 8);
        assert_eq!(result.current_day_status, Some(DailyStatus::InProgress));
    }

    #[test]
    fn seven_successful_consecutive_days_acquire() {
        let events: Vec<_> = (0..7).flat_map(|d| qualifying_day(BASE_DAY + d)).collect();
        let result = state(&events);
        assert_eq!(result.streak_days, 7);
        assert!(result.acquired_at_utc_ms.is_some());
    }

    #[test]
    fn missed_day_resets_streak() {
        let mut events = qualifying_day(BASE_DAY);
        events.extend(qualifying_day(BASE_DAY + 2));
        let result = state(&events);
        assert_eq!(result.streak_days, 1);
    }

    #[test]
    fn exceeding_allowance_fails_and_resets_prior_streak() {
        for (allowance, allowed) in [
            (MistakeAllowance::Strict, 0),
            (MistakeAllowance::One, 1),
            (MistakeAllowance::Two, 2),
        ] {
            let mut events = qualifying_day(BASE_DAY);
            for i in 0..=allowed {
                let mut failure = observation(BASE_DAY + 1, 0, i, false);
                failure.mistake_allowance = allowance;
                events.push(failure);
            }
            events.extend(qualifying_day(BASE_DAY + 1).into_iter().map(|mut e| {
                e.attempt_id = format!("success-{}", e.attempt_id);
                e.mistake_allowance = allowance;
                e
            }));
            let result = state(&events);
            assert_eq!(result.streak_days, 0);
            assert_eq!(result.current_day_status, Some(DailyStatus::Failed));
        }
    }

    #[test]
    fn lock_prevents_later_mistakes_from_invalidating_day() {
        let mut events = qualifying_day(BASE_DAY);
        events.push(observation(BASE_DAY, 3, 11, false));
        let result = state(&events);
        assert_eq!(result.current_day_status, Some(DailyStatus::Qualified));
        assert_eq!(result.current_day_mistakes, 0);
    }

    #[test]
    fn same_attempt_retry_is_idempotent_but_conflict_is_rejected() {
        let mut events = qualifying_day(BASE_DAY);
        events.push(events[0].clone());
        assert_eq!(state(&events).current_day_correct, 10);
        let mut conflicting = events[0].clone();
        conflicting.correct = false;
        events.push(conflicting);
        assert!(matches!(
            project_recognition(&events, (BASE_DAY + 20) * DAY_MS),
            Err(EvidenceError::ConflictingDuplicate(_))
        ));
    }

    #[test]
    fn supporting_and_assisted_attempts_do_not_qualify() {
        let mut events = qualifying_day(BASE_DAY);
        events[0].assisted = true;
        events[1].format = AssessmentFormat::SupportingOnly;
        let result = state(&events);
        assert_eq!(result.current_day_correct, 8);
        assert_eq!(result.current_day_status, Some(DailyStatus::InProgress));
    }

    #[test]
    fn retention_can_fall_without_revoking_acquisition() {
        let mut events: Vec<_> = (0..7).flat_map(|d| qualifying_day(BASE_DAY + d)).collect();
        events.push(observation(BASE_DAY + 8, 0, 0, false));
        let result = state(&events);
        assert!(result.acquired_at_utc_ms.is_some());
        assert!(result.needs_review);
    }

    #[test]
    fn qualification_waits_for_full_twenty_four_hours() {
        let mut events = qualifying_day(BASE_DAY);
        let previous_end = events.last().unwrap().occurred_at_utc_ms;
        let mut early = qualifying_day(BASE_DAY + 1);
        for event in &mut early {
            event.occurred_at_utc_ms -= 60_000;
        }
        events.extend(early);
        let result = state(&events);
        assert_eq!(result.streak_days, 1);
        assert_eq!(result.current_day_status, Some(DailyStatus::InProgress));
        assert_eq!(result.current_day_correct, 10);
        let mut eligible = observation(BASE_DAY + 1, 3, 11, true);
        eligible.occurred_at_utc_ms = previous_end + DAY_MS;
        events.push(eligible);
        assert_eq!(state(&events).streak_days, 2);
    }

    #[test]
    fn daily_policy_is_frozen_and_can_change_on_following_day() {
        let mut events = qualifying_day(BASE_DAY);
        let mut following = qualifying_day(BASE_DAY + 1);
        for event in &mut following {
            event.mistake_allowance = MistakeAllowance::One;
        }
        events.extend(following);
        assert_eq!(state(&events).streak_days, 2);
        events[10].mistake_allowance = MistakeAllowance::Two;
        assert_eq!(
            project_recognition(&events, (BASE_DAY + 2) * DAY_MS),
            Err(EvidenceError::InconsistentDayPolicy)
        );
    }

    #[test]
    fn one_allowed_mistake_still_needs_ten_successes() {
        let mut events = qualifying_day(BASE_DAY);
        for event in &mut events {
            event.mistake_allowance = MistakeAllowance::One;
        }
        let mut wrong = observation(BASE_DAY, 0, 10, false);
        wrong.mistake_allowance = MistakeAllowance::One;
        wrong.occurred_at_utc_ms -= 90_000;
        events.push(wrong);
        let result = state(&events);
        assert_eq!(result.current_day_status, Some(DailyStatus::Qualified));
        assert_eq!(result.current_day_mistakes, 1);
    }

    #[test]
    fn replay_order_does_not_change_projection() {
        let mut events: Vec<_> = (0..7).flat_map(|d| qualifying_day(BASE_DAY + d)).collect();
        let reference = state(&events);
        events.reverse();
        assert_eq!(state(&events), reference);
    }

    #[test]
    fn independent_characters_have_independent_streaks() {
        let mut events = qualifying_day(BASE_DAY);
        let mut other = qualifying_day(BASE_DAY);
        for event in &mut other {
            event.character = '水';
            event.attempt_id = format!("water-{}", event.attempt_id);
        }
        other.push({
            let mut failure = observation(BASE_DAY, 3, 11, false);
            failure.character = '水';
            failure.attempt_id = "water-failure".into();
            failure.occurred_at_utc_ms -= 60_000;
            failure
        });
        events.extend(other);
        let projection = project_recognition(&events, (BASE_DAY + 1) * DAY_MS).unwrap();
        assert_eq!(projection.states[&'学'].streak_days, 1);
        assert_eq!(projection.states[&'水'].streak_days, 0);
    }

    #[test]
    fn validated_choice_counts_but_unvalidated_choice_does_not() {
        let mut events = qualifying_day(BASE_DAY);
        events[0].format = AssessmentFormat::ValidatedChoice {
            validation_id: "v1-proof".into(),
        };
        events[1].format = AssessmentFormat::ValidatedChoice {
            validation_id: String::new(),
        };
        let result = state(&events);
        assert_eq!(result.current_day_correct, 9);
        assert_eq!(result.current_day_status, Some(DailyStatus::InProgress));
    }

    #[test]
    fn day_summaries_explain_why_qualification_is_pending() {
        let events: Vec<_> = (0..8)
            .map(|i| observation(BASE_DAY, i / 4, i, true))
            .collect();
        let projection = project_recognition(&events, (BASE_DAY + 1) * DAY_MS).unwrap();
        assert_eq!(projection.days.len(), 1);
        let summary = &projection.days[0];
        assert_eq!(summary.valid_correct, 8);
        assert_eq!(summary.credited_sessions, 2);
        assert_eq!(summary.status, DailyStatus::InProgress);
        assert!(summary.qualified_at_utc_ms.is_none());
    }

    #[test]
    fn subsequent_success_restores_retention_without_losing_award() {
        let mut events: Vec<_> = (0..7).flat_map(|d| qualifying_day(BASE_DAY + d)).collect();
        events.push(observation(BASE_DAY + 8, 0, 0, false));
        events.push(observation(BASE_DAY + 8, 1, 1, true));
        let state = state(&events);
        assert!(state.acquired_at_utc_ms.is_some());
        assert!(!state.needs_review);
    }

    #[test]
    fn malformed_dates_are_rejected() {
        let mut event = observation(BASE_DAY, 0, 0, true);
        event.local_date = "2024-02-30".into();
        assert_eq!(
            project_recognition(&[event], (BASE_DAY + 20) * DAY_MS),
            Err(EvidenceError::InvalidCalendar)
        );
    }
}
