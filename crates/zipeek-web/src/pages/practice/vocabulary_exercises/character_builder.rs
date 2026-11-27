use crate::services::practice_session_store::{
    AssessmentCapability, AssessmentEvidence, ExerciseVerdict,
};
use wasm_bindgen::JsCast;
use web_sys::PointerEvent;
use yew::prelude::*;
use zipeek_core::Word;

use crate::{
    components::ListenButton,
    models::{AnswerPolicy, AudioPreference, CharacterBuilderOptions},
    pages::practice::{
        feedback::{ExercisePrimaryAction, ExerciseReview, FeedbackVerdict},
        surface::ExerciseSurface,
    },
};

use super::{
    answer::{AnswerAction, AnswerPhase, AnswerState},
    character::{
        add_tile, assembled_answer, builder_bank, place_tile, reading_for_character, remove_tile,
    },
    progress::VocabularyRoundResult,
    vocabulary::concise_prompt_meaning,
};

#[derive(Properties, PartialEq)]
pub(super) struct CharacterBuilderProps {
    pub word: Word,
    pub words: Vec<Word>,
    pub seed: u64,
    pub options: CharacterBuilderOptions,
    pub on_completed: Callback<VocabularyRoundResult>,
    #[prop_or_default]
    pub on_review_change: Callback<bool>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(CharacterBuilder)]
