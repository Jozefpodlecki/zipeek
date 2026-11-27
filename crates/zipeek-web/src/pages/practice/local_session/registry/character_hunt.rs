use super::{ExerciseDefinition, ExerciseRenderContext};
use crate::pages::learning::games::ResumableCharacterHunt;
use yew::prelude::*;

pub(super) struct CharacterHuntDefinition;

impl ExerciseDefinition for CharacterHuntDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["character-hunt"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let step = context.step;
        let query = context.query.clone();
        let on_finish = context.completed_callback();
        html! {
            <ResumableCharacterHunt
                key={step.id.clone()}
                vocabulary={query.clone()}
                seed={step.instance_seed}
                on_completed={on_finish.clone()}
            />
        }
    }
}
