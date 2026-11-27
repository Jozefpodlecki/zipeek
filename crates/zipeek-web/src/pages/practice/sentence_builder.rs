use yew::prelude::*;
use zipeek_exercises::sentence_builder::{PROMPTS, validate};

use crate::{models::SentenceBuilderOptions, services::practice_session_store::ExerciseAssessment};

mod exercise;
mod tile;

#[derive(Properties, PartialEq)]
pub(crate) struct SentenceBuilderProps {
    pub seed: u64,
    pub prompt_index: usize,
    pub options: SentenceBuilderOptions,
    pub on_completed: Callback<()>,
    #[prop_or_default]
    pub on_assessed: Option<Callback<ExerciseAssessment>>,
}

#[function_component(SentenceBuilder)]
pub(crate) fn sentence_builder(props: &SentenceBuilderProps) -> Html {
    let prompt = props
        .prompt_index
        .checked_rem(PROMPTS.len())
        .and_then(|index| PROMPTS.get(index));
    let Some(prompt) = prompt.filter(|prompt| validate(prompt)) else {
        return html! { <p role="alert">{"This sentence could not be loaded. Leave Practice and try again."}</p> };
    };
    let identity = format!(
        "{}:{}:{}:{}",
        props.seed,
        props.prompt_index,
        props.options.automatic_check,
        props.options.retry_incorrect
    );
    html! {
        <exercise::SentenceBuilderExercise key={identity} seed={props.seed} prompt={prompt}
            options={props.options} on_completed={props.on_completed.clone()} on_assessed={props.on_assessed.clone()} />
    }
}
