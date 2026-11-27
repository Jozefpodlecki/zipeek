use std::collections::HashMap;
use yew::prelude::*;

use crate::services::practice_session_store::{
    ExerciseAssessment, ExerciseOutcome, PracticeSession, PracticeStep,
};

pub(super) struct ExerciseRenderContext<'a> {
    pub saved: &'a PracticeSession,
    pub step: &'a PracticeStep,
    pub query: &'a HashMap<String, String>,
    pub on_outcome: &'a Callback<ExerciseOutcome>,
    pub on_hanzi_ready_change: &'a Callback<bool>,
}

impl ExerciseRenderContext<'_> {
    pub fn completed_callback(&self) -> Callback<()> {
        let on_outcome = self.on_outcome.clone();
        Callback::from(move |()| on_outcome.emit(ExerciseOutcome::Completed))
    }

    /// Preferred boundary for evaluators that have a real verdict and full
    /// attempt evidence. The registry does not decide gameplay correctness.
    pub fn assessment_callback(&self) -> Callback<ExerciseAssessment> {
        let on_outcome = self.on_outcome.clone();
        Callback::from(move |assessment| on_outcome.emit(ExerciseOutcome::Assessed(assessment)))
    }

    /// Compatibility boundary for currently binary-only generated evaluators.
    /// Completion-only activities must continue to use `completed_callback`.
    pub fn binary_score_callback(&self) -> Callback<bool> {
        let on_assessment = self.assessment_callback();
        Callback::from(move |correct| on_assessment.emit(ExerciseAssessment::binary(correct)))
    }
}

pub(super) trait ExerciseDefinition: Sync {
    fn ids(&self) -> &'static [&'static str];
    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html;
}

mod authored;
mod character_hunt;
mod generated;
mod listening_choice;
mod quiz;
mod sentence_builder;
mod sentence_transformation;
mod speaking;
mod spot_mistake;
mod vocabulary;
mod write_hanzi;

use authored::AuthoredDefinition;
use character_hunt::CharacterHuntDefinition;
use generated::GeneratedDefinition;
use listening_choice::ListeningChoiceDefinition;
use quiz::QuizDefinition;
use sentence_builder::SentenceBuilderDefinition;
use sentence_transformation::SentenceTransformationDefinition;
use speaking::SpeakingDefinition;
use spot_mistake::SpotMistakeDefinition;
use vocabulary::VocabularyDefinition;
use write_hanzi::WriteHanziDefinition;

static DEFINITIONS: &[&dyn ExerciseDefinition] = &[
    &VocabularyDefinition,
    &GeneratedDefinition,
    &QuizDefinition,
    &CharacterHuntDefinition,
    &ListeningChoiceDefinition,
    &SpeakingDefinition,
    &WriteHanziDefinition,
    &SpotMistakeDefinition,
    &SentenceTransformationDefinition,
    &SentenceBuilderDefinition,
    &AuthoredDefinition,
];

pub(super) fn render_registered(context: &ExerciseRenderContext<'_>) -> Html {
    let id = context.step.exercise_id.as_str();
    match DEFINITIONS
        .iter()
        .find(|definition| definition.ids().contains(&id))
    {
        Some(definition) => definition.render(context),
        None => html! {
            <p role="status" class="text-sm text-muted">
                {"This exercise cannot run in the resumable session yet. You may skip it without earning a score, or return to setup to choose a supported exercise."}
            </p>
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn outcome_callbacks_preserve_unscored_and_binary_results() {
        use std::{cell::RefCell, rc::Rc};

        let observed = Rc::new(RefCell::new(Vec::new()));
        let received = observed.clone();
        let on_outcome = Callback::from(move |outcome| received.borrow_mut().push(outcome));
        let saved = PracticeSession::new(
            uuid::Uuid::now_v7().to_string(),
            &["matching".to_owned()].into_iter().collect(),
            1,
            42,
            Default::default(),
        )
        .expect("valid test session");
        let step = saved.current_step().expect("active step");
        let query = HashMap::new();
        let ready = Callback::from(|_: bool| {});
        let context = ExerciseRenderContext {
            saved: &saved,
            step,
            query: &query,
            on_outcome: &on_outcome,
            on_hanzi_ready_change: &ready,
        };
        context.completed_callback().emit(());
        context.binary_score_callback().emit(false);
        context.binary_score_callback().emit(true);
        assert_eq!(
            *observed.borrow(),
            vec![
                ExerciseOutcome::Completed,
                ExerciseOutcome::Assessed(ExerciseAssessment::binary(false)),
                ExerciseOutcome::Assessed(ExerciseAssessment::binary(true)),
            ]
        );
    }

    #[test]
    fn assessment_boundary_keeps_partial_and_retry_evidence() {
        use crate::services::practice_session_store::ExerciseVerdict;
        use std::{cell::RefCell, rc::Rc};
        let observed = Rc::new(RefCell::new(Vec::new()));
        let sink = observed.clone();
        let on_outcome = Callback::from(move |result| sink.borrow_mut().push(result));
        let saved = PracticeSession::new(
            uuid::Uuid::now_v7().to_string(),
            &["matching".to_owned()].into_iter().collect(),
            1,
            42,
            Default::default(),
        )
        .unwrap();
        let query = HashMap::new();
        let ready = Callback::from(|_: bool| {});
        let context = ExerciseRenderContext {
            saved: &saved,
            step: saved.current_step().unwrap(),
            query: &query,
            on_outcome: &on_outcome,
            on_hanzi_ready_change: &ready,
        };
        let assessment = ExerciseAssessment {
            verdict: ExerciseVerdict::PartiallyCorrect,
            attempts: 3,
            mistakes: 1,
            hints: 1,
            assisted: true,
            item_evidence: Vec::new(),
        };
        context.assessment_callback().emit(assessment.clone());
        assert_eq!(
            *observed.borrow(),
            vec![ExerciseOutcome::Assessed(assessment)]
        );
    }

    #[test]
    fn exercise_ids_are_unique_and_nonempty() {
        let mut ids = HashSet::new();
        for definition in DEFINITIONS {
            for id in definition.ids() {
                assert!(!id.is_empty());
                assert!(ids.insert(*id), "duplicate exercise registration: {id}");
            }
        }
    }
}
