use yew::prelude::*;
use zipeek_exercises::province_quest::{Difficulty, LearningSupport};

#[derive(Properties, PartialEq)]
pub struct ProvinceSetupProps {
    pub difficulty: Difficulty,
    pub support: LearningSupport,
    pub on_difficulty: Callback<Difficulty>,
    pub on_support: Callback<LearningSupport>,
    pub on_start: Callback<MouseEvent>,
    pub error: Option<String>,
    pub loading: bool,
}

#[function_component(ProvinceSetup)]
pub fn province_setup(props: &ProvinceSetupProps) -> Html {
    let difficulty_buttons = [
        (Difficulty::Easy, "Easy", "Identify provincial-level regions by Hanzi"),
        (Difficulty::Medium, "Medium", "Regions, capitals and outlines"),
        (Difficulty::Hard, "Hard", "Find prefecture-level divisions within provinces"),
    ].into_iter().map(|(difficulty,title,description)| {
        let selected = props.difficulty == difficulty;
        let onclick = props.on_difficulty.reform(move |_: MouseEvent| difficulty);
        let classes = format!("rounded-xl border p-4 text-left transition-colors {}",if selected {"border-accent bg-raised"} else {"border-line hover:border-accent"});
        html! {<button type="button" key={title} class={classes} aria-pressed={selected.to_string()} onclick={onclick}>
            <strong class="block">{title}</strong><span class="text-sm text-muted">{description}</span>
        </button>}
    }).collect::<Html>();
    let support_buttons = [
        (LearningSupport::Guided,"Guided","Hanzi, Pinyin and English"),
        (LearningSupport::Reading,"Reading","Hanzi; reveal pronunciation"),
        (LearningSupport::Challenge,"Challenge","Hanzi only until review"),
    ].into_iter().map(|(support,title,description)| {
        let selected = props.support == support;
        let onclick = props.on_support.reform(move |_: MouseEvent| support);
        let classes = format!("rounded-xl border p-4 text-left transition-colors {}",if selected {"border-accent bg-raised"} else {"border-line hover:border-accent"});
        html! {<button type="button" key={title} class={classes} aria-pressed={selected.to_string()} onclick={onclick}>
            <strong class="block">{title}</strong><span class="text-sm text-muted">{description}</span>
        </button>}
    }).collect::<Html>();
    html! {
        <section class="space-y-6 rounded-2xl border border-line bg-surface p-5 sm:p-8" data-game-setup="China Map Challenge">
            <header class="space-y-2">
                <p class="text-sm font-semibold text-accent">{"Learn through play"}</p>
                <h1 class="text-3xl font-bold">{"China Map Challenge"}</h1>
                <p class="text-muted">{"Find China's administrative regions on an interactive map."}</p>
            </header>
            <fieldset class="space-y-3">
                <legend class="text-lg font-semibold">{"Map difficulty"}</legend>
                <div class="grid gap-3 sm:grid-cols-3">{difficulty_buttons}</div>
            </fieldset>
            <fieldset class="space-y-3">
                <legend class="text-lg font-semibold">{"Learning support"}</legend>
                <div class="grid gap-3 sm:grid-cols-3">{support_buttons}</div>
            </fieldset>
            if let Some(error) = &props.error {
                <p role="alert" class="rounded-lg border border-red-600 p-3 text-sm text-red-400">{error}</p>
            }
            <div class="flex items-center justify-between gap-4 rounded-xl bg-raised p-4">
                <span class="text-sm text-muted">{"10 randomized locations per game"}</span>
                <button type="button" class="rounded-xl bg-brand-action px-5 py-3 font-semibold text-white disabled:opacity-50" disabled={props.loading} onclick={props.on_start.clone()}> {if props.loading { "Loading map…" } else { "Start game" }} </button>
            </div>
        </section>
    }
}
