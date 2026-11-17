use crate::planned_handoff::{HandoffError, PlannedRoundTicket};
use crate::planned_session::{PlannedSession, PlannedSessionError};
use crate::planner::PlannedExercise;
use crate::practice_draft::PracticeDraft;
use crate::session::ExerciseResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    AlreadyActive,
    NoActiveSession,
    StaleGeneration,
    Coordinator(PlannedSessionError),
    InvalidTicket(HandoffError),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlannedRuntime {
    generation: u64,
    active: Option<PlannedSession>,
}

impl PlannedRuntime {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn active(&self) -> Option<&PlannedSession> {
        self.active.as_ref()
    }

    pub fn current(&self) -> Option<&PlannedExercise> {
        self.active.as_ref().and_then(PlannedSession::current)
    }

    pub fn start(&mut self, draft: &PracticeDraft) -> Result<u64, RuntimeError> {
        if self
            .active
            .as_ref()
            .is_some_and(|session| !session.is_completed())
        {
            return Err(RuntimeError::AlreadyActive);
        }
        let generation = self.generation.wrapping_add(1);
        let session = PlannedSession::new(draft, format!("planned-session-{generation}"))
            .map_err(RuntimeError::Coordinator)?;
        self.generation = generation;
        self.active = Some(session);
        Ok(generation)
    }

    pub fn publish(
        &mut self,
        generation: u64,
        instance_id: &str,
        definition_id: &'static str,
        visible_input: impl Into<String>,
        result: ExerciseResult,
    ) -> Result<(), RuntimeError> {
        self.verify_generation(generation)?;
        self.active
            .as_mut()
            .ok_or(RuntimeError::NoActiveSession)?
            .publish(instance_id, definition_id, visible_input, result)
            .map_err(RuntimeError::Coordinator)
    }

    pub fn publish_ticket(
        &mut self,
        ticket: &PlannedRoundTicket,
        visible_input: impl Into<String>,
        result: ExerciseResult,
    ) -> Result<(), RuntimeError> {
        ticket.validate(self).map_err(RuntimeError::InvalidTicket)?;
        self.publish(
            ticket.generation,
            &ticket.instance_id,
            ticket.definition_id,
            visible_input,
            result,
        )
    }

    pub fn acknowledge(
        &mut self,
        generation: u64,
        instance_id: &str,
    ) -> Result<bool, RuntimeError> {
        self.verify_generation(generation)?;
        self.active
            .as_mut()
            .ok_or(RuntimeError::NoActiveSession)?
            .acknowledge(instance_id)
            .map_err(RuntimeError::Coordinator)
    }

    pub fn cancel(&mut self, generation: u64) -> Result<(), RuntimeError> {
        self.verify_generation(generation)?;
        if self.active.is_none() {
            return Err(RuntimeError::NoActiveSession);
        }
        self.active = None;
        self.generation = self.generation.wrapping_add(1);
        Ok(())
    }

