mod image;
mod order;
mod translation;

use crate::services::practice_generated::GeneratedInstance;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GeneratedPracticeProps {
    pub raw: String,
    pub exercise_id: String,
    pub seed: u64,
    pub on_completed: Callback<()>,
    pub on_scored: Callback<bool>,
}

#[function_component(GeneratedPractice)]
pub fn generated_practice(props: &GeneratedPracticeProps) -> Html {
    let body = match GeneratedInstance::decode(&props.raw, &props.exercise_id) {
        Err(error) => {
            html! { <p role="alert">{format!("This generated exercise is invalid: {error}")}</p> }
        }
        Ok(GeneratedInstance::Matching { .. }) => html! { <p role="alert">{"Matching content must be opened with the Matching exercise renderer."}</p> },
        Ok(GeneratedInstance::SentenceTranslation { sentence }) => html! {
            <translation::TranslationPractice
                key={format!("{}:{}", props.seed, props.raw)}
                sentence={sentence}
                seed={props.seed}
                on_scored={props.on_scored.clone()}
            />
        },
        Ok(GeneratedInstance::ImageVocabulary {
            choices,
            correct_ids,
            image_url,
            scene,
        }) => html! {
            <image::ImagePractice
                key={props.raw.clone()}
                choices={choices}
                correct_ids={correct_ids}
                image_url={image_url}
                scene={scene}
                on_completed={props.on_completed.clone()}
            />
        },
    };
    html! { <section class="space-y-4">{body}</section> }
}
