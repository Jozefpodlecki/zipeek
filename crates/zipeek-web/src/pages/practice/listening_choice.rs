use crate::hooks::use_word_sets;
use std::{cell::Cell, collections::BTreeSet, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlSelectElement;
use yew::prelude::*;
use zipeek_core::{Word, WordId};
use zipeek_exercises::listening_choice::{ListeningQuestion, ListeningWord, prepare};

use crate::{
    hooks::{use_app_environment, use_dictionary},
    services::practice_audio,
};

#[function_component(ListeningChoicePage)]
pub fn listening_choice_page() -> Html {
    let dictionary = use_dictionary();
    let sets = use_word_sets();
    let environment = use_app_environment();
    let selected_set = use_state(|| None::<crate::services::word_sets::WordSetId>);
    let pool = use_state(|| None::<Result<Vec<Word>, String>>);
    let question = use_state(|| None::<ListeningQuestion>);
    let selected = use_state(|| None::<i32>);
    let error = use_state(String::new);
    let saved_sets = sets.state.sets.as_ref().ok();
    let ids = saved_sets
        .and_then(|items| {
            selected_set
                .as_ref()
                .and_then(|id| crate::services::word_sets::find(items, id))
                .or_else(|| {
                    if selected_set.is_none() {
                        items.first()
                    } else {
                        None
                    }
                })
        })
        .map(|set| set.word_ids.clone())
        .unwrap_or_default();
    {
        let dictionary = dictionary.clone();
        let pool = pool.clone();
        use_effect_with(ids, move |ids| {
            let active = Rc::new(Cell::new(true));
            let still_active = active.clone();
            let ids = ids.clone();
            pool.set(None);
            spawn_local(async move {
                let result = async {
                    let ids = ids
                        .iter()
                        .map(|id| {
                            WordId::try_from(i64::from(*id)).map_err(|error| error.to_string())
                        })
                        .collect::<Result<BTreeSet<_>, _>>()?;
                    dictionary
                        .database()
                        .map_err(|error| error.to_string())?
                        .practice_eligible_words_by_ids(&ids)
                        .await
                        .map_err(|error| error.to_string())
                }
                .await;
                if still_active.get() {
                    pool.set(Some(result));
                }
            });
            move || active.set(false)
        });
    }
    let change_set = {
        let selected_set = selected_set.clone();
        let question = question.clone();
        let selected = selected.clone();
        let error = error.clone();
        Callback::from(move |event: Event| {
            let input: HtmlSelectElement = event.target_unchecked_into();
            if let Ok(id) = input.value().parse() {
                selected_set.set(Some(id));
                question.set(None);
                selected.set(None);
                error.set(String::new());
            }
        })
    };
    let next_question = {
        let pool = pool.clone();
        let question = question.clone();
        let selected = selected.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let Some(Ok(words)) = &*pool else {
                error.set("Select a saved Word Set with available vocabulary".into());
                return;
            };
            let candidates = words
                .iter()
                .map(|word| ListeningWord {
                    id: word.id.as_i32(),
                    hanzi: word.simplified.clone(),
                })
                .collect::<Vec<_>>();
            let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
            match prepare(&candidates, seed) {
                Ok(next) => {
                    question.set(Some(next));
                    selected.set(None);
                    error.set(String::new());
                }
                Err(reason) => error.set(format!("Cannot create listening question: {reason:?}. A Word Set needs at least three distinct Chinese words.")),
            }
        })
    };
    let play = {
        let question = question.clone();
        let error = error.clone();
        let speech = environment.speech.clone();
        Callback::from(move |_| {
            if let Some(question) = &*question {
                if let Err(reason) = practice_audio::speak(&speech, &question.spoken_hanzi) {
                    error.set(format!("Mandarin speech unavailable: {reason:?}"));
                } else {
                    error.set(String::new());
                }
            }
        })
    };
    let choose = {
        let selected = selected.clone();
        Callback::from(move |id: i32| {
            if (*selected).is_none() {
                selected.set(Some(id));
            }
        })
    };
    let verdict = (*selected).and_then(|id| (*question).as_ref().and_then(|q| q.grade(id).ok()));
    let listening_question_content = {
        if let Some(question) = &*question {
            let answer_verdict_notice = {
                if let Some(correct) = verdict {
                    html! {
                        <p
                            role="status"
                            class="font-semibold"
                        >
                            {if correct {"Correct!"} else {"Not quite."}}{format!(" The answer is {}.", question.spoken_hanzi)}
                        </p>
                    }
                } else {
                    Html::default()
                }
            };
            html! {
                <div
                    class="space-y-4"
                >
                    <button
                        type="button"
                        class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white"
                        onclick={play}
                    >
                        {"▶ Play Mandarin audio"}
                    </button>
                    <div
                        role="group"
                        aria-label="Choose the word you heard"
                        class="grid gap-3 sm:grid-cols-3"
                    >
                        {for question.choices.iter().map(|choice| {
                            let id = choice.id;
                            let chosen = *selected == Some(id);
                            let correct = verdict.is_some() && id == question.correct_id;
                            let choose = choose.clone();
                            let on_choose_answer = Callback::from(move |_| choose.emit(id));
                            let element_classes_1 = classes!(
                                            "min-h-20",
                                            "rounded-xl",
                                            "border-2",
                                            "p-4",
                                            "text-2xl",
                                            "font-semibold",
                                            if correct {"border-green-600"} else if chosen {"border-brand-action"} else {"border-line"},
                                        );
                            html! {
                                <button
                                    key={id}
                                    type="button"
                                    disabled={(*selected).is_some()}
                                    aria-pressed={chosen.to_string()}
                                    class={element_classes_1}
                                    onclick={on_choose_answer}
                                >
                                    {&choice.hanzi}
                                </button>
                            }
                        })}
                    </div>
                    {answer_verdict_notice}
                </div>
            }
        } else {
            Html::default()
        }
    };
    let vocabulary_pool_status = {
        match &*pool {
            None => html! { <p
                role="status"
            >
            {"Loading saved vocabulary…"}
            </p> },
            Some(Err(reason)) => html! { <p
                role="alert"
            >
            {reason}
            </p> },
            Some(Ok(words)) if words.len() < 3 => html! { <p
                role="status"
            >
            {"This set needs at least three distinct words."}
            </p> },
            _ => Html::default(),
        }
    };

    let empty_saved_sets_notice = {
        if saved_sets.is_some_and(|items| items.is_empty()) {
            html! {
                <p
                    role="status"
                    class="text-sm text-muted"
                >
                    {"Create a Word Set in Browse before starting listening practice."}
                </p>
            }
        } else {
            Html::default()
        }
    };
    let listening_choice_error = if !error.is_empty() {
        html! { <p
            role="alert"
            class="text-sm text-red-400"
        >
            {&*error}
        </p> }
    } else {
        Html::default()
    };
    html! {
        <section
            data-listening-choice=""
            class="mx-auto max-w-3xl space-y-5 rounded-2xl border border-line bg-surface p-5"
        >
            <header
                class="space-y-2"
            >
                <h1
                    class="text-2xl font-semibold"
                >
                    {"Listening practice"}
                </h1>
                <p
                    class="text-sm text-muted"
                >
                    {"Listen to Mandarin and choose the matching Chinese word. Audio is generated by your browser and may depend on available voices."}
                </p>
            </header>
            <label
                class="flex max-w-sm flex-col gap-1 text-sm"
            >
                {"Word Set"}
                <select
                    class="rounded-lg border border-line bg-raised p-2"
                    onchange={change_set}
                    value={selected_set.as_ref().map(ToString::to_string).or_else(|| saved_sets.and_then(|sets| sets.first()).map(|set| set.id.to_string())).unwrap_or_default()}
                >
                    {for saved_sets.into_iter().flatten().map(|set| html! {
                        <option
                            value={set.id.to_string()}
                            key={set.id.to_string()}
                        >
                            {format!("{} ({} words)", set.name, set.word_ids.len())}
                        </option>
                    })}
                </select>
            </label>
            {empty_saved_sets_notice}
            {vocabulary_pool_status}
            {listening_question_content}
            <button
                type="button"
                class="rounded-lg border border-line bg-raised px-5 py-3 font-semibold"
                onclick={next_question}
                disabled={!matches!(&*pool, Some(Ok(words)) if words.len() >= 3)}
            >
                {if (*question).is_some() {"Next question"} else {"Start listening"}}
            </button>
            {listening_choice_error}
        </section>
    }
}