    fn verify_generation(&self, generation: u64) -> Result<(), RuntimeError> {
        if generation != self.generation {
            Err(RuntimeError::StaleGeneration)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::practice_draft::SessionLength;

    fn draft() -> PracticeDraft {
        let mut draft = PracticeDraft::default();
        draft.length = SessionLength::Fixed(2);
        draft
    }

    fn score() -> ExerciseResult {
        ExerciseResult {
            earned: 1,
            possible: 2,
            mistakes: 1,
        }
    }

    #[test]
    fn fixed_five_session_reaches_completed_after_fifth_result() {
        let mut runtime = PlannedRuntime::default();
        let mut draft = draft();
        draft.length = SessionLength::Fixed(5);
        let generation = runtime.start(&draft).unwrap();
        for index in 0..5 {
            let ticket = PlannedRoundTicket::from_runtime(&runtime).unwrap();
            runtime
                .publish_ticket(&ticket, format!("round {index}"), score())
                .unwrap();
            let completed = runtime
                .acknowledge(generation, &ticket.instance_id)
                .unwrap();
            assert_eq!(completed, index == 4);
        }
        assert!(
            runtime
                .active()
                .is_some_and(|session| session.is_completed())
        );
        assert!(runtime.current().is_none());
    }

    #[test]
    fn ticket_publication_rejects_pending_stale_and_wrong_definition() {
        let mut runtime = PlannedRuntime::default();
        let generation = runtime.start(&draft()).unwrap();
        let ticket = PlannedRoundTicket::from_runtime(&runtime).unwrap();
        assert_eq!(
            runtime.publish_ticket(
                &PlannedRoundTicket {
                    definition_id: "not-this-exercise",
                    ..ticket.clone()
                },
                "",
                score()
            ),
            Err(RuntimeError::InvalidTicket(HandoffError::WrongDefinition))
        );
        assert_eq!(
            runtime.publish_ticket(&ticket, "test prompt", score()),
            Ok(())
        );
        assert_eq!(
            runtime.publish_ticket(&ticket, "again", score()),
            Err(RuntimeError::InvalidTicket(HandoffError::ResultPending))
        );
        runtime
            .acknowledge(generation, &ticket.instance_id)
            .unwrap();
        assert_eq!(
            runtime.publish_ticket(&ticket, "old", score()),
            Err(RuntimeError::InvalidTicket(HandoffError::WrongInstance))
        );
        runtime.cancel(generation).unwrap();
        assert_eq!(
            runtime.publish_ticket(&ticket, "stale", score()),
            Err(RuntimeError::InvalidTicket(HandoffError::StaleGeneration))
        );
    }

    #[test]
    fn valid_plan_rejects_overlapping_start_and_preserves_state() {
        let mut runtime = PlannedRuntime::default();
        let generation = runtime.start(&draft()).unwrap();
        let original = runtime.clone();
        assert_eq!(runtime.start(&draft()), Err(RuntimeError::AlreadyActive));
        assert_eq!(runtime, original);
        assert_eq!(generation, 1);
    }

    #[test]
    fn invalid_draft_does_not_advance_generation() {
        let mut runtime = PlannedRuntime::default();
        let mut invalid = draft();
        invalid.length = SessionLength::Endless;
        assert!(matches!(
            runtime.start(&invalid),
            Err(RuntimeError::Coordinator(_))
        ));
        assert_eq!(runtime.generation(), 0);
        assert!(runtime.active().is_none());
    }

    #[test]
    fn acknowledgement_is_required_and_last_step_finishes() {
        let mut runtime = PlannedRuntime::default();
        let generation = runtime.start(&draft()).unwrap();
        for i in 0..2 {
            let step = runtime.current().unwrap().clone();
            assert!(matches!(
                runtime.acknowledge(generation, &step.instance_id),
                Err(RuntimeError::Coordinator(_))
            ));
            runtime
                .publish(
                    generation,
                    &step.instance_id,
                    step.definition_id,
                    "visible",
                    score(),
                )
                .unwrap();
            assert_eq!(
                runtime.acknowledge(generation, &step.instance_id),
                Ok(i == 1)
            );
        }
        assert!(runtime.active().unwrap().is_completed());
        assert_eq!(runtime.active().unwrap().ledger().entries.len(), 2);
    }

    #[test]
    fn duplicate_and_stale_messages_never_mutate_state() {
        let mut runtime = PlannedRuntime::default();
        let generation = runtime.start(&draft()).unwrap();
        let step = runtime.current().unwrap().clone();
        runtime
            .publish(
                generation,
                &step.instance_id,
                step.definition_id,
                "visible",
                score(),
            )
            .unwrap();
        let pending = runtime.clone();
        assert!(matches!(
            runtime.publish(
                generation,
                &step.instance_id,
                step.definition_id,
                "visible",
                score()
            ),
            Err(RuntimeError::Coordinator(_))
        ));
        assert_eq!(runtime, pending);
        runtime.cancel(generation).unwrap();
        let after_cancel = runtime.clone();
        assert_eq!(
            runtime.publish(
                generation,
                &step.instance_id,
                step.definition_id,
                "visible",
                score()
            ),
            Err(RuntimeError::StaleGeneration)
        );
        assert_eq!(
            runtime.acknowledge(generation, &step.instance_id),
            Err(RuntimeError::StaleGeneration)
        );
        assert_eq!(runtime, after_cancel);
        let next = runtime.start(&draft()).unwrap();
        assert!(next > generation);
        assert_eq!(
            runtime.publish(
                generation,
                &step.instance_id,
                step.definition_id,
                "visible",
                score()
            ),
            Err(RuntimeError::StaleGeneration)
        );
    }

    #[test]
    fn invalid_identity_cannot_mutate_the_active_plan() {
        let mut runtime = PlannedRuntime::default();
        let generation = runtime.start(&draft()).unwrap();
        let step = runtime.current().unwrap().clone();
        let original = runtime.clone();
        assert!(matches!(
            runtime.publish(generation, "other", step.definition_id, "", score()),
            Err(RuntimeError::Coordinator(_))
        ));
        assert!(matches!(
            runtime.publish(generation, &step.instance_id, "matching", "", score()),
            Err(RuntimeError::Coordinator(_))
        ));
        assert_eq!(runtime, original);
    }
}
