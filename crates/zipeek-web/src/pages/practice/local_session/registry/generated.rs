use super::super::super::generated_session::GeneratedPractice;
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct GeneratedDefinition;

impl ExerciseDefinition for GeneratedDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &[
            "sentence-translation",
            "image-vocabulary",
            "story-builder",
            "reading-comprehension",
            "conversation-practice",
        ]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let step = context.step;
        let on_finish = context.completed_callback();
        let on_scored = context.binary_score_callback();
        {
            if let Some(raw) = &step.generated_instance {
                html! { <GeneratedPractice
                    key={step.id.clone()}
                    raw={raw.clone()}
                    exercise_id={step.exercise_id.clone()}
                    seed={step.instance_seed}
                    on_completed={on_finish.clone()}
                    on_scored={on_scored.clone()}
                /> }
            } else {
                html! { <p role="alert">{"Generated exercise content is missing."}</p> }
            }
        }
    }
}
