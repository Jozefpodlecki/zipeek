use crate::planned_runtime::PlannedRuntime;
use crate::planner::PlanStatus;
use crate::session::ExerciseResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRoundTicket {
    pub generation: u64,
    pub instance_id: String,
    pub definition_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffError {
    NoActiveSession,
    StaleGeneration,
    WrongInstance,
    WrongDefinition,
    ResultPending,
}

impl PlannedRoundTicket {
    pub fn from_runtime(runtime: &PlannedRuntime) -> Result<Self, HandoffError> {
        let current = runtime.current().ok_or(HandoffError::NoActiveSession)?;
        Ok(Self {
            generation: runtime.generation(),
            instance_id: current.instance_id.clone(),
            definition_id: current.definition_id,
        })
    }

    pub fn validate(&self, runtime: &PlannedRuntime) -> Result<(), HandoffError> {
        if self.generation != runtime.generation() {
            return Err(HandoffError::StaleGeneration);
        }
        let session = runtime.active().ok_or(HandoffError::NoActiveSession)?;
        let current = session.current().ok_or(HandoffError::NoActiveSession)?;
        if self.instance_id != current.instance_id {
            return Err(HandoffError::WrongInstance);
        }
        if self.definition_id != current.definition_id {
            return Err(HandoffError::WrongDefinition);
        }
        if session.plan().status != PlanStatus::Ready {
            return Err(HandoffError::ResultPending);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundScoreSnapshot {
    pub correct: u64,
    pub mistakes: u64,
}

pub fn score_completed_round(
    current: RoundScoreSnapshot,
    previous: RoundScoreSnapshot,
    possible: u64,
) -> Option<ExerciseResult> {
    if possible == 0 || current.correct < previous.correct || current.mistakes < previous.mistakes {
        return None;
    }
    let earned = current.correct - previous.correct;
    if earned > possible {
        return None;
    }
    Some(ExerciseResult {
        earned,
        possible,
        mistakes: current.mistakes - previous.mistakes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::practice_draft::{PracticeDraft, SessionLength};

    fn started() -> PlannedRuntime {
        let mut runtime = PlannedRuntime::default();
        let mut draft = PracticeDraft::default();
        draft.length = SessionLength::Fixed(2);
        runtime.start(&draft).unwrap();
        runtime
    }

    #[test]
    fn tickets_bind_exact_generation_instance_and_definition() {
        let runtime = started();
        let ticket = PlannedRoundTicket::from_runtime(&runtime).unwrap();
        assert_eq!(ticket.validate(&runtime), Ok(()));
        assert_eq!(
            PlannedRoundTicket {
                generation: 0,
                ..ticket.clone()
            }
            .validate(&runtime),
            Err(HandoffError::StaleGeneration)
        );
        assert_eq!(
            PlannedRoundTicket {
                instance_id: "wrong".into(),
                ..ticket.clone()
            }
            .validate(&runtime),
            Err(HandoffError::WrongInstance)
        );
        assert_eq!(
            PlannedRoundTicket {
                definition_id: "matching",
                ..ticket.clone()
            }
            .validate(&runtime),
            Err(HandoffError::WrongDefinition)
        );
    }

    #[test]
    fn publishing_requires_acknowledgement_before_next_ticket() {
        let mut runtime = started();
        let ticket = PlannedRoundTicket::from_runtime(&runtime).unwrap();
        runtime
            .publish(
                ticket.generation,
                &ticket.instance_id,
                ticket.definition_id,
                "prompt",
                ExerciseResult {
                    earned: 1,
                    possible: 1,
                    mistakes: 0,
                },
            )
            .unwrap();
        assert_eq!(ticket.validate(&runtime), Err(HandoffError::ResultPending));
        runtime
            .acknowledge(ticket.generation, &ticket.instance_id)
            .unwrap();
        assert_eq!(ticket.validate(&runtime), Err(HandoffError::WrongInstance));
        assert!(
            PlannedRoundTicket::from_runtime(&runtime)
                .unwrap()
                .validate(&runtime)
                .is_ok()
        );
    }

    #[test]
    fn cancelled_ticket_cannot_be_replayed() {
        let mut runtime = started();
        let ticket = PlannedRoundTicket::from_runtime(&runtime).unwrap();
        runtime.cancel(ticket.generation).unwrap();
        assert_eq!(
            ticket.validate(&runtime),
            Err(HandoffError::StaleGeneration)
        );
    }

    #[test]
    fn scored_delta_rejects_invalid_cumulative_counters() {
        let before = RoundScoreSnapshot {
            correct: 8,
            mistakes: 3,
        };
        assert_eq!(
            score_completed_round(
                RoundScoreSnapshot {
                    correct: 11,
                    mistakes: 5
                },
                before,
                4
            ),
            Some(ExerciseResult {
                earned: 3,
                possible: 4,
                mistakes: 2
            })
        );
        assert_eq!(
            score_completed_round(
                RoundScoreSnapshot {
                    correct: 12,
                    mistakes: 3
                },
                before,
                3
            ),
            None
        );
        assert_eq!(
            score_completed_round(
                RoundScoreSnapshot {
                    correct: 7,
                    mistakes: 4
                },
                before,
                4
            ),
            None
        );
        assert_eq!(
            score_completed_round(
                RoundScoreSnapshot {
                    correct: 8,
                    mistakes: 2
                },
                before,
                4
            ),
            None
        );
        assert_eq!(score_completed_round(before, before, 0), None);
    }
}
