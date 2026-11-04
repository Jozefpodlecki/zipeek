use yew::prelude::*;
use zipeek_exercises::speaking::PREVIEW_TARGETS;

use crate::components::SpeakingRecorder;

#[function_component(Speaking)]
pub fn speaking() -> Html {
    let selected = use_state(|| 0usize);
    let target = &PREVIEW_TARGETS[*selected];
    let instance = target.instance().expect("validated built-in speaking target");

    html! {
        <main data-component="speaking-page" class="mx-auto flex w-full max-w-3xl flex-col gap-6 p-5">
            <header class="space-y-2">
                <h1 class="text-3xl font-bold">{"Speaking practice"}</h1>
                <p class="text-muted">{"Choose a sentence, then record and listen to the intended pronunciation in context."}</p>
            </header>
            <section data-component="speaking-target-selector" class="space-y-3" aria-label="Choose a speaking sentence">
                <h2 class="font-semibold">{"Choose a sentence"}</h2>
                <div class="flex flex-wrap gap-2">
                    {for PREVIEW_TARGETS.iter().enumerate().map(|(index, candidate)| {
                        let selected = selected.clone();
                        let current = *selected == index;
                        let on_select = Callback::from(move |_: MouseEvent| selected.set(index));
                        html! {
                            <button
                                key={candidate.id}
                                type="button"
                                data-action="select-speaking-target"
                                aria-pressed={current.to_string()}
                                class={classes!("rounded-lg", "border", "px-4", "py-2", "text-sm", "font-semibold", if current { "border-blue-500 bg-blue-500/10" } else { "border-line bg-surface" })}
                                onclick={on_select}
                            >
                                {candidate.focus_word}
                            </button>
                        }
                    })}
                </div>
            </section>
            <section data-component="speaking-target" data-exercise-definition={instance.definition_id} data-exercise-instance={instance.id} class="rounded-2xl border border-line bg-surface p-5 space-y-2">
                <p class="text-xs font-semibold uppercase tracking-wider text-muted">{"Read this sentence aloud"}</p>
                <p lang="zh-Hans" class="text-3xl font-semibold">{target.sentence}</p>
                <p class="text-muted">{target.pinyin}</p>
                <p class="text-sm text-muted">{target.translation}</p>
                <p class="text-sm font-semibold">{format!("Focus: {} · {}", target.focus_word, target.focus_reading)}</p>
            </section>
            <SpeakingRecorder key={target.id} />
            <p class="text-sm text-muted">{"Changing the sentence discards the previous recording. This is a local, unscored practice preview; it does not assess tones, pronunciation, or vocabulary proficiency."}</p>
        </main>
    }
}
