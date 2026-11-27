use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct SpeakingDefinition;

impl ExerciseDefinition for SpeakingDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["speaking"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let step = context.step;
        let on_finish = context.completed_callback();
        html! {
            <crate::pages::translation::speaking::SpeakingPractice
                key={step.id.clone()}
                seed={step.instance_seed}
                on_completed={Some(on_finish.clone())}
            />
        }
    }
}
