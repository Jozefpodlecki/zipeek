use yew::prelude::*;
use zipeek_exercises::spot_mistake::{self, MistakePrompt};

#[derive(Properties, PartialEq)]
pub struct SpotMistakeProps {
    pub seed: u64,
    pub on_completed: Callback<()>,
}

#[function_component(SpotMistake)]
pub fn spot_mistake(props: &SpotMistakeProps) -> Html {
    let prompt: &'static MistakePrompt =
        &spot_mistake::PROMPTS[props.seed as usize % spot_mistake::PROMPTS.len()];
    let selected = use_state(|| None::<usize>);
    let replacement = use_state(String::new);
    let checked = use_state(|| false);
    let composing = use_state(|| false);
    {
        let selected = selected.clone();
        let replacement = replacement.clone();
        let checked = checked.clone();
        let composing = composing.clone();
        use_effect_with(props.seed, move |_| {
            selected.set(None);
            replacement.set(String::new());
            checked.set(false);
            composing.set(false);
        });
    }
    let verdict = spot_mistake::grade(prompt, *selected, &replacement);
    let on_input = {
        let replacement = replacement.clone();
        let composing = composing.clone();
        Callback::from(move |event: InputEvent| {
            composing.set(event.is_composing());
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            replacement.set(input.value());
        })
    };
    let on_keydown = Callback::from(move |event: KeyboardEvent| {
        if event.key() == "Enter" && (event.is_composing() || event.key_code() == 229) {
            event.prevent_default();
        }
    });
    let on_check = {
        let selected = selected.clone();
        let replacement = replacement.clone();
        let checked = checked.clone();
        let composing = composing.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if selected.is_some() && !replacement.trim().is_empty() && !*composing && !*checked {
                checked.set(true);
            }
        })
    };
    let on_continue = {
        let on_completed = props.on_completed.clone();
        Callback::from(move |_: MouseEvent| on_completed.emit(()))
    };
    html! {
        <section class="space-y-5 rounded-2xl border border-line bg-surface p-6" data-practice-spot-mistake="">
            <h2 class="text-xl font-semibold">{"Spot the Mistake"}</h2>
            <p class="text-sm text-muted">{"Authored grammar practice only. No mastery evidence is awarded."}</p>
            <p>{prompt.meaning}</p>
            <p>{"Select the incorrect Chinese phrase, then type a replacement."}</p>
            <div class="flex flex-wrap gap-2" role="group" aria-label="Select the incorrect Chinese phrase" lang="zh-CN">
                {for prompt.spans.iter().enumerate().map(|(index, span)| {
                    let selected_value = selected.clone();
                    let choose = Callback::from(move |_: MouseEvent| selected_value.set(Some(index)));
                    html! {
                        <button key={index.to_string()} type="button" onclick={choose}
                            disabled={*checked} aria-pressed={(*selected == Some(index)).to_string()}
                            class="rounded-lg border border-line bg-raised px-3 py-3 text-ink disabled:opacity-60">
                            {span.to_string()}
                        </button>
                    }
                })}
            </div>
            <form onsubmit={on_check} class="space-y-3">
                <label for="spot-mistake-repair" class="block font-medium">{"Replacement Chinese phrase"}</label>
                <input id="spot-mistake-repair" type="text" lang="zh-CN" autocomplete="off"
                    class="w-full rounded-lg border border-line bg-raised px-4 py-3 text-ink"
                    value={(*replacement).clone()} oninput={on_input} onkeydown={on_keydown}
                    disabled={*checked} />
                {if !*checked { html! {
                    <button type="submit" data-spot-mistake-check=""
                        disabled={selected.is_none() || replacement.trim().is_empty() || *composing}
                        class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white disabled:opacity-50">
                        {"Check answer"}
                    </button>
                }} else { Html::default() }}
            </form>
            {if *checked { html! {
                <div aria-live="polite" data-practice-feedback="" class="space-y-3">
                    <p>{if verdict.identified { "Mistake identified correctly" } else { "Mistake not identified" }}</p>
                    <p>{if verdict.repaired { "Correction accepted" } else { "Correction not accepted" }}</p>
                    <p lang="zh-CN">{spot_mistake::corrected_sentence(prompt, prompt.accepted_replacements[0]).unwrap_or_default()}</p>
                    <p class="text-sm text-muted">{prompt.explanation}</p>
                    <button type="button" data-spot-mistake-continue="" onclick={on_continue}
                        class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white">{"Continue session"}</button>
                </div>
            }} else { Html::default() }}
        </section>
    }
}
