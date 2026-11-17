use yew::prelude::*;
use zipeek_exercises::sentence_builder::{self, SentencePrompt};

#[derive(Properties, PartialEq)]
pub struct SentenceBuilderProps {
    pub seed: u64,
    pub on_completed: Callback<()>,
}

#[function_component(SentenceBuilder)]
pub fn sentence_builder(props: &SentenceBuilderProps) -> Html {
    let prompt: &'static SentencePrompt =
        &sentence_builder::PROMPTS[props.seed as usize % sentence_builder::PROMPTS.len()];
    let selected = use_state(Vec::<usize>::new);
    let checked = use_state(|| false);
    {
        let selected = selected.clone();
        let checked = checked.clone();
        use_effect_with(props.seed, move |_| {
            selected.set(Vec::new());
            checked.set(false);
        });
    }
    let order = sentence_builder::shuffled_tile_ids(prompt, props.seed);
    let correct = sentence_builder::grade(prompt, &selected);
    let add_tile = {
        let selected = selected.clone();
        let checked = checked.clone();
        Callback::from(move |id: usize| {
            if !*checked && id < prompt.tiles.len() && !selected.contains(&id) {
                let mut next = (*selected).clone();
                next.push(id);
                selected.set(next);
            }
        })
    };
    let undo = {
        let selected = selected.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*selected).clone();
            next.pop();
            selected.set(next);
        })
    };
    let clear = {
        let selected = selected.clone();
        Callback::from(move |_: MouseEvent| selected.set(Vec::new()))
    };
    let check = {
        let checked = checked.clone();
        let selected = selected.clone();
        Callback::from(move |_: MouseEvent| {
            if selected.len() == prompt.tiles.len() {
                checked.set(true);
            }
        })
    };
    let continue_session = {
        let completed = props.on_completed.clone();
        Callback::from(move |_: MouseEvent| completed.emit(()))
    };
    html! {
        <section class="space-y-5 rounded-2xl border border-line bg-surface p-6" data-practice-sentence-builder="">
            <h2 class="text-xl font-semibold">{"Sentence Builder"}</h2>
            <p class="text-sm text-muted">{"Arrange every Chinese phrase into a natural sentence. The supplied tiles do not count as Hanzi typing evidence."}</p>
            <p>{prompt.meaning}</p>
            <div class="flex min-h-14 flex-wrap items-center gap-2 rounded-lg border border-line p-3" aria-label="Constructed sentence" lang="zh-CN" data-sentence-builder-selection="">
                {for selected.iter().map(|&id| html! {
                    <span key={id.to_string()} class="rounded-md bg-raised px-3 py-2">{prompt.tiles[id]}</span>
                })}
                {if selected.is_empty() { html!{<span class="text-muted">{"Select the tiles below"}</span>} } else {Html::default()}}
            </div>
            <div class="flex flex-wrap gap-2" role="group" aria-label="Available Chinese phrase tiles" lang="zh-CN">
                {for order.iter().map(|&id| {
                    let on_click = {
                        let add_tile = add_tile.clone();
                        Callback::from(move |_: MouseEvent| add_tile.emit(id))
                    };
                    html! {
                        <button key={id.to_string()} type="button" onclick={on_click} disabled={*checked || selected.contains(&id)}
                            class="rounded-lg border border-line bg-raised px-4 py-3 text-ink disabled:opacity-40"
                            aria-label={format!("Add {}", prompt.tiles[id])}>
                            {prompt.tiles[id]}
                        </button>
                    }
                })}
            </div>
            {if !*checked { html! {
                <div class="flex flex-wrap gap-3">
                    <button type="button" onclick={undo} disabled={selected.is_empty()} class="rounded-lg border border-line px-4 py-2 disabled:opacity-40">{"Undo"}</button>
                    <button type="button" onclick={clear} disabled={selected.is_empty()} class="rounded-lg border border-line px-4 py-2 disabled:opacity-40">{"Clear"}</button>
                    <button type="button" data-sentence-builder-check="" onclick={check} disabled={selected.len() != prompt.tiles.len()} class="rounded-lg bg-brand-action px-4 py-2 text-white disabled:opacity-40">{"Check answer"}</button>
                </div>
            }} else { html! {
                <div aria-live="polite" data-practice-feedback="" class="space-y-3">
                    <p class="font-semibold">{if correct {"Correct"} else {"Not quite"}}</p>
                    <p lang="zh-CN">{sentence_builder::sentence(prompt, prompt.accepted_orders[0]).unwrap_or_default()}</p>
                    <p class="text-sm text-muted">{prompt.explanation}</p>
                    <button type="button" data-sentence-builder-continue="" onclick={continue_session} class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white">{"Continue session"}</button>
                </div>
            }}}
        </section>
    }
}