pub(super) fn character_builder(props: &CharacterBuilderProps) -> Html {
    let picked = use_state(Vec::<usize>::new);
    let dragged_tile = use_state(|| None::<usize>);
    let pointer_drag = use_mut_ref(|| None::<(usize, i32, i32, bool)>);
    let suppress_click = use_mut_ref(|| 0.0_f64);
    let answer_state = use_reducer(AnswerState::default);
    let bank = builder_bank(&props.word, &props.words, props.seed);
    let assembled = assembled_answer(&bank, &picked).unwrap_or_default();
    let correct = assembled == props.word.simplified;
    let reviewed = matches!(answer_state.phase, AnswerPhase::Reviewed { .. });
    {
        let on_review_change = props.on_review_change.clone();
        let ready = matches!(answer_state.phase, AnswerPhase::Reviewed { correct: true });
        use_effect_with(ready, move |ready| {
            on_review_change.emit(*ready);
        });
    }
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
    {
        let pointer_drag = pointer_drag.clone();
        let suppress_click = suppress_click.clone();
        let picked = picked.clone();
        let answer_state = answer_state.clone();
        let available = bank.len();
        use_effect_with(
            ((*picked).clone(), reviewed, *auto_error, capacity),
            move |(_, reviewed, error, capacity)| {
                let blocked = *reviewed || *error;
                let capacity = *capacity;
                let document = web_sys::window().and_then(|window| window.document());
                let listeners = document.map(|document| {
                    let target_document = document.clone();
                    let pointer_drag_for_up = pointer_drag.clone();
                    let suppress_click = suppress_click.clone();
                    let picked = picked.clone();
                    let answer_state = answer_state.clone();
                    let up =
                        gloo::events::EventListener::new(&document, "pointerup", move |event| {
                            let Some((tile, start_x, start_y, was_placed)) =
                                pointer_drag_for_up.borrow_mut().take()
                            else {
                                return;
                            };
                            let Some(event) = event.dyn_ref::<PointerEvent>() else {
                                return;
                            };
                            if (event.client_x() - start_x).abs()
                                + (event.client_y() - start_y).abs()
                                < 12
                            {
                                return;
                            }
                            let Some(target) = target_document.element_from_point(
                                event.client_x() as f32,
                                event.client_y() as f32,
                            ) else {
                                return;
                            };
                            if blocked {
                                return;
                            }
                            if let Ok(Some(slot)) = target.closest("[data-character-slot]") {
                                if let Some(position) = slot
                                    .get_attribute("data-character-slot")
                                    .and_then(|slot| slot.parse::<usize>().ok())
                                {
                                    let next =
                                        place_tile(&picked, tile, available, capacity, position);
                                    if next != *picked {
                                        answer_state.dispatch(AnswerAction::Edit);
                                        picked.set(next);
                                        *suppress_click.borrow_mut() = js_sys::Date::now() + 350.0;
                                    }
                                }
                            } else if was_placed
                                && target
                                    .closest("[data-character-bank]")
                                    .ok()
                                    .flatten()
                                    .is_some()
                            {
                                if let Some(index) =
                                    picked.iter().position(|selected| *selected == tile)
                                {
                                    answer_state.dispatch(AnswerAction::Edit);
                                    picked.set(remove_tile(&picked, index));
                                    *suppress_click.borrow_mut() = js_sys::Date::now() + 350.0;
                                }
                            }
                        });
                    let pointer_drag = pointer_drag.clone();
                    let cancel =
                        gloo::events::EventListener::new(&document, "pointercancel", move |_| {
                            pointer_drag.borrow_mut().take();
                        });
                    (up, cancel)
                });
                move || drop(listeners)
            },
        );
    }
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
    let on_drop_bank = {
        let picked = picked.clone();
        let dragged_tile = dragged_tile.clone();
        let answer_state = answer_state.clone();
        let auto_error = auto_error.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            if !reviewed && !*auto_error {
                if let Some(tile) = *dragged_tile {
                    if let Some(slot) = picked.iter().position(|selected| *selected == tile) {
                        answer_state.dispatch(AnswerAction::Edit);
                        picked.set(remove_tile(&picked, slot));
                    }
                }
            }
            dragged_tile.set(None);
        })
    };
    let on_drag_over = Callback::from(|event: DragEvent| event.prevent_default());
    let on_continue = {
        let on_completed = props.on_completed.clone();
        let answer_state = answer_state.clone();
        let leaving = leaving.clone();
        let pending_advance = pending_advance.clone();
        let target = props.word.simplified.clone();
        Callback::from(move |_| {
            if *leaving {
                return;
            }
            leaving.set(true);
            let on_completed = on_completed.clone();
            let result = match answer_state.phase {
                AnswerPhase::Reviewed { correct } => VocabularyRoundResult {
                    correct,
                    attempts: answer_state.attempts,
                    mistakes: answer_state.mistakes,
                    item_evidence: vec![AssessmentEvidence {
                        capability: AssessmentCapability::CharacterSequenceConstruction,
                        target: target.clone(),
                        verdict: if !correct {
                            ExerciseVerdict::Incorrect
                        } else if answer_state.mistakes == 0 {
                            ExerciseVerdict::Correct
                        } else {
                            ExerciseVerdict::PartiallyCorrect
                        },
                        attempts: answer_state.attempts.clamp(1, u16::MAX as u32) as u16,
                        mistakes: answer_state.mistakes.min(u16::MAX as u32) as u16,
                        hints: 0,
                        assisted: false,
                    }],
                },
                AnswerPhase::Editing => return,
            };
            *pending_advance.borrow_mut() =
                Some(gloo::timers::callback::Timeout::new(180, move || {
                    on_completed.emit(result);
                }));
        })
    };

    let slot_class = if *auto_error {
        "flex min-h-20 min-w-20 flex-col items-center justify-center rounded-xl border-2 border-line bg-accent-soft text-ink"
    } else {
        "flex min-h-20 min-w-20 flex-col items-center justify-center rounded-xl border-2 border-dashed border-line bg-raised"
    };
    let element_classes_1 = classes!(
        "practice-round",
        "text-center",
        if *leaving {
            "practice-round-leaving"
        } else {
            ""
        }
    );
    html! {
        <div class={element_classes_1} data-exercise="character-builder" data-auto-check={props.options.automatic_check.to_string()}>
            <ExerciseSurface icon={crate::pages::practice::instruction::InstructionIcon::Arrange} instruction="">
            <h2 class="text-3xl font-bold">{concise_prompt_meaning(&props.word).unwrap_or_default()}</h2>
            <div role="group" aria-label="Selected characters" class="flex flex-wrap items-center justify-center gap-3" ondragover={on_drag_over.clone()} ondrop={on_drop_tile}>
                {for (0..capacity).map(|slot| {
                    let picked = picked.clone();
                    let character = assembled.chars().nth(slot).map(|ch| ch.to_string()).unwrap_or_default();
                    let reading = character.chars().next().map(|ch| reading_for_character(ch, &props.word, &props.words)).unwrap_or_default();
                    let disabled = reviewed || *auto_error;
                    let tile_in_slot = picked.get(slot).copied();
                    let pointer_drag = pointer_drag.clone();
                    let suppress_click_for_remove = suppress_click.clone();
                    let answer_state_for_remove = answer_state.clone();
                    let picked_for_remove = picked.clone();
                    let on_remove = Callback::from(move |_| {
                        if disabled || js_sys::Date::now() < *suppress_click_for_remove.borrow() || slot >= picked_for_remove.len() {
                            return;
                        }
                        answer_state_for_remove.dispatch(AnswerAction::Edit);
                        picked_for_remove.set(remove_tile(&picked_for_remove, slot));
                    });
                    let on_pointer_start = Callback::from(move |event: PointerEvent| {
                        if event.pointer_type() != "mouse" && !disabled {
                            if let Some(tile) = tile_in_slot {
                                *pointer_drag.borrow_mut() = Some((tile, event.client_x(), event.client_y(), true));
                            }
                        }
                    });
                    let dragged = dragged_tile.clone();
                    let on_drag_start = Callback::from(move |event: DragEvent| {
                        if let Some(tile) = tile_in_slot {
                            if let Some(transfer) = event.data_transfer() {
                                let _ = transfer.set_data("text/plain", &tile.to_string());
                                transfer.set_effect_allowed("move");
                            }
                            dragged.set(Some(tile));
                        }
                    });
                    let dragged = dragged_tile.clone();
                    let on_drag_end = Callback::from(move |_: DragEvent| dragged.set(None));
                    let dragged = dragged_tile.clone();
                    let bank_len = bank.len();
                    let auto_error_for_drop = auto_error.clone();
                    let answer_state_for_drop = answer_state.clone();
                    let picked_for_drop = picked.clone();
                    let suppress_click_for_drop = suppress_click.clone();
                    let on_drop_slot = Callback::from(move |event: DragEvent| {
                        event.prevent_default();
                        event.stop_propagation();
                        if !reviewed && !*auto_error_for_drop {
                            if let Some(tile) = *dragged {
                                let next = place_tile(&picked_for_drop, tile, bank_len, capacity, slot);
                                if next != *picked_for_drop {
                                    answer_state_for_drop.dispatch(AnswerAction::Edit);
                                    picked_for_drop.set(next);
                                    *suppress_click_for_drop.borrow_mut() = js_sys::Date::now() + 350.0;
                                }
                            }
                        }
                        dragged.set(None);
                    });
                    html! {
                        <button
                            type="button"
                            onclick={on_remove}
                            draggable={if tile_in_slot.is_some() && !disabled { "true" } else { "false" }}
                            ondragstart={on_drag_start}
                            ondragend={on_drag_end}
                            ondragover={on_drag_over.clone()}
                            ondrop={on_drop_slot}
                            class={classes!(slot_class, "touch-none")}
                            data-character-slot={slot.to_string()}
                            onpointerdown={on_pointer_start}
                            aria-label={if tile_in_slot.is_some() { format!("Remove character from slot {}", slot + 1) } else { format!("Empty character slot {}", slot + 1) }}
                            tabindex={if tile_in_slot.is_some() && !disabled { "0" } else { "-1" }}
                        >
                            <span class="character-builder-hanzi" lang="zh-Hans">{character}</span>
                            <span class="character-builder-reading text-muted" lang="zh-Latn" data-character-reading="">{reading}</span>
                        </button>
                    }
                })}
            </div>
            <div class="border-t border-line pt-4 flex flex-wrap justify-center gap-3" role="group" aria-label="Available characters" data-character-bank="" ondragover={on_drag_over.clone()} ondrop={on_drop_bank}>
                {for bank.iter().enumerate().map(|(tile, character)| {
                    let picked = picked.clone();
                    let disabled = reviewed || *auto_error || picked.contains(&tile) || picked.len() == capacity;
                    let reading = reading_for_character(*character, &props.word, &props.words);
                    let available = bank.len();
                    let answer_state = answer_state.clone();
                    let suppress_click = suppress_click.clone();
                    let on_add = Callback::from(move |_| {
                        if js_sys::Date::now() < *suppress_click.borrow() { return; }
                        answer_state.dispatch(AnswerAction::Edit);
                        picked.set(add_tile(&picked, tile, available, capacity));
                    });
                    let pointer_drag = pointer_drag.clone();
                    let on_pointer_start = Callback::from(move |event: PointerEvent| {
                        if event.pointer_type() != "mouse" && !disabled {
                            *pointer_drag.borrow_mut() = Some((tile, event.client_x(), event.client_y(), false));
                        }
                    });
                    let on_drag_start = {
                        let dragged_tile = dragged_tile.clone();
                        Callback::from(move |event: DragEvent| {
                            if let Some(transfer) = event.data_transfer() {
                                let _ = transfer.set_data("text/plain", &tile.to_string());
                                transfer.set_effect_allowed("move");
                            }
                            dragged_tile.set(Some(tile));
                        })
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
                            onpointerdown={on_pointer_start}
                            class="touch-none min-h-20 min-w-20 flex flex-col items-center justify-center rounded-xl border border-line bg-raised disabled:opacity-30"
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
            <div class="w-full" data-practice-answer-feedback="">
                {if *auto_error {
                    html! { <span role="status" class="text-sm font-semibold text-ink">{"Not quite — try another arrangement"}</span> }
                } else if let AnswerPhase::Reviewed { correct } = answer_state.phase {
                    html! {
                        <ExerciseReview verdict={if correct { FeedbackVerdict::Correct } else { FeedbackVerdict::Incorrect }} on_continue={on_continue} disabled={*leaving} character_builder=true>
                            {if correct { Html::default() } else { html! { <p>{format!("Correct answer: {}", props.word.simplified)}</p> } }}
                        </ExerciseReview>
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
                Html::default()
            } else if !props.options.automatic_check {
                html! { <ExercisePrimaryAction label="Check answer" on_activate={on_check} disabled={picked.len() != capacity} vocabulary=true /> }
            } else { Html::default() }}
            </div>
            </ExerciseSurface>
        </div>
    }
}
