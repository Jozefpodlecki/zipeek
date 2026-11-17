use crate::definition::DEFINITIONS;
use crate::practice_draft::{DraftError, PracticeDraft, SessionLength};
use crate::session::ExerciseResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedExercise {
    pub ordinal: u16,
    pub definition_id: &'static str,
    pub instance_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStatus {
    Ready,
    AwaitingAcknowledgement,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    Draft(DraftError),
    EndlessRequiresIncrementalScheduling,
    NoCurrentExercise,
    AlreadySubmitted,
    NotSubmitted,
    WrongInstance,
    InvalidResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPlan {
    pub steps: Vec<PlannedExercise>,
    pub cursor: usize,
    pub status: PlanStatus,
    pending_result: Option<ExerciseResult>,
}

impl SessionPlan {
    pub fn from_draft(draft: &PracticeDraft) -> Result<Self, PlanError> {
        draft
            .validate_for_scored_session()
            .map_err(PlanError::Draft)?;
        let count = match draft.length {
            SessionLength::Fixed(count) => count,
            SessionLength::Endless => return Err(PlanError::EndlessRequiresIncrementalScheduling),
        };
        let selected: Vec<&'static str> = DEFINITIONS
            .iter()
            .filter(|definition| draft.exercise_ids.contains(definition.id))
            .map(|definition| definition.id)
            .collect();
        let steps = (0..count)
            .map(|index| {
                let definition_id = selected[usize::from(index) % selected.len()];
                PlannedExercise {
                    ordinal: index,
                    definition_id,
                    instance_id: format!("planned-{index}-{definition_id}"),
                }
            })
            .collect();
        Ok(Self {
            steps,
            cursor: 0,
            status: PlanStatus::Ready,
            pending_result: None,
        })
    }

    pub fn current(&self) -> Option<&PlannedExercise> {
        if self.status == PlanStatus::Completed {
            None
        } else {
            self.steps.get(self.cursor)
        }
    }

    pub fn submit(&mut self, instance_id: &str, result: ExerciseResult) -> Result<(), PlanError> {
        let current = self.current().ok_or(PlanError::NoCurrentExercise)?;
        if current.instance_id != instance_id {
            return Err(PlanError::WrongInstance);
        }
        if self.status == PlanStatus::AwaitingAcknowledgement {
            return Err(PlanError::AlreadySubmitted);
        }
        if result.possible == 0 || result.earned > result.possible {
            return Err(PlanError::InvalidResult);
        }
        self.pending_result = Some(result);
        self.status = PlanStatus::AwaitingAcknowledgement;
        Ok(())
    }

    pub fn acknowledge(&mut self, instance_id: &str) -> Result<bool, PlanError> {
        let current = self.current().ok_or(PlanError::NoCurrentExercise)?;
        if current.instance_id != instance_id {
            return Err(PlanError::WrongInstance);
        }
        if self.status != PlanStatus::AwaitingAcknowledgement || self.pending_result.is_none() {
            return Err(PlanError::NotSubmitted);
        }
        self.pending_result = None;
        self.cursor += 1;
        self.status = if self.cursor == self.steps.len() {
            PlanStatus::Completed
        } else {
            PlanStatus::Ready
        };
        Ok(self.status == PlanStatus::Completed)
    }

    pub fn pending_result(&self) -> Option<ExerciseResult> {
        self.pending_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_listening_never_enters_the_legacy_scored_plan() {
        let mut draft = PracticeDraft::default();
        draft.toggle_exercise("listening-choice").unwrap();
        assert!(draft.supports_local_vocabulary_session());
        assert_eq!(
            SessionPlan::from_draft(&draft),
            Err(PlanError::Draft(DraftError::UnsupportedAssessment(
                "listening-choice".to_owned()
            )))
        );
    }

    #[test]
    fn fixed_plan_uses_registry_order_and_stable_instances() {
        let mut draft = PracticeDraft::default();
        draft.toggle_exercise("matching").unwrap();
        draft.length = SessionLength::Fixed(5);
        let first = SessionPlan::from_draft(&draft).unwrap();
        let second = SessionPlan::from_draft(&draft).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first
                .steps
                .iter()
                .map(|step| step.definition_id)
                .collect::<Vec<_>>(),
            [
                "matching",
                "multiple-choice",
                "matching",
                "multiple-choice",
                "matching"
            ]
        );
    }

    #[test]
    fn rejected_or_unacknowledged_results_do_not_advance() {
        let draft = PracticeDraft::default();
        let mut plan = SessionPlan::from_draft(&draft).unwrap();
        let id = plan.current().unwrap().instance_id.clone();
        let score = ExerciseResult {
            earned: 1,
            possible: 1,
            mistakes: 0,
        };
        assert_eq!(plan.acknowledge(&id), Err(PlanError::NotSubmitted));
        assert_eq!(plan.submit("wrong", score), Err(PlanError::WrongInstance));
        assert_eq!(
            plan.submit(
                &id,
                ExerciseResult {
                    earned: 2,
                    possible: 1,
                    mistakes: 0
                }
            ),
            Err(PlanError::InvalidResult)
        );
        plan.submit(&id, score).unwrap();
        assert_eq!(plan.cursor, 0);
        assert_eq!(plan.submit(&id, score), Err(PlanError::AlreadySubmitted));
        assert_eq!(plan.acknowledge("wrong"), Err(PlanError::WrongInstance));
        assert_eq!(plan.cursor, 0);
        assert_eq!(plan.pending_result(), Some(score));
        assert_eq!(plan.acknowledge(&id), Ok(false));
        assert_eq!(plan.cursor, 1);
        assert_eq!(plan.acknowledge(&id), Err(PlanError::WrongInstance));
    }

    #[test]
    fn final_acknowledgement_finishes_plan() {
        let mut draft = PracticeDraft::default();
        draft.length = SessionLength::Fixed(1);
        let mut plan = SessionPlan::from_draft(&draft).unwrap();
        let id = plan.current().unwrap().instance_id.clone();
        plan.submit(
            &id,
            ExerciseResult {
                earned: 0,
                possible: 1,
                mistakes: 1,
            },
        )
        .unwrap();
        assert_eq!(plan.acknowledge(&id), Ok(true));
        assert_eq!(plan.status, PlanStatus::Completed);
        assert!(plan.current().is_none());
    }

    #[test]
    fn rejects_unsupported_or_endless_configuration() {
        let mut draft = PracticeDraft::default();
        draft.length = SessionLength::Endless;
        assert_eq!(
            SessionPlan::from_draft(&draft),
            Err(PlanError::EndlessRequiresIncrementalScheduling)
        );
        draft.length = SessionLength::Fixed(2);
        draft.toggle_exercise("speaking").unwrap();
        assert_eq!(
            SessionPlan::from_draft(&draft),
            Err(PlanError::Draft(DraftError::UnsupportedAssessment(
                "speaking".into()
            )))
        );
    }
}
