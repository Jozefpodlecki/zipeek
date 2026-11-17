use yew::prelude::*;
use zipeek_exercises::{audio_dictation, fill_blank};

use crate::{hooks::use_app_environment, services::practice_audio};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoredExerciseMode {
    FillBlank,
    AudioDictation,
}

#[derive(Properties, PartialEq)]
pub struct AuthoredExerciseProps {
    pub mode: AuthoredExerciseMode,
    pub seed: u64,
    pub on_completed: Callback<()>,
}

#[function_component(AuthoredExercise)]
pub fn authored_exercise(props: &AuthoredExerciseProps) -> Html {
    let environment = use_app_environment();
    let input = use_state(String::new);
    let checked = use_state(|| false);
    let composing = use_state(|| false);
    let played = use_state(|| false);
    let audio_error = use_state(String::new);
    {
        let input = input.clone();
        let checked = checked.clone();
        let composing = composing.clone();
        let played = played.clone();
        let audio_error = audio_error.clone();
        use_effect_with((props.mode, props.seed), move |_| {
            input.set(String::new());
            checked.set(false);
            composing.set(false);
            played.set(false);
            audio_error.set(String::new());
        });
    }
    let cloze = &fill_blank::PROMPTS[props.seed as usize % fill_blank::PROMPTS.len()];
    let dictation = &audio_dictation::PROMPTS[props.seed as usize % audio_dictation::PROMPTS.len()];
    let is_dictation = props.mode == AuthoredExerciseMode::AudioDictation;
    let is_correct = if is_dictation {
        audio_dictation::grade(dictation, &input)
    } else {
        fill_blank::grade(cloze, &input)
    };
    let on_input = {
        let input = input.clone();
        let composing = composing.clone();
        Callback::from(move |event: InputEvent| {
            composing.set(event.is_composing());
            let field: web_sys::HtmlInputElement = event.target_unchecked_into();
            input.set(field.value());
        })
    };
    let on_keydown = Callback::from(move |event: KeyboardEvent| {
        if event.key() == "Enter" && (event.is_composing() || event.key_code() == 229) {
            event.prevent_default();
        }
    });
    let on_check = {
        let checked = checked.clone();
        let composing = composing.clone();
        let input = input.clone();
        let played = played.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if !*checked && !*composing && !input.trim().is_empty() && (!is_dictation || *played) {
                checked.set(true);
            }
        })
    };
    let on_play = {
        let speech = environment.speech.clone();
        let audio_error = audio_error.clone();
        let played = played.clone();
        let transcript = dictation.transcript.to_owned();
        Callback::from(
            move |_: MouseEvent| match practice_audio::speak(&speech, &transcript) {
                Ok(()) => {
                    played.set(true);
                    audio_error.set(String::new());
                }
                Err(error) => {
                    audio_error.set(format!("Mandarin audio unavailable: {error:?}"));
                }
            },
        )
    };
    let on_continue = {
        let completed = props.on_completed.clone();
        Callback::from(move |_: MouseEvent| completed.emit(()))
    };
    html! {
        <section
            class="space-y-5 rounded-2xl border border-line bg-surface p-6"
            data-practice-authored-exercise=""
        >
            <h2 class="text-xl font-semibold">
                {if is_dictation { "Audio Dictation" } else { "Fill in the Blank" }}
            </h2>
            <p class="text-sm text-muted">
                {"Author-provided practice examples. Feedback here does not award learning mastery."}
            </p>
            {
                if is_dictation {
                    html! {
                        <div class="space-y-3">
                            <p>{"Listen and type the Mandarin sentence in Chinese characters. No transcript is shown until you check."}</p>
                            <button
                                type="button"
                                class="rounded-lg bg-brand-action px-4 py-3 font-semibold text-white"
                                onclick={on_play}
                            >
                                {"Play Mandarin audio"}
                            </button>
                            {if !audio_error.is_empty() {
                                html! { <p role="alert" class="text-sm text-red-600">{(*audio_error).clone()}</p> }
                            } else { Html::default() }}
                        </div>
                    }
                } else {
                    html! {
                        <div class="space-y-3">
                            <p class="text-muted">{cloze.meaning_cue}</p>
                            <p lang="zh-CN" class="text-2xl font-semibold">
                                {cloze.sentence_before}{" ____ "}{cloze.sentence_after}
                            </p>
                        </div>
                    }
                }
            }
            <form onsubmit={on_check} class="space-y-3">
                <label class="block font-medium" for="authored-hanzi-response">
                    {if is_dictation { "Your Hanzi transcription" } else { "Missing Chinese word" }}
                </label>
                <input
                    id="authored-hanzi-response"
                    type="text"
                    lang="zh-CN"
                    autocomplete="off"
                    class="w-full rounded-lg border border-line bg-raised px-4 py-3 text-ink"
                    value={(*input).clone()}
                    oninput={on_input}
                    onkeydown={on_keydown}
                    disabled={*checked}
                />
                {if !*checked {
                    html! {
                        <button
                            type="submit"
                            disabled={input.trim().is_empty() || *composing || (is_dictation && !*played)}
                            class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white disabled:opacity-50"
                        >
                            {"Check answer"}
                        </button>
                    }
                } else { Html::default() }}
            </form>
            {if *checked {
                html! {
                    <div class="space-y-3" aria-live="polite" data-practice-feedback="">
                        <p class="font-semibold">
                            {if is_correct { "Correct" } else { "Not quite" }}
                        </p>
                        <p lang="zh-CN">
                            {if is_dictation { dictation.transcript.to_owned() } else {
                                format!("{}{}{}", cloze.sentence_before, cloze.accepted_hanzi[0], cloze.sentence_after)
                            }}
                        </p>
                        <p class="text-sm text-muted">
                            {if is_dictation { dictation.meaning } else { cloze.explanation }}
                        </p>
                        <button
                            type="button"
                            class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white"
                            onclick={on_continue}
                        >
                            {"Continue session"}
                        </button>
                    </div>
                }
            } else { Html::default() }}
        </section>
    }
}
