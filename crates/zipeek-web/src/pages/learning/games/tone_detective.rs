use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::tone_detective::ToneGame;

use crate::hooks::use_app_environment;

#[function_component(ToneDetectivePage)]
pub fn tone_detective_page() -> Html {
    let environment = use_app_environment();
    let game = use_state(|| None::<ToneGame>);
    let audio_error = use_state(|| None::<String>);

    let on_new_game = {
        let game = game.clone();
        let audio_error = audio_error.clone();
        Callback::from(move |_: MouseEvent| {
            game.set(Some(ToneGame::new(
                chrono::Utc::now().timestamp_millis() as u64
            )));
            audio_error.set(None);
        })
    };
    let on_exit = {
        let game = game.clone();
        Callback::from(move |_: MouseEvent| game.set(None))
    };
    let on_next = {
        let game = game.clone();
        let audio_error = audio_error.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(mut round) = (*game).clone() {
                if round.next() {
                    game.set(Some(round));
                    audio_error.set(None);
                }
            }
        })
    };
    let on_listen = {
        let speech = environment.speech.clone();
        let audio_error = audio_error.clone();
        let hanzi = (*game)
            .as_ref()
            .and_then(ToneGame::prompt)
            .map(|prompt| prompt.hanzi.to_owned());
        Callback::from(move |_: MouseEvent| {
            if let Some(hanzi) = &hanzi {
                match crate::services::practice_audio::speak(&speech, hanzi) {
                    Ok(()) => audio_error.set(None),
                    Err(_) => audio_error.set(Some(
                        "Speech playback is unavailable in this browser.".to_owned(),
                    )),
                }
            }
        })
    };

    let content = match &*game {
        None => html! {
            <section class="mx-auto flex max-w-xl flex-col items-center gap-5 rounded-2xl border border-line bg-surface p-8 text-center">
                <Icon
                    data={IconData::LUCIDE_EAR}
                    width="48px"
                    height="48px"
                />
                <h2 class="text-2xl font-bold">
                    {"Hear the tone. Find the answer."}
                </h2>
                <p class="text-muted">
                    {"Listen to a single Mandarin syllable and identify its tone. Each round has 12 questions covering all four tones. Speech playback uses your browser's Mandarin voice."}
                </p>
                <button
                    type="button"
                    data-tone-start=""
                    onclick={on_new_game.clone()}
                    class="rounded-xl bg-brand-action px-6 py-3 font-semibold text-white"
                >
                    {"Start game"}
                </button>
            </section>
        },
        Some(round) if round.finished() => html! {
            <section
                class="mx-auto flex max-w-xl flex-col items-center gap-5 rounded-2xl border border-line bg-surface p-8 text-center"
                data-tone-finished=""
            >
                <Icon
                    data={IconData::LUCIDE_TROPHY}
                    width="48px"
                    height="48px"
                />
                <h2 class="text-2xl font-bold">
                    {"Investigation complete"}
                </h2>
                <p
                    class="text-3xl font-bold"
                    data-tone-score=""
                >
                    {format!("{} / {}", round.correct(), round.total())}
                </p>
                <p class="text-muted">
                    {"Your answers are for practice only and do not change learning proficiency."}
                </p>
                {if round.misses().count() > 0 {
                    html! {
                        <section
                            class="w-full text-left"
                            data-tone-missed=""
                        >
                            <h3 class="mb-3 text-lg font-semibold">
                                {"Tones to review"}
                            </h3>
                            <ul class="grid gap-2 sm:grid-cols-2">
                                {for round.misses().map(|(prompt, selected)| html! {
                                    <li
                                        key={prompt.hanzi}
                                        class="rounded-xl border border-line bg-raised p-3"
                                    >
                                        <span
                                            lang="zh-Hans"
                                            class="text-xl font-semibold"
                                        >
                                            {prompt.hanzi}
                                        </span>
                                        <span class="ml-2 text-sm">
                                            {prompt.pinyin}
                                        </span>
                                        <p class="text-sm text-muted">
                                            {format!("You chose tone {selected}. Correct: tone {}.", prompt.tone)}
                                        </p>
                                    </li>
                                })}
                            </ul>
                        </section>
                    }
                } else {
                    Html::default()
                }}
                <div class="flex flex-wrap justify-center gap-3">
                    <button
                        type="button"
                        onclick={on_new_game.clone()}
                        data-tone-replay=""
                        class="rounded-xl bg-brand-action px-5 py-3 font-semibold text-white"
                    >
                        {"Play again"}
                    </button>
                    <button
                        type="button"
                        onclick={on_exit}
                        class="rounded-xl border border-line px-5 py-3 font-semibold"
                    >
                        {"Back to setup"}
                    </button>
                </div>
            </section>
        },
        Some(round) => {
            let prompt = round.prompt().expect("unfinished round has a prompt");
            let answer = round.answer();
            let correct = answer == Some(prompt.tone);
            let buttons = (1..=4u8).map(|tone| {
                let game = game.clone();
                let on_answer = Callback::from(move |_: MouseEvent| {
                    if let Some(mut next) = (*game).clone() {
                        if next.choose(tone) {
                            game.set(Some(next));
                        }
                    }
                });
                let label = match tone {
                    1 => "1 - High level",
                    2 => "2 - Rising",
                    3 => "3 - Low dipping",
                    _ => "4 - Falling",
                };
                let element_classes_1 = classes!(
                    "min-h-16",
                    "rounded-xl",
                    "border",
                    "px-4",
                    "py-3",
                    "font-semibold",
                    "transition-colors",
                    "focus-visible:outline-2",
                    "focus-visible:outline-offset-2",
                    if answer.is_some() && tone == prompt.tone {
                        "border-green-600 bg-green-600/15"
                    } else if answer == Some(tone) {
                        "border-red-500 bg-red-500/15"
                    } else {
                        "border-line bg-raised hover:border-accent"
                    },
                );
                html! {
                    <button
                        key={tone}
                        type="button"
                        data-tone-answer={tone.to_string()}
                        disabled={answer.is_some()}
                        aria-pressed={(answer == Some(tone)).to_string()}
                        onclick={on_answer}
                        class={element_classes_1}
                    >
                        {label}
                    </button>
                }
            });
            html! {
                <section
                    class="mx-auto flex w-full max-w-2xl flex-col gap-5 rounded-2xl border border-line bg-surface p-6 sm:p-8"
                    data-tone-question=""
                >
                    <div class="flex justify-between gap-3 text-sm text-muted">
                        <span>
                            {format!("Question {} of {}", round.completed() + usize::from(answer.is_none()), round.total())}
                        </span>
                        <span>
                            {format!("{} correct", round.correct())}
                        </span>
                    </div>
                    <div class="flex flex-col items-center gap-3 rounded-xl bg-raised p-6 text-center">
                        <p class="text-sm text-muted">
                            {"Listen, then identify the tone"}
                        </p>
                        <p
                            class="text-6xl font-bold"
                            lang="zh-Hans"
                        >
                            {prompt.hanzi}
                        </p>
                        <p class="text-sm text-muted">
                            {prompt.meaning}
                        </p>
                        <button
                            type="button"
                            data-tone-listen=""
                            onclick={on_listen}
                            class="inline-flex min-h-11 items-center gap-2 rounded-xl border border-accent px-5 py-2 font-semibold"
                        >
                            <Icon
                                data={IconData::LUCIDE_VOLUME_2}
                                width="20px"
                                height="20px"
                            />
                            {"Play pronunciation"}
                        </button>
                        {if let Some(error) = &*audio_error {
                            html! {
                                <p
                                    role="alert"
                                    class="text-sm text-red-400"
                                >
                                    {error}
                                </p>
                            }
                        } else {
                            Html::default()
                        }}
                    </div>
                    <div
                        role="group"
                        aria-label="Choose Mandarin tone"
                        class="grid grid-cols-2 gap-3"
                    >
                        {for buttons}
                    </div>
                    {if answer.is_some() {
                        html! {
                            <div
                                class="flex flex-col gap-3 rounded-xl border border-line bg-raised p-4"
                                data-tone-feedback=""
                                role="status"
                            >
                                <p class="font-semibold">
                                    {if correct { "Correct tone" } else { "Not quite" }}
                                </p>
                                <p>
                                    {format!("{} ({}) - tone {}", prompt.hanzi, prompt.pinyin, prompt.tone)}
                                </p>
                                <button
                                    type="button"
                                    data-tone-next=""
                                    onclick={on_next}
                                    class="self-start rounded-xl bg-brand-action px-5 py-3 font-semibold text-white"
                                >
                                    {if round.completed() == round.total() {
                                        "See results"
                                    } else {
                                        "Next question"
                                    }}
                                </button>
                            </div>
                        }
                    } else {
                        Html::default()
                    }}
                </section>
            }
        }
    };

    html! {
        <main
            data-component="tone-detective"
            class="mx-auto flex w-full max-w-4xl flex-col gap-6 py-5"
        >
            <header class="space-y-2 text-center">
                <p class="text-sm font-semibold text-accent">
                    {"Mandarin listening game"}
                </p>
                <h1 class="text-3xl font-bold">
                    {"Tone Detective"}
                </h1>
            </header>
            {content}
        </main>
    }
}
