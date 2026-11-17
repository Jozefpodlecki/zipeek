use super::{ExerciseStage, PracticeSetupFlow};
use crate::components::practice::setup::practice_flow_navigation::PracticeStep;

pub(super) fn select_step(flow: &mut PracticeSetupFlow, step: PracticeStep) -> bool {
    if step.index() > flow.furthest.index() {
        return false;
    }
    flow.step = step;
    true
}

pub(super) fn select_exercise_stage(flow: &mut PracticeSetupFlow, stage: ExerciseStage) -> bool {
    if flow.step != PracticeStep::Exercises {
        return false;
    }
    if stage == ExerciseStage::Configure && flow.draft.exercise_ids.is_empty() {
        return false;
    }
    flow.exercise_stage = stage;
    true
}

pub(super) fn back(flow: &mut PracticeSetupFlow) {
    if flow.step == PracticeStep::Exercises && flow.exercise_stage == ExerciseStage::Configure {
        flow.exercise_stage = ExerciseStage::Choose;
    } else if let Some(previous) = flow.step.previous() {
        flow.step = previous;
    }
}

pub(super) fn continue_to_next_step(flow: &mut PracticeSetupFlow) -> bool {
    if flow.step == PracticeStep::Exercises {
        if flow.draft.exercise_ids.is_empty() {
            return false;
        }
        flow.step = PracticeStep::Vocabulary;
    } else if let Some(following) = flow.step.next() {
        flow.step = following;
    }
    if flow.step.index() > flow.furthest.index() {
        flow.furthest = flow.step;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_is_unavailable_without_exercises() {
        let mut flow = PracticeSetupFlow::with_source(None);
        flow.step = PracticeStep::Exercises;
        flow.draft.exercise_ids.clear();
        assert!(!select_exercise_stage(&mut flow, ExerciseStage::Configure));
        assert_eq!(flow.exercise_stage, ExerciseStage::Choose);
        assert!(!continue_to_next_step(&mut flow));
        assert_eq!(flow.step, PracticeStep::Exercises);
    }

    #[test]
    fn back_from_configuration_returns_to_selection() {
        let mut flow = PracticeSetupFlow::with_source(None);
        flow.step = PracticeStep::Exercises;
        flow.exercise_stage = ExerciseStage::Configure;
        back(&mut flow);
        assert_eq!(flow.step, PracticeStep::Exercises);
        assert_eq!(flow.exercise_stage, ExerciseStage::Choose);
    }
}
