use std::collections::HashMap;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{route::Route, services::ai};

#[function_component(SentencesPage)]
pub fn sentences_page() -> Html {
    let location = use_location();
    let params = location
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .unwrap_or_default();
    let hanzi = params.get("word").cloned().unwrap_or_default();
    let pinyin = params.get("pinyin").cloned().unwrap_or_default();
    let meaning = params.get("meaning").cloned().unwrap_or_default();
    let valid_word = !hanzi.is_empty() && hanzi.chars().count() <= 32;
    let configured = !ai::configuration().model.trim().is_empty();
    let busy = use_state(|| false);
    let result = use_state(|| None::<Result<String, String>>);
    let request_generation = use_mut_ref(|| 0_u64);
    let on_generate = {
        let busy = busy.clone();
        let result = result.clone();
        let request_generation = request_generation.clone();
        let hanzi = hanzi.clone();
        let pinyin = pinyin.clone();
        let meaning = meaning.clone();
        Callback::from(move |_: MouseEvent| {
            if *busy || hanzi.is_empty() {
                return;
            }
            let request_id = {
                let mut generation = request_generation.borrow_mut();
                *generation += 1;
                *generation
            };
            busy.set(true);
            result.set(None);
            let result = result.clone();
            let busy = busy.clone();
            let generation = request_generation.clone();
            let hanzi = hanzi.clone();
            let pinyin = pinyin.clone();
            let meaning = meaning.clone();
            spawn_local(async move {
                let response = ai::example_sentence(&hanzi, &pinyin, &meaning).await;
                if *generation.borrow() == request_id {
                    result.set(Some(response));
                    busy.set(false);
                }
            });
        })
    };

    html! {
        <main data-component="sentences-page" class="mx-auto flex w-full max-w-3xl flex-col gap-5 py-5 sm:py-8">
            <header class="space-y-2">
                <Link<Route> to={Route::Home} classes="text-sm font-semibold text-accent no-underline hover:underline">
                    {"← Word of the Day"}
                </Link<Route>>
                <h1 class="text-2xl font-bold text-ink">{"Sentences"}</h1>
                <p class="text-sm text-muted">{"Generate a Mandarin example using the selected dictionary meaning."}</p>
            </header>
            <section class="space-y-4 rounded-2xl border border-line bg-surface p-5 sm:p-7">
                if valid_word {
                    <div class="flex flex-wrap items-center gap-3 border-b border-line pb-4">
                        <strong lang="zh-Hans" class="text-4xl text-ink">{&hanzi}</strong>
                        <span class="font-medium text-accent">{&pinyin}</span>
                        <p class="w-full text-sm text-muted">{&meaning}</p>
                    </div>
                } else {
                    <p role="alert" class="text-sm text-muted">{"Select a word from Home to create an example."}</p>
                }
                if !configured {
                    <div data-sentences-ai-unavailable="" role="status" class="rounded-xl border border-amber-500/40 bg-amber-500/10 p-4 text-sm text-ink">
                        <p class="font-semibold">{"AI model not configured"}</p>
                        <p class="mt-1 text-muted">{"Choose a model in Settings before generating a sentence."}</p>
                        <a class="mt-3 inline-flex font-semibold text-accent underline-offset-4 hover:underline" href="#/settings?section=ai">{"Open AI settings"}</a>
                    </div>
                }
                <button
                    type="button"
                    data-generate-sentence=""
                    onclick={on_generate}
                    disabled={!valid_word || !configured || *busy}
                    class="inline-flex min-h-10 items-center justify-center rounded-lg bg-brand-action px-5 text-sm font-semibold text-white hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-50"
                >
                    {if *busy { "Generating…" } else { "Generate sentence" }}
                </button>
                if *busy {
                    <p role="status" class="text-sm text-muted">{"Waiting for your AI model…"}</p>
                }
                if let Some(response) = &*result {
                    <div data-generated-sentence="" class="space-y-2 rounded-xl border border-line bg-raised p-4 text-sm leading-relaxed">
                        <p class="text-xs font-semibold text-muted">{"AI-generated · not a verified dictionary citation"}</p>
                        {match response {
                            Ok(sentence) => html! { <p class="whitespace-pre-line text-ink">{sentence}</p> },
                            Err(error) => html! { <div role="alert" class="space-y-2"><p class="font-semibold text-ink">{"Generation unsuccessful"}</p><p class="text-muted">{error}</p><a href="#/settings?section=ai" class="text-accent hover:underline">{"Check AI settings"}</a></div> },
                        }}
                    </div>
                }
            </section>
        </main>
    }
}
