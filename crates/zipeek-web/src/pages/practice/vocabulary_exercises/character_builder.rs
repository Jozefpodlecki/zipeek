use yew::prelude::*;
use zipeek_core::Word;

use crate::{
    components::ListenButton,
    models::{AnswerPolicy, AudioPreference, CharacterBuilderOptions},
};

use super::{
    answer::{AnswerAction, AnswerPhase, AnswerState},
    character::{add_tile, assembled_answer, builder_bank, reading_for_character, remove_tile},
    vocabulary::concise_prompt_meaning,
};

#[derive(Properties, PartialEq)]
pub(super) struct CharacterBuilderProps {
    pub word: Word,
    pub words: Vec<Word>,
    pub seed: u64,
    pub options: CharacterBuilderOptions,
    pub on_completed: Callback<u32>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(CharacterBuilder)]
pub(super) fn character_builder(props: &CharacterBuilderProps) -> Html {
    let picked = use_state(Vec::<usize>::new);
    let dragged_tile = use_state(|| None::<usize>);
    let answer_state = use_reducer(AnswerState::default);
    let bank = builder_bank(&props.word, &props.words, props.seed);
    let assembled = assembled_answer(&bank, &picked).unwrap_or_default();
    let correct = assembled == props.word.simplified;
    let reviewed = matches!(answer_state.phase, AnswerPhase::Reviewed { .. });
    let capacity = props.word.simplified.chars().count();
    let retry = props.options.answer_policy == AnswerPolicy::RetryUntilCorrect;
    let auto_error = use_state(|| false);
    let leaving = use_state(|| false);
    let pending_advance = use_mut_ref(|| None::<gloo::timers::callback::Timeout>);
    {
        let pending_advance = pending_advance.clone();
        use_effect_with((), move |_| {
            move || {
                pending_advance.borrow_mut().take();
            }
        });
    }
    {
        let answer_state = answer_state.clone();
        let picked = picked.clone();
        let error = auto_error.clone();
        let bank = bank.clone();
        let expected = props.word.simplified.clone();
        use_effect_with(
            ((*picked).clone(), props.options.automatic_check),
            move |(selection, enabled)| {
                let reset = if *enabled
                    && selection.len() == capacity
                    && matches!(answer_state.phase, AnswerPhase::Editing)
                {
                    let correct =
                        assembled_answer(&bank, selection).is_some_and(|value| value == expected);
                    answer_state.dispatch(AnswerAction::Submit { correct, retry });
                    if !correct && retry {
                        error.set(true);
                        let picked = picked.clone();
                        let error = error.clone();
                        Some(gloo::timers::callback::Timeout::new(500, move || {
                            picked.set(Vec::new());
                            error.set(false);
                        }))
                    } else {
                        None
                    }
                } else {
                    None
                };
                move || drop(reset)
            },
        );
    }
    let on_check = {
        let answer_state = answer_state.clone();
        let picked = picked.clone();
        Callback::from(move |_| {
            answer_state.dispatch(AnswerAction::Submit { correct, retry });
            if !correct && retry {
                picked.set(Vec::new());
            }
        })
    };
    let on_drop_tile = {
        let picked = picked.clone();
        let dragged_tile = dragged_tile.clone();
        let bank_len = bank.len();
        let answer_state = answer_state.clone();
        let auto_error = auto_error.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            if let Some(tile) = *dragged_tile {
                if !reviewed && !*auto_error {
                    answer_state.dispatch(AnswerAction::Edit);
                    picked.set(add_tile(&picked, tile, bank_len, capacity));
                }
                dragged_tile.set(None);
            }
        })
    };
    let on_drag_over = Callback::from(|event: DragEvent| event.prevent_default());
    let on_continue = {
        let on_completed = props.on_completed.clone();
        let answer_state = answer_state.clone();
        let leaving = leaving.clone();
        let pending_advance = pending_advance.clone();
        Callback::from(move |_| {
            if *leaving {
                return;
            }
            leaving.set(true);
            let on_completed = on_completed.clone();
            let mistakes = answer_state.mistakes;
            *pending_advance.borrow_mut() =
                Some(gloo::timers::callback::Timeout::new(180, move || {
                    on_completed.emit(mistakes);
                }));
        })
    };

    let slot_class = if *auto_error {
        "flex min-h-20 min-w-20 flex-col items-center justify-center rounded-xl border-2 border-rose-400 bg-rose-500/20 text-rose-100"
    } else {
        "flex min-h-20 min-w-20 flex-col items-center justify-center rounded-xl border-2 border-dashed border-line bg-raised"
    };
    let element_classes_1 = classes!(
        "practice-round",
        "space-y-5",
        "text-center",
        if *leaving {
            "practice-round-leaving"
        } else {
            ""
        }
    );
    html! {
        <div class={element_classes_1} data-exercise="character-builder" data-auto-check={props.options.automatic_check.to_string()}>
            <h2 class="text-3xl font-bold">{concise_prompt_meaning(&props.word).unwrap_or_default()}</h2>
            <div role="group" aria-label="Selected characters" class="flex min-h-24 flex-wrap items-center justify-center gap-3" ondragover={on_drag_over} ondrop={on_drop_tile}>
                {for (0..capacity).map(|slot| {
                    let picked = picked.clone();
                    let character = assembled.chars().nth(slot).map(|ch| ch.to_string()).unwrap_or_default();
                    let reading = character.chars().next().map(|ch| reading_for_character(ch, &props.word, &props.words)).unwrap_or_default();
                    let disabled = reviewed || *auto_error || slot >= picked.len();
                    let answer_state = answer_state.clone();
                    let on_remove = Callback::from(move |_| {
                        answer_state.dispatch(AnswerAction::Edit);
                        picked.set(remove_tile(&picked, slot));
                    });
                    html! {
                        <button
                            type="button"
                            disabled={disabled}
                            onclick={on_remove}
                            class={slot_class}
                            aria-label={format!("Slot {}", slot + 1)}
                        >
                            <span class="character-builder-hanzi" lang="zh-Hans">{character}</span>
                            <span class="character-builder-reading text-muted" lang="zh-Latn" data-character-reading="">{reading}</span>
                        </button>
                    }
                })}
            </div>
            <div class="border-t border-line pt-6 flex flex-wrap justify-center gap-3" role="group" aria-label="Available characters">
                {for bank.iter().enumerate().map(|(tile, character)| {
                    let picked = picked.clone();
                    let disabled = reviewed || *auto_error || picked.contains(&tile) || picked.len() == capacity;
                    let reading = reading_for_character(*character, &props.word, &props.words);
                    let available = bank.len();
                    let answer_state = answer_state.clone();
                    let on_add = Callback::from(move |_| {
                        answer_state.dispatch(AnswerAction::Edit);
                        picked.set(add_tile(&picked, tile, available, capacity));
                    });
                    let on_drag_start = {
                        let dragged_tile = dragged_tile.clone();
                        Callback::from(move |_: DragEvent| dragged_tile.set(Some(tile)))
                    };
                    let on_drag_end = {
                        let dragged_tile = dragged_tile.clone();
                        Callback::from(move |_: DragEvent| dragged_tile.set(None))
                    };
                    html! {
                        <button
                            type="button"
                            disabled={disabled}
                            onclick={on_add}
                            draggable="true"
                            ondragstart={on_drag_start}
                            ondragend={on_drag_end}
                            class="min-h-20 min-w-20 flex flex-col items-center justify-center rounded-xl border border-line bg-raised disabled:opacity-30"
                            lang="zh-Hans"
                        >
                            <span class="character-builder-hanzi" lang="zh-Hans">{character}</span>
                            <span class="character-builder-reading text-muted" lang="zh-Latn" data-character-reading="">{reading}</span>
                        </button>
                    }
                })}
            </div>
            {if props.options.audio == AudioPreference::Manual {
                html! { <div class="flex justify-center" data-practice-audio=""><ListenButton hanzi={props.word.simplified.clone()} /></div> }
            } else { Html::default() }}
            <div class="flex min-h-12 items-center justify-center" data-practice-answer-feedback="">
                {if *auto_error {
                    html! { <span role="status" class="text-sm font-semibold text-rose-200">{"Not quite — try another arrangement"}</span> }
                } else if let AnswerPhase::Reviewed { correct } = answer_state.phase {
                    html! {
                        <div role="status" class={if correct { "inline-flex items-center justify-center rounded-full border border-emerald-400/40 bg-emerald-400/10 px-5 py-2 font-semibold text-emerald-200" } else { "inline-flex flex-col items-center justify-center rounded-xl border border-rose-400/40 bg-rose-400/10 px-5 py-2 text-rose-200" }}>
                            <strong>{if correct { "Correct" } else { "Incorrect" }}</strong>
                            {if correct { Html::default() } else { html! { <p>{format!("Correct answer: {}", props.word.simplified)}</p> } }}
                        </div>
                    }
                } else if answer_state.retry_notice && !props.options.automatic_check {
                    html! { <p role="status" data-practice-retry-notice="">{"Try again"}</p> }
                } else {
                    Html::default()
                }}
            </div>
            <div class="flex flex-wrap items-center justify-between gap-3">
            {for props.children.iter()}
            {if reviewed {
                html! { <button type="button" data-exercise-continue="" onclick={on_continue} disabled={*leaving} class="rounded-xl bg-brand-action px-6 py-4 font-bold text-white">{"Continue"}</button> }
            } else if !props.options.automatic_check {
                html! { <button type="button" data-exercise-check="" onclick={on_check} disabled={picked.len() != capacity} class="rounded-xl bg-brand-action px-6 py-4 font-bold text-white disabled:opacity-40">{"Check answer"}</button> }
            } else { Html::default() }}
            </div>
        </div>
    }
}
