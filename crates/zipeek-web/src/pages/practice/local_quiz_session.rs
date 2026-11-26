use std::{
    cell::{Cell, RefCell},
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use gloo::timers::callback::Timeout;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_core::practice::ExerciseVariant;
use zipeek_browser::local_speech::{self, MatchingAudio};
use zipeek_exercises::local_quiz::{
    PairAttempt, QuizRound, QuizWord, prepare_for_format_with_count,
};

use crate::hooks::use_dictionary;

use super::vocabulary_exercises::load_words;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LocalQuizMode {
    Matching,
    MultipleChoice,
}

#[derive(Properties, PartialEq)]
pub struct LocalQuizSessionProps {
    pub mode: LocalQuizMode,
    pub vocabulary: HashMap<String, String>,
    pub seed: u64,
    pub quiz_format: String,
    pub reverse_matching: bool,
    pub pair_count: u8,
    pub generated_words: Option<Vec<QuizWord>>,
    pub show_pinyin: bool,
    pub on_completed: Callback<()>,
}

fn selected_quiz_format(token: &str, mode: LocalQuizMode) -> Result<ExerciseVariant, String> {
    let format =
        ExerciseVariant::from_seed_token(token).ok_or_else(|| "Unknown quiz format".to_owned())?;
    if format.is_mcq() != (mode == LocalQuizMode::MultipleChoice) {
        return Err("The saved format is incompatible with this quiz".to_owned());
    }
    Ok(format)
}

#[function_component(LocalQuizSession)]
pub fn local_quiz_session(props: &LocalQuizSessionProps) -> Html {
    let dictionary = use_dictionary();
    let round = use_state(|| None::<Result<QuizRound, String>>);
    let selected = use_state(|| None::<i32>);
    let matched = use_state(BTreeSet::<i32>::new);
    let wrong_attempts = use_state(|| 0u32);
    let pairing_error = use_state(|| None::<String>);
    let last_incorrect = use_state(|| None::<(i32, i32)>);
    let prepared_audio = use_state(|| None::<Result<Vec<MatchingAudio>, String>>);
    let audio_progress = use_state(|| 0usize);
    let audio_failed_word = use_state(|| None::<(usize, String)>);
    let audio_retry = use_state(|| 0u32);

    {
        let dictionary = dictionary.clone();
        let round = round.clone();
        let selected = selected.clone();
        let matched = matched.clone();
        let wrong_attempts = wrong_attempts.clone();
        let pairing_error = pairing_error.clone();
        let last_incorrect = last_incorrect.clone();
        use_effect_with(
            (
                props.vocabulary.clone(),
                props.seed,
                props.mode,
                props.quiz_format.clone(),
                props.pair_count,
                props.generated_words.clone(),
            ),
            move |(vocabulary, seed, mode, token, pair_count, generated_words)| {
                let alive = Rc::new(Cell::new(true));
                let pending = alive.clone();
                let vocabulary = vocabulary.clone();
                let generated_words = generated_words.clone();
                let seed = *seed;
                let pair_count = usize::from(*pair_count);
                let format = selected_quiz_format(token, *mode);
                round.set(None);
                selected.set(None);
                matched.set(BTreeSet::new());
                wrong_attempts.set(0);
                pairing_error.set(None);
                last_incorrect.set(None);
                spawn_local(async move {
                    let result = if let Some(words) = generated_words {
                    format.and_then(|variant| prepare_for_format_with_count(&words, seed, variant, pair_count)
                        .map_err(|error| format!("Cannot prepare generated matching pairs: {error:?}")))
                } else {
                    load_words(&dictionary, &vocabulary, true)
                    .await
                    .and_then(|words| {
                        let format = format?;
                        let candidates = words.iter().filter_map(|word| {
                            let meaning = word.english.as_deref()?.split('|').next()?.trim();
                            if meaning.is_empty() { return None; }
                            Some(QuizWord {
                                id: word.id.as_i32(),
                                hanzi: word.simplified.clone(),
                                meaning: meaning.to_owned(),
                                pinyin: word.pinyin.clone(),
                            })
                        }).collect::<Vec<_>>();
                        prepare_for_format_with_count(&candidates, seed, format, pair_count).map_err(|error| {
                            format!("Cannot prepare matching vocabulary: {error:?}. Choose enough distinct words with distinct meanings for the configured number of pairs.")
                        })
                    })
                };
                    if pending.get() {
                        round.set(Some(result));
                    }
                });
                move || alive.set(false)
            },
        );
    }

    {
        let prepared_audio = prepared_audio.clone();
        let audio_progress = audio_progress.clone();
        let audio_failed_word = audio_failed_word.clone();
        let round = (*round).clone();
        use_effect_with((props.quiz_format.clone(), round, *audio_retry), move |(token, round, _)| {
            let alive = Rc::new(Cell::new(true));
            let owned = Rc::new(RefCell::new(Vec::<MatchingAudio>::new()));
            prepared_audio.set(None);
            audio_progress.set(0);
            audio_failed_word.set(None);
            let is_audio = matches!(token.as_str(), "audio-hanzi" | "audio-meaning");
            if is_audio {
                if let Some(Ok(question)) = round {
                    let words = question.words.clone();
                    let pending = alive.clone();
                    let prepared = owned.clone();
                    let result_state = prepared_audio.clone();
                    let progress = audio_progress.clone();
                    let failed_word = audio_failed_word.clone();
                    spawn_local(async move {
                        let mut result = Ok(());
                        if !local_speech::enabled() {
                            result = Err("Enable Local Speech in Settings before starting an audio matching exercise.".to_owned());
                        } else {
                            for (index, word) in words.iter().enumerate() {
                                if !pending.get() { break; }
                                match local_speech::prepare_matching_audio(&word.hanzi).await {
                                    Ok(audio) => {
                                        if !pending.get() {
                                            local_speech::dispose_matching_audio(&audio);
                                            break;
                                        }
                                        prepared.borrow_mut().push(audio);
                                        progress.set(index + 1);
                                    }
                                    Err(error) => {
                                        let detail = error.as_string().unwrap_or_else(|| "Speech synthesis failed".to_owned());
                                        log::error!("Audio matching TTS failed for word {} of {} ({}): {}", index + 1, words.len(), word.hanzi, detail);
                                        if pending.get() {
                                            failed_word.set(Some((index + 1, word.hanzi.clone())));
                                        }
                                        result = Err(if detail.contains("maximum acoustic frames without EOS") {
                                            "Speech generation did not finish for this word. Try again or choose different vocabulary.".to_owned()
                                        } else {
                                            "Could not prepare this audio prompt. Check your speech service and try again.".to_owned()
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                        if pending.get() {
                            result_state.set(Some(result.map(|()| prepared.borrow().clone())));
                        }
                    });
                }
            }
            move || {
                alive.set(false);
                if is_audio { local_speech::cancel(); }
                for audio in owned.borrow().iter() { local_speech::dispose_matching_audio(audio); }
            }
        });
    }

    {
        let last_incorrect = last_incorrect.clone();
        use_effect_with(*last_incorrect, move |incorrect| {
            let timeout = incorrect.map(|_| {
                let last_incorrect = last_incorrect.clone();
                Timeout::new(650, move || last_incorrect.set(None))
            });
            move || drop(timeout)
        });
    }

    let on_pick_pair = {
        let round = round.clone();
        let selected = selected.clone();
        let matched = matched.clone();
        let wrong_attempts = wrong_attempts.clone();
        let pairing_error = pairing_error.clone();
        let last_incorrect = last_incorrect.clone();
        Callback::from(move |right_id: i32| {
            let Some(left_id) = *selected else {
                return;
            };
            let Some(Ok(question)) = &*round else {
                return;
            };
            match question.evaluate_pair(&matched, left_id, right_id) {
                Ok(PairAttempt::Correct) => {
                    let mut updated = (*matched).clone();
                    updated.insert(right_id);
                    matched.set(updated);
                    pairing_error.set(None);
                    last_incorrect.set(None);
                }
                Ok(PairAttempt::AlreadyMatched) => {
                    pairing_error.set(Some("This pair has already been matched.".to_owned()));
                }
                Ok(PairAttempt::Incorrect) => {
                    wrong_attempts.set(wrong_attempts.saturating_add(1));
                    pairing_error.set(Some("Not a match. Try another pair.".to_owned()));
                    last_incorrect.set(Some((left_id, right_id)));
                }
                Err(error) => {
                    pairing_error.set(Some(format!("Cannot validate this pair: {error:?}")));
                }
            }
            selected.set(None);
        })
    };
    let on_continue = {
        let on_completed = props.on_completed.clone();
        Callback::from(move |_: MouseEvent| on_completed.emit(()))
    };
    let body = match &*round {
        None => html! { <p role="status">{"Loading vocabulary..."}</p> },
        Some(Err(reason)) => html! { <p role="alert">{reason}</p> },
        Some(Ok(question)) => match props.mode {
            LocalQuizMode::MultipleChoice => {
                let verdict = (*selected).and_then(|id| question.grade(id).ok());
                let prompt = question.prompt_for(question.answer());
                let hanzi_prompt = matches!(
                    props.quiz_format.as_str(),
                    "hanzi-to-meaning" | "hanzi-eng" | "hanzi-to-pinyin"
                );
                html! {
                    <div class="space-y-6" data-mcq-exercise="">
                        <div class="space-y-3 rounded-xl border border-line bg-raised/60 px-4 py-6 text-center sm:py-8">
                            <p class="text-xs font-semibold uppercase tracking-widest text-muted">{"Choose the correct answer"}</p>
                            <p lang={if hanzi_prompt {"zh"} else {"en"}}
                                class={if hanzi_prompt {"font-[family-name:var(--hanzi-font)] text-4xl font-semibold tracking-wide text-ink sm:text-5xl"} else {"text-2xl font-semibold text-ink sm:text-3xl"}}
                                data-mcq-prompt="">{prompt}</p>
                        </div>
                        <div role="group" aria-label="Choose the correct answer" class="grid gap-3 sm:grid-cols-2">
                            {for question.words.iter().map(|word| {
                                let id = word.id;
                                let is_selected = *selected == Some(id);
                                let is_correct_answer = id == question.answer().id;
                                let selected_for_click = selected.clone();
                                let on_pick = Callback::from(move |_: MouseEvent| selected_for_click.set(Some(id)));
                                let appearance = match verdict {
                                    Some(_) if is_correct_answer => "border-emerald-500 bg-emerald-500/20 text-ink ring-1 ring-emerald-500/60",
                                    Some(false) if is_selected => "border-rose-500 bg-rose-500/20 text-ink ring-1 ring-rose-500/60",
                                    Some(_) => "border-line bg-raised/60 text-muted",
                                    None if is_selected => "border-brand-action bg-accent-soft text-ink ring-2 ring-brand-action/50",
                                    None => "border-line bg-raised text-ink hover:border-brand-action hover:bg-accent-soft",
                                };
                                html! {
                                    <button type="button" key={id}
                                        class={classes!("min-h-20", "rounded-xl", "border-2", "px-5", "py-4", "text-lg", "font-semibold", "transition-[border-color,background-color,box-shadow,transform]", "duration-200", "active:scale-[0.985]", "motion-reduce:transition-none", "focus-visible:outline-2", "focus-visible:outline-accent", appearance)}
                                        data-mcq-correct={(verdict.is_some() && is_correct_answer).then_some("")}
                                        data-mcq-incorrect={(verdict == Some(false) && is_selected).then_some("")}
                                        aria-pressed={is_selected.to_string()}
                                        disabled={verdict.is_some()}
                                        onclick={on_pick}>
                                        <span class="block text-center">{question.option_for(word)}</span>
                                    </button>
                                }
                            })}
                        </div>
                        {if let Some(correct) = verdict {
                            html! {
                                <div class={if correct {"space-y-4 rounded-xl border border-emerald-500/50 bg-emerald-500/10 p-4 sm:p-5"} else {"space-y-4 rounded-xl border border-rose-500/50 bg-rose-500/10 p-4 sm:p-5"}}
                                    role="status" data-mcq-feedback="">
                                    <div class="space-y-1">
                                        <p class={if correct {"text-lg font-bold text-emerald-500"} else {"text-lg font-bold text-rose-400"}}>
                                            {if correct {"Correct"} else {"Incorrect"}}
                                        </p>
                                        {if !correct { html! { <p class="text-sm text-ink">{format!("Correct answer: {}", question.option_for(question.answer()))}</p> } } else { Html::default() }}
                                    </div>
                                    <button type="button" data-local-quiz-continue=""
                                        class="inline-flex min-h-11 w-full items-center justify-center rounded-xl bg-brand-action sm:w-auto px-6 py-3 text-sm font-semibold text-white transition-colors hover:bg-brand-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
                                        onclick={on_continue.clone()}>{"Continue"}</button>
                                </div>
                            }
                        } else { Html::default() }}
                    </div>
                }
            }
            LocalQuizMode::Matching => {
                let audio_mode = matches!(props.quiz_format.as_str(), "audio-hanzi" | "audio-meaning");
                let audio_hanzi = props.quiz_format == "audio-hanzi";
                let reverse = props.reverse_matching && !audio_mode;
                let all_matched = matched.len() == question.words.len();
                let answer_order = if audio_mode {
                    let offset = 1 + (props.seed as usize % (question.words.len() - 1));
                    question.words.iter().cycle().skip(offset).take(question.words.len()).collect::<Vec<_>>()
                } else {
                    question.words.iter().rev().collect::<Vec<_>>()
                };
                let first_label = if reverse {
                    "Answers"
                } else {
                    "Prompts"
                };
                let second_label = if reverse {
                    "Prompts"
                } else {
                    "Answers"
                };
                if audio_mode && prepared_audio.is_none() {
                    html! {
                        <div class="mx-auto flex w-full max-w-md flex-col items-center gap-5 py-6 text-center" data-audio-preparing="" aria-busy="true">
                            <span class="loading-spinner" aria-hidden="true" />
                            <div class="space-y-2">
                                <h2 class="text-lg font-semibold text-ink">{"Preparing..."}</h2>
                                <p role="status" aria-live="polite" class="sr-only" data-audio-preparation-progress="">
                                    {format!("{} of {} audio prompts ready", *audio_progress, question.words.len())}
                                </p>
                            </div>
                            <div class="h-1.5 w-full overflow-hidden rounded-full bg-raised" role="progressbar"
                                aria-label="Audio preparation" aria-valuemin="0"
                                aria-valuemax={question.words.len().to_string()}
                                aria-valuenow={audio_progress.to_string()}>
                                <div class="h-full rounded-full bg-brand-action transition-[width] duration-300 motion-reduce:transition-none"
                                    style={format!("width: {}%", *audio_progress * 100 / question.words.len().max(1))} />
                            </div>
                        </div>
                    }
                } else if audio_mode && prepared_audio.as_ref().is_some_and(Result::is_err) {
                    let error = prepared_audio.as_ref().and_then(|v| v.as_ref().err()).cloned().unwrap_or_default();
                    let on_retry = {
                        let audio_retry = audio_retry.clone();
                        Callback::from(move |_: MouseEvent| audio_retry.set(audio_retry.wrapping_add(1)))
                    };
                    let failed = (*audio_failed_word).clone();
                    html! {
                        <div class="mx-auto flex w-full max-w-lg flex-col gap-4 py-4" data-audio-preparation-failed="">
                            <div class="space-y-1">
                                <h2 class="text-lg font-semibold text-ink">{"Audio could not be prepared"}</h2>
                                <p class="text-sm text-muted" role="alert">{error}</p>
                                {if let Some((index, hanzi)) = failed {
                                    html! { <p class="text-sm text-ink" data-audio-failed-word="">
                                        {format!("Failed on word {index} of {}: {hanzi}", question.words.len())}
                                    </p> }
                                } else { Html::default() }}
                            </div>
                            <div class="rounded-lg border border-line bg-raised/50 px-4 py-3">
                                <p class="mb-2 text-xs font-medium text-muted">{"Words selected for this round"}</p>
                                <div class="flex flex-wrap gap-2" data-audio-selected-words="">
                                    {for question.words.iter().map(|word| html! {
                                        <span key={word.id} lang="zh" class="rounded-md border border-line px-2.5 py-1 text-sm font-[family-name:var(--hanzi-font)] text-ink">{&word.hanzi}</span>
                                    })}
                                </div>
                            </div>
                            <button type="button" class="inline-flex min-h-11 w-fit items-center rounded-lg bg-brand-action px-5 py-2 text-sm font-semibold text-white hover:bg-brand-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent" onclick={on_retry}>
                                {"Try again"}
                            </button>
                        </div>
                    }
                } else {
                html! {
                    <div class="space-y-4">
                        <p role="status" aria-live="polite" data-matching-selection="" class="sr-only">
                            {if all_matched { "All pairs matched".to_owned() } else if let Some(id) = *selected { format!("Prompt {id} selected. Choose the matching answer.") } else { "Choose a prompt to begin".to_owned() }}
                        </p>
                        <div data-matching-board="" class="grid grid-cols-2 gap-2 sm:gap-4">
                            <div role="group" aria-label={first_label} class="space-y-2">
                                {for question.words.iter().map(|word| {
                                    let id = word.id;
                                    let is_selected = *selected == Some(id);
                                    let selected_for_click = selected.clone();
                                    let error_for_click = pairing_error.clone();
                                    let last_incorrect_for_click = last_incorrect.clone();
                                    let disabled = matched.contains(&id);
                                    let was_wrong = (*last_incorrect).is_some_and(|(left, _)| left == id);
                                    let is_hanzi = match props.quiz_format.as_str() {
                                        "hanzi-eng" => !reverse,
                                        "pinyin-hanzi" => reverse,
                                        _ => false,
                                    };
                                    let playable = if audio_mode {
                                        question.words.iter().position(|word| word.id == id)
                                            .and_then(|index| prepared_audio.as_ref().and_then(|value| value.as_ref().ok()).and_then(|items| items.get(index))).cloned()
                                    } else { None };
                                    let on_pick = Callback::from(move |_: MouseEvent| {
                                        selected_for_click.set(Some(id));
                                        error_for_click.set(None);
                                        last_incorrect_for_click.set(None);
                                        if let Some(audio) = playable.clone() {
                                            spawn_local(async move {
                                                if let Err(error) = local_speech::play_matching_audio(&audio).await {
                                                    log::warn!("Matching audio playback failed: {error:?}");
                                                }
                                            });
                                        }
                                    });
                                    html! {
                                        <button
                                            type="button"
                                            key={id}
                                            class={classes!("relative", "block", "min-h-16", "w-full", "rounded-xl", "border-2", "px-3", "py-4", "text-left", "font-medium", "transition-all", "duration-200", "motion-reduce:transition-none", "focus-visible:outline", "focus-visible:outline-2", "focus-visible:outline-accent", "disabled:opacity-80",
                                                if disabled {"border-emerald-500 bg-emerald-500/20 text-ink"} else if was_wrong {"border-rose-500 bg-rose-500/20 text-ink motion-safe:animate-match-shake"} else if is_selected {"border-brand-action bg-accent-soft text-ink shadow-[inset_0_0_0_2px_var(--brand-action)] ring-2 ring-brand-action"} else {"border-line bg-raised text-ink hover:border-brand-action hover:bg-accent-soft"})}
                                            data-matching-selected={is_selected.then_some("")}
                                            data-matching-matched={disabled.then_some("")}
                                            aria-pressed={is_selected.to_string()}
                                            aria-label={if audio_mode { format!("Play audio prompt {} and select it", question.words.iter().position(|item| item.id == id).unwrap_or(0) + 1) } else { (if reverse {question.option_for(word)} else {question.prompt_for(word)}).to_owned() }}
                                            disabled={disabled}
                                            onclick={on_pick}
                                        >
                                            {if audio_mode {
                                                let index = question.words.iter().position(|item| item.id == id).unwrap_or(0);
                                                let audio = prepared_audio.as_ref().and_then(|value| value.as_ref().ok()).and_then(|items| items.get(index)).cloned();
                                                if let Some(audio) = audio {
                                                    let peaks = audio.peaks.clone();
                                                    html! { <span class="flex items-center justify-center gap-3" aria-hidden="true">
                                                        <span class="text-xl">{"▶"}</span>
                                                        <span class="flex h-9 flex-1 items-center justify-center gap-[2px]">
                                                            {for peaks.iter().map(|peak| html! { <span class="w-[3px] rounded-full bg-sky-300" style={format!("height: {}%", (*peak * 85.0).max(12.0))}></span> })}
                                                        </span>
                                                        <span class="text-xs text-muted">{format!("{:.1}s", audio.duration_seconds)}</span>
                                                    </span> }
                                                } else { Html::default() }
                                            } else {
                                                let text = if reverse { question.option_for(word) } else { question.prompt_for(word) };
                                                html! {<>
                                                    <span lang={if is_hanzi {"zh"} else {"en"}} class={classes!("block", "text-center", "leading-snug", if is_hanzi {"font-[family-name:var(--hanzi-font)] text-2xl sm:text-3xl font-semibold tracking-wide"} else {"text-base"})}>{text}</span>
                                                    {if is_hanzi && props.show_pinyin { html! {<span class="mt-1 block text-center text-sm text-muted">{&word.pinyin}</span>} } else { Html::default() }}
                                                </>}
                                            }}
                                        </button>
                                    }
                                })}
                            </div>
                            <div role="group" aria-label={second_label} class="space-y-2">
                                {for answer_order.into_iter().map(|word| {
                                    let id = word.id;
                                    let label = if reverse {
                                        question.prompt_for(word)
                                    } else {
                                        question.option_for(word)
                                    }.to_owned();
                                    let disabled = matched.contains(&id);
                                    let was_wrong = (*last_incorrect).is_some_and(|(_, right)| right == id);
                                    let is_hanzi = if audio_mode { audio_hanzi } else { match props.quiz_format.as_str() {
                                        "hanzi-eng" => reverse,
                                        "pinyin-hanzi" => !reverse,
                                        _ => false,
                                    }};
                                    let on_pick_pair = on_pick_pair.clone();
                                    let on_pick = Callback::from(move |_: MouseEvent| on_pick_pair.emit(id));
                                    html! {
                                        <button
                                            type="button"
                                            key={id}
                                            class={classes!("block", "min-h-16", "w-full", "rounded-xl", "border-2", "px-3", "py-4", "transition-all", "duration-200", "motion-reduce:transition-none", "focus-visible:outline", "focus-visible:outline-2", "focus-visible:outline-accent", "disabled:opacity-80",
                                                if disabled {"border-emerald-500 bg-emerald-500/20 text-ink"} else if was_wrong {"border-rose-500 bg-rose-500/20 text-ink motion-safe:animate-match-shake"} else {"border-line bg-raised text-ink hover:border-brand-action hover:bg-accent-soft"})}
                                            data-matching-matched={disabled.then_some("")}
                                            disabled={disabled}
                                            onclick={on_pick}
                                        ><span lang={if is_hanzi {"zh"} else {"en"}} class={classes!("block", "text-center", "leading-snug", if is_hanzi {"font-[family-name:var(--hanzi-font)] text-2xl sm:text-3xl font-semibold tracking-wide"} else {"text-base"})}>{label}</span>
                                            {if is_hanzi && props.show_pinyin { html! {<span class="mt-1 block text-center text-sm text-muted">{&word.pinyin}</span>} } else { Html::default() }}</button>
                                    }
                                })}
                            </div>
                        </div>
                        <p data-matching-progress="" class="text-center text-sm font-medium text-muted" aria-label={format!("{} of {} pairs matched, {} incorrect attempts", matched.len(), question.words.len(), *wrong_attempts)}>{format!("{} / {} · {} mistakes", matched.len(), question.words.len(), *wrong_attempts)}</p>
                        <p role="status" aria-live="polite" data-matching-feedback="" class="sr-only">
                            {(*pairing_error).as_deref().unwrap_or("")}
                        </p>
                        <button
                            type="button"
                            data-local-quiz-continue=""
                            class="block w-full rounded-lg bg-brand-action px-5 py-3 font-semibold text-white transition-opacity disabled:cursor-not-allowed disabled:opacity-40 motion-reduce:transition-none"
                            disabled={!all_matched}
                            onclick={on_continue.clone()}
                        >{"Continue"}</button>
                    </div>
                }
                }
            }
        },
    };
    html! {
        <section data-local-quiz="" aria-label={if props.mode == LocalQuizMode::Matching { "Matching exercise" } else { "Multiple choice exercise" }} class="mx-auto w-full max-w-4xl rounded-2xl border border-line bg-surface p-5 shadow-xl sm:p-8">
            {body}
        </section>
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;

    #[test]
    fn saved_quiz_format_must_match_the_active_quiz_host() {
        assert!(selected_quiz_format("unknown-format", LocalQuizMode::Matching).is_err());
        for token in [
            "hanzi-eng",
            "eng-hanzi",
            "hanzi-to-meaning",
            "meaning-to-hanzi",
        ] {
            if let Some(variant) = ExerciseVariant::from_seed_token(token) {
                let correct_mode = if variant.is_mcq() {
                    LocalQuizMode::MultipleChoice
                } else {
                    LocalQuizMode::Matching
                };
                let wrong_mode = if variant.is_mcq() {
                    LocalQuizMode::Matching
                } else {
                    LocalQuizMode::MultipleChoice
                };
                assert_eq!(selected_quiz_format(token, correct_mode), Ok(variant));
                assert!(selected_quiz_format(token, wrong_mode).is_err());
            }
        }
    }
}
