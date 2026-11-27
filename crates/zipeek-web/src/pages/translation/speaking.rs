use yew::prelude::*;
use zipeek_exercises::speaking::PREVIEW_TARGETS;

use crate::{
    components::SpeakingRecorder,
    pages::practice::{
        feedback::ContinueExercise, instruction::InstructionIcon, surface::ExerciseSurface,
    },
};

#[derive(Properties, PartialEq)]
pub struct SpeakingPracticeProps {
    pub seed: u64,
    pub on_completed: Option<Callback<()>>,
}

#[function_component(Speaking)]
pub fn speaking() -> Html {
    html! {
        <SpeakingPractice
            seed={0}
            on_completed={None}
        />
    }
}

#[function_component(SpeakingPractice)]
pub fn speaking_practice(props: &SpeakingPracticeProps) -> Html {
    let in_session = props.on_completed.is_some();
    let initial = props.seed as usize % PREVIEW_TARGETS.len();
    let selected = use_state(move || initial);
    let target = &PREVIEW_TARGETS[*selected];
    let instance = target
        .instance()
        .expect("validated built-in speaking target");
    let on_continue = {
        let completed = props.on_completed.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(completed) = &completed {
                completed.emit(());
            }
        })
    };

    let body = html! {
        <>
            <section
                data-component="speaking-target-selector"
                class="space-y-3"
                aria-label="Choose a pronunciation practice sentence"
            >
                <h2
                    class="font-semibold"
                >
                    {"Choose a sentence"}
                </h2>
                <div
                    class="flex flex-wrap gap-2"
                >
                    {for PREVIEW_TARGETS.iter().enumerate().map(|(index, candidate)| {
                        let selected = selected.clone();
                        let current = *selected == index;
                        let on_select = Callback::from(move |_: MouseEvent| selected.set(index));
                        let element_classes_1 = classes!(
                                        "rounded-lg",
                                        "border",
                                        "px-4",
                                        "py-2",
                                        "text-sm",
                                        "font-semibold",
                                        if current { "border-blue-500 bg-blue-500/10" } else { "border-line bg-surface" },
                                    );
                        html! {
                            <button
                                key={candidate.id}
                                type="button"
                                data-action="select-speaking-target"
                                aria-pressed={current.to_string()}
                                class={element_classes_1}
                                onclick={on_select}
                            >
                                {candidate.focus_word}
                            </button>
                        }
                    })}
                </div>
            </section>
            <section
                data-component="speaking-target"
                data-exercise-definition={instance.definition_id}
                data-exercise-instance={instance.id}
                class="rounded-2xl border border-line bg-surface p-5 space-y-2"
            >
                <p
                    class="text-xs font-semibold uppercase tracking-wider text-muted"
                >
                    {"Read this sentence aloud"}
                </p>
                <p
                    lang="zh-Hans"
                    class="text-3xl font-semibold"
                >
                    {target.sentence}
                </p>
                <p
                    class="text-muted"
                >
                    {target.pinyin}
                </p>
                <p
                    class="text-sm text-muted"
                >
                    {target.translation}
                </p>
                <p
                    class="text-sm font-semibold"
                >
                    {format!("Focus: {} · {}", target.focus_word, target.focus_reading)}
                </p>
            </section>
            <SpeakingRecorder
                key={target.id}
            />
            <p
                class="text-sm text-muted"
            >
                {"Changing the sentence discards the previous recording. This is local, unscored practice; it does not assess tones, pronunciation, or vocabulary proficiency."}
            </p>
            {
                if props.on_completed.is_some() {
                    html! {
                        <ContinueExercise
                            on_continue={on_continue}
                            label={Some(AttrValue::from("Continue without a score"))}
                            speaking=true
                        />
                    }
                } else {
                    Html::default()
                }
            }
        </>
    };
    html! {
        <section data-component="speaking-page" class="mx-auto w-full max-w-3xl">
            {if in_session {
                html! {
                    <ExerciseSurface icon={InstructionIcon::Speak} instruction="Read the sentence aloud, then listen to your recording.">
                        {body}
                    </ExerciseSurface>
                }
            } else {
                html! {
                    <div class="space-y-6 p-5">
                        <header class="space-y-2">
                            <h1 class="text-3xl font-bold">{"Pronunciation Practice"}</h1>
                            <p class="text-muted">{"Choose a Mandarin sentence, then record and replay your pronunciation in context."}</p>
                        </header>
                        {body}
                    </div>
                }
            }}
        </section>
    }
}
