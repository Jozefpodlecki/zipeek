use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::hooks::use_dictionary;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::use_location;
use zipeek_exercises::character_hunt::Candidate;

use super::{config::HuntConfig, session::HuntSession, vocabulary};

#[function_component(CharacterHuntPage)]
pub fn character_hunt_page() -> Html {
    let dictionary = use_dictionary();
    let location = use_location();
    let params = location
        .as_ref()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .unwrap_or_default();
    let in_playlist = params.contains_key("playlist");
    let config = HuntConfig::from_query(&params);
    let pool = use_state(|| None::<Result<Vec<Candidate>, String>>);

    {
        let dictionary = dictionary.clone();
        let pool = pool.clone();
        let vocabulary = config.vocabulary.clone();
        use_effect_with(vocabulary, move |vocabulary| {
            let live = Rc::new(Cell::new(true));
            let pending = live.clone();
            let vocabulary = vocabulary.clone();
            pool.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let result = vocabulary::load_candidates(&dictionary, &vocabulary).await;
                if pending.get() {
                    pool.set(Some(result));
                }
            });
            move || live.set(false)
        });
    }

    let candidate_loading_content = {
        match &*pool {
            Some(Ok(candidates)) if candidates.is_empty() => html! {
                <p role="alert" class="rounded-xl border border-line bg-surface p-5">
                    {"No eligible Character Hunt words were found. Choose a larger vocabulary source."}
                </p>
            },
            Some(Ok(candidates)) => html! {
                <HuntSession
                    candidates={candidates.clone()}
                    difficulty={config.difficulty}
                    rounds={config.rounds}
                />
            },
            Some(Err(error)) => html! {
                <p
                    role="alert"
                    class="rounded-xl border border-line bg-surface p-5"
                >
                    {format!("Could not load vocabulary: {error}")}
                </p>
            },
            None => html! {
                <p
                    role="status"
                >
                    {"Loading eligible vocabulary…"}
                </p>
            },
        }
    };
    html! {
        <main
            data-component="character-hunt"
            class="mx-auto flex w-full max-w-6xl min-h-0 flex-col gap-3 py-3"
        >
            <header>
                <h1
                    class="flex items-center gap-2 text-xl font-bold"
                >
                    <Icon
                        data={IconData::LUCIDE_GRID}
                        width="22px"
                        height="22px"
                    />
                    {"Character Hunt"}
                </h1>
                <p
                    class="text-sm text-muted"
                >
                    {if in_playlist {
                        "Find words to finish this exercise and continue to the next."
                    } else {
                        "Select two or three characters to find a word."
                    }}
                </p>
            </header>
            {candidate_loading_content}
        </main>
    }
}
