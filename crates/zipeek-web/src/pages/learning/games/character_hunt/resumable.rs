use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::{
    hooks::use_dictionary,
    pages::practice::{instruction::InstructionIcon, surface::ExerciseSurface},
};
use yew::prelude::*;
use zipeek_exercises::character_hunt::Candidate;

use super::{config::HuntConfig, session::HuntSession, vocabulary::load_candidates};

#[derive(Properties, PartialEq)]
pub struct ResumableCharacterHuntProps {
    pub vocabulary: HashMap<String, String>,
    pub seed: u64,
    pub on_completed: Callback<()>,
}

#[function_component(ResumableCharacterHunt)]
pub fn resumable_character_hunt(props: &ResumableCharacterHuntProps) -> Html {
    let dictionary = use_dictionary();
    let candidates = use_state(|| None::<Result<Vec<Candidate>, String>>);
    let configuration = HuntConfig::from_query(&props.vocabulary);
    let source = configuration.vocabulary;
    let difficulty = configuration.difficulty;

    {
        let dictionary = dictionary.clone();
        let candidates = candidates.clone();
        use_effect_with(source.clone(), move |source| {
            let active = Rc::new(Cell::new(true));
            let pending = active.clone();
            let source = source.clone();
            candidates.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let loaded = load_candidates(&dictionary, &source).await;
                if pending.get() {
                    candidates.set(Some(loaded));
                }
            });
            move || active.set(false)
        });
    }

    let content = match &*candidates {
        None => html! {
            <p role="status">{"Loading Character Hunt vocabulary…"}</p>
        },
        Some(Err(error)) => html! {
            <p role="alert">{format!("Could not load Character Hunt vocabulary: {error}")}</p>
        },
        Some(Ok(words)) if words.is_empty() => html! {
            <p role="alert">{"No eligible 2-3 character words were found. Skip this exercise or change the vocabulary source."}</p>
        },
        Some(Ok(words)) => html! {
            <HuntSession
                candidates={words.clone()}
                difficulty={difficulty}
                rounds={Some(1)}
                instance_seed={Some(props.seed)}
                on_completed={Some(props.on_completed.clone())}
            />
        },
    };

    html! {
        <div data-resumable-character-hunt="" class="w-full">
            <ExerciseSurface icon={InstructionIcon::Find} instruction="Find all hidden words in the character grid.">
            <p class="text-sm text-muted">{"This round is practice-only and does not award mastery evidence."}</p>
            {content}
            </ExerciseSurface>
        </div>
    }
}
