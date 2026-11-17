use crate::planner::{PlanError, PlanStatus, PlannedExercise, SessionPlan};
use crate::practice_draft::PracticeDraft;
use crate::session::{ExerciseInstance, ExerciseResult, QuizSession, SessionError, SessionStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedSessionError {
    Plan(PlanError),
    Ledger(SessionError),
    WrongInstance,
    WrongDefinition,
    NotAwaitingAcknowledgement,
    AlreadyCompleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSession {
    plan: SessionPlan,
    ledger: QuizSession,
}

impl PlannedSession {
    pub fn new(
        draft: &PracticeDraft,
        session_id: impl Into<String>,
    ) -> Result<Self, PlannedSessionError> {
        let plan = SessionPlan::from_draft(draft).map_err(PlannedSessionError::Plan)?;
        Ok(Self {
            plan,
            ledger: QuizSession::new(session_id),
        })
    }

    pub fn current(&self) -> Option<&PlannedExercise> {
        self.plan.current()
    }

    pub fn plan(&self) -> &SessionPlan {
        &self.plan
    }

    pub fn ledger(&self) -> &QuizSession {
        &self.ledger
    }

    pub fn is_completed(&self) -> bool {
        self.plan.status == PlanStatus::Completed && self.ledger.status == SessionStatus::Completed
    }

    pub fn publish(
        &mut self,
        instance_id: &str,
        definition_id: &'static str,
        visible_input: impl Into<String>,
        result: ExerciseResult,
    ) -> Result<(), PlannedSessionError> {
        let step = self
            .current()
            .ok_or(PlannedSessionError::AlreadyCompleted)?;
        if step.instance_id != instance_id {
            return Err(PlannedSessionError::WrongInstance);
        }
        if step.definition_id != definition_id {
            return Err(PlannedSessionError::WrongDefinition);
        }
        let mut next = self.clone();
        next.plan
            .submit(instance_id, result)
            .map_err(PlannedSessionError::Plan)?;
        let instance = ExerciseInstance::new(instance_id, definition_id, visible_input)
            .map_err(PlannedSessionError::Ledger)?;
        next.ledger
            .record_completed(instance, result)
            .map_err(PlannedSessionError::Ledger)?;
        *self = next;
        Ok(())
    }

    pub fn acknowledge(&mut self, instance_id: &str) -> Result<bool, PlannedSessionError> {
        if self.plan.status != PlanStatus::AwaitingAcknowledgement {
            return Err(PlannedSessionError::NotAwaitingAcknowledgement);
        }
        let mut next = self.clone();
        let completed = next
            .plan
            .acknowledge(instance_id)
            .map_err(PlannedSessionError::Plan)?;
        if completed {
            next.ledger
                .complete()
                .map_err(PlannedSessionError::Ledger)?;
        }
        *self = next;
        Ok(completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::practice_draft::SessionLength;

    fn mixed_draft() -> PracticeDraft {
        let mut draft = PracticeDraft::default();
        draft.toggle_exercise("matching").unwrap();
        draft.length = SessionLength::Fixed(3);
        draft
    }

    fn score(earned: u64, possible: u64) -> ExerciseResult {
        ExerciseResult {
            earned,
            possible,
            mistakes: possible - earned,
        }
    }

    #[test]
    fn results_require_acknowledgement_before_next_step() {
        let mut session = PlannedSession::new(&mixed_draft(), "fixed-1").unwrap();
        let first = session.current().unwrap().clone();
        assert_eq!(first.definition_id, "matching");
        assert_eq!(
            session.acknowledge(&first.instance_id),
            Err(PlannedSessionError::NotAwaitingAcknowledgement)
        );
        session
            .publish(
                &first.instance_id,
                first.definition_id,
                "prompt",
                score(3, 4),
            )
            .unwrap();
        assert_eq!(session.current(), Some(&first));
        assert_eq!(session.plan().status, PlanStatus::AwaitingAcknowledgement);
        assert_eq!(session.ledger().entries.len(), 1);
        assert_eq!(
            session.publish(
                &first.instance_id,
                first.definition_id,
                "prompt",
                score(3, 4)
            ),
            Err(PlannedSessionError::Plan(PlanError::AlreadySubmitted))
        );
        assert_eq!(session.acknowledge(&first.instance_id), Ok(false));
        assert_eq!(session.current().unwrap().definition_id, "multiple-choice");
        assert_eq!(session.ledger().totals(), score(3, 4));
    }

    #[test]
    fn invalid_messages_leave_both_authorities_unchanged() {
        let mut session = PlannedSession::new(&mixed_draft(), "fixed-2").unwrap();
        let first = session.current().unwrap().clone();
        let before = session.clone();
        assert_eq!(
            session.publish("other", first.definition_id, "", score(1, 1)),
            Err(PlannedSessionError::WrongInstance)
        );
        assert_eq!(
            session.publish(&first.instance_id, "multiple-choice", "", score(1, 1)),
            Err(PlannedSessionError::WrongDefinition)
        );
        assert_eq!(
            session.publish(
                &first.instance_id,
                first.definition_id,
                "",
                ExerciseResult {
                    earned: 2,
                    possible: 1,
                    mistakes: 0
                }
            ),
            Err(PlannedSessionError::Plan(PlanError::InvalidResult))
        );
        assert_eq!(session, before);
        session
            .publish(&first.instance_id, first.definition_id, "", score(1, 1))
            .unwrap();
        let pending = session.clone();
        assert_eq!(
            session.acknowledge("wrong"),
            Err(PlannedSessionError::Plan(PlanError::WrongInstance))
        );
        assert_eq!(session, pending);
    }

    #[test]
    fn final_acknowledgement_completes_ledger_and_plan_once() {
        let mut draft = PracticeDraft::default();
        draft.length = SessionLength::Fixed(2);
        let mut session = PlannedSession::new(&draft, "fixed-3").unwrap();
        for ordinal in 0..2 {
            let step = session.current().unwrap().clone();
            session
                .publish(
                    &step.instance_id,
                    step.definition_id,
                    "question",
                    score(1, 1),
                )
                .unwrap();
            assert_eq!(session.acknowledge(&step.instance_id), Ok(ordinal == 1));
        }
        assert!(session.is_completed());
        assert!(session.current().is_none());
        assert_eq!(session.ledger().entries.len(), 2);
        assert_eq!(session.ledger().totals().earned, 2);
        assert_eq!(
            session.acknowledge("unused"),
            Err(PlannedSessionError::NotAwaitingAcknowledgement)
        );
        assert_eq!(
            session.publish("unused", "multiple-choice", "", score(1, 1)),
            Err(PlannedSessionError::AlreadyCompleted)
        );
    }

    #[test]
    fn unsupported_draft_never_starts() {
        let mut draft = PracticeDraft::default();
        draft.length = SessionLength::Endless;
        assert_eq!(
            PlannedSession::new(&draft, "invalid"),
            Err(PlannedSessionError::Plan(
                PlanError::EndlessRequiresIncrementalScheduling
            ))
        );
    }
}
