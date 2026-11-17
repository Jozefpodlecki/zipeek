use std::{
    cell::Cell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_core::practice::ExerciseVariant;
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
            ),
            move |(vocabulary, seed, mode, token, pair_count)| {
                let alive = Rc::new(Cell::new(true));
                let pending = alive.clone();
                let vocabulary = vocabulary.clone();
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
                    let result = load_words(&dictionary, &vocabulary, true)
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
                    });
                    if pending.get() {
                        round.set(Some(result));
                    }
                });
                move || alive.set(false)
            },
        );
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
                let all_matched = matched.len() == question.words.len();
                let first_label = if props.reverse_matching {
                    "Answers"
                } else {
                    "Prompts"
                };
                let second_label = if props.reverse_matching {
                    "Prompts"
                } else {
                    "Answers"
                };
                html! {
                    <div class="space-y-4">
                        <p class="text-sm text-muted">{"Select the Chinese word, then choose its matching meaning."}</p>
                        <p role="status" aria-live="polite" data-matching-selection="" class={classes!("min-h-10", "rounded-lg", "border", "px-3", "py-2", "text-sm", "font-medium", "transition-colors", "duration-150", "motion-reduce:transition-none", if selected.is_some() {"border-brand-action bg-accent-soft text-ink"} else {"border-line bg-raised text-muted"})}>
                            {if let Some(selected_id) = *selected {
                                question.words.iter().find(|word| word.id == selected_id).map(|word| format!("Selected: {}. Choose the matching answer.", if props.reverse_matching {question.option_for(word)} else {question.prompt_for(word)})).unwrap_or_default()
                            } else if all_matched {
                                "All pairs matched. Continue when you are ready.".to_owned()
                            } else {
                                "Choose a word to begin.".to_owned()
                            }}
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
                                        "hanzi-eng" => !props.reverse_matching,
                                        "pinyin-hanzi" => props.reverse_matching,
                                        _ => false,
                                    };
                                    let on_pick = Callback::from(move |_: MouseEvent| {
                                        selected_for_click.set(Some(id));
                                        error_for_click.set(None);
                                        last_incorrect_for_click.set(None);
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
                                            disabled={disabled}
                                            onclick={on_pick}
                                        >
                                            <span lang={if is_hanzi {"zh"} else {"en"}} class={classes!("block", "text-center", "leading-snug", if is_hanzi {"font-[family-name:var(--hanzi-font)] text-2xl sm:text-3xl font-semibold tracking-wide"} else {"text-base"})}>{if props.reverse_matching { question.option_for(word) } else { question.prompt_for(word) }}</span>
                                            {if disabled {html! {<span class="mt-1 block text-center text-xs font-semibold text-emerald-500">{"Matched"}</span>}} else if is_selected {html! {<span class="mt-1 block text-center text-xs font-semibold text-accent">{"Selected"}</span>}} else {Html::default()}}
                                        </button>
                                    }
                                })}
                            </div>
                            <div role="group" aria-label={second_label} class="space-y-2">
                                {for question.words.iter().rev().map(|word| {
                                    let id = word.id;
                                    let label = if props.reverse_matching {
                                        question.prompt_for(word)
                                    } else {
                                        question.option_for(word)
                                    }.to_owned();
                                    let disabled = matched.contains(&id);
                                    let was_wrong = (*last_incorrect).is_some_and(|(_, right)| right == id);
                                    let is_hanzi = match props.quiz_format.as_str() {
                                        "hanzi-eng" => props.reverse_matching,
                                        "pinyin-hanzi" => !props.reverse_matching,
                                        _ => false,
                                    };
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
                                        ><span lang={if is_hanzi {"zh"} else {"en"}} class={classes!("block", "text-center", "leading-snug", if is_hanzi {"font-[family-name:var(--hanzi-font)] text-2xl sm:text-3xl font-semibold tracking-wide"} else {"text-base"})}>{label}</span>{if disabled {html! {<span class="mt-1 block text-center text-xs font-semibold text-emerald-500">{"Matched"}</span>}} else {Html::default()}}</button>
                                    }
                                })}
                            </div>
                        </div>
                        <p role="status" class="text-sm font-medium text-ink">{format!("{} of {} pairs matched · {} incorrect attempts", matched.len(), question.words.len(), *wrong_attempts)}</p>
                        {if let Some(error) = &*pairing_error {
                            html! { <p role="alert" class="rounded-lg border border-rose-500/50 bg-rose-500/10 px-3 py-2 text-sm text-rose-400 motion-safe:animate-match-shake">{error}</p> }
                        } else { Html::default() }}
                        {if all_matched {
                            html! {
                                <button
                                    type="button"
                                    data-local-quiz-continue=""
                                    class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white"
                                    onclick={on_continue.clone()}
                                >{"Continue"}</button>
                            }
                        } else { Html::default() }}
                    </div>
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
