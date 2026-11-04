use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_ai::{
    evaluation_messages_for_direction, generation_messages_for_vocabulary, parse_evaluation,
    parse_sentence_for_vocabulary,
};
use zipeek_core::sentence_tiles::{TileMove, apply_tile_move, assembled_answer_with_separator};
use zipeek_exercises::{session::ExerciseResult, translation::TranslationDirection};

use crate::services::{ai, translation_instance, translation_vocabulary};

use super::state::{Stage, TranslationState};

pub(super) struct Callbacks {
    pub generate: Callback<MouseEvent>,
    pub evaluate: Callback<MouseEvent>,
    pub reset: Callback<MouseEvent>,
    pub add_tile: Callback<usize>,
    pub remove_tile: Callback<usize>,
    pub reorder_tile: Callback<(usize, usize)>,
}

pub(super) fn callbacks(state: &TranslationState) -> Callbacks {
    Callbacks {
        generate: generate(state.clone()),
        evaluate: evaluate(state.clone()),
        reset: reset(state.clone()),
        add_tile: move_tile(state.clone(), TileMoveKind::Add),
        remove_tile: move_tile(state.clone(), TileMoveKind::Remove),
        reorder_tile: reorder_tile(state.clone()),
    }
}

#[derive(Clone, Copy)]
enum TileMoveKind {
    Add,
    Remove,
}

fn move_tile(state: TranslationState, kind: TileMoveKind) -> Callback<usize> {
    Callback::from(move |index| {
        if let Stage::Answering(sentence) = &*state.stage {
            if index >= sentence.word_bank.len() {
                return;
            }
            if matches!(kind, TileMoveKind::Add) && state.selected_tiles.contains(&index) {
                return;
            }
            let movement = match kind {
                TileMoveKind::Add => TileMove::Add(index),
                TileMoveKind::Remove => TileMove::Remove(index),
            };
            state.selected_tiles.set(apply_tile_move(
                &state.selected_tiles,
                sentence.word_bank.len(),
                movement,
            ));
        }
    })
}

fn reorder_tile(state: TranslationState) -> Callback<(usize, usize)> {
    Callback::from(move |(from, to)| {
        if let Stage::Answering(sentence) = &*state.stage {
            state.selected_tiles.set(apply_tile_move(
                &state.selected_tiles,
                sentence.word_bank.len(),
                TileMove::Before { tile: from, target: to },
            ));
        }
    })
}

fn generate(state: TranslationState) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        if state.round_limit.is_some_and(|limit| *state.completed >= limit) {
            state.stage.set(Stage::Complete);
            return;
        }
        if !state.quiz_session.active() {
            state.stage.set(Stage::Failed(
                "This shared session is finished. Start a new session before generating exercises.".to_owned(),
                None,
            ));
            return;
        }
        if state.model.trim().is_empty() {
            state.stage.set(Stage::Failed("Enter a model identifier from LM Studio".to_owned(), None));
            return;
        }

        state.abandon_pending();
        let direction = state.direction_selection.choose(
            *state.first_direction_is_chinese.borrow(),
            *state.exercise_index.borrow(),
        );
        let endpoint = (*state.endpoint).clone();
        let model = (*state.model).clone();
        let source = (*state.set_id).clone();
        state.selected_tiles.set(Vec::new());
        state.bank_order.set(Vec::new());
        state.stage.set(Stage::Generating);
        let request_id = state.next_request_id();

        let task_state = state.clone();
        spawn_local(async move {
            let response = async {
                let vocabulary = translation_vocabulary::load(&source, &task_state.dictionary).await?;
                if *task_state.request_epoch.borrow() != request_id {
                    return Err("Generation cancelled".to_owned());
                }
                let raw = ai::complete(
                    &endpoint,
                    &model,
                    generation_messages_for_vocabulary(&vocabulary, direction),
                ).await?;
                parse_sentence_for_vocabulary(&raw, &vocabulary, direction)
            }.await;
            if *task_state.request_epoch.borrow() != request_id {
                return;
            }
            let result = match response {
                Ok(sentence) => (|| {
                    let order = translation_instance::shuffled_bank_order(&sentence, |limit| {
                        (js_sys::Math::random() * limit as f64) as usize
                    });
                    let instance_id = format!(
                        "{}:translation-{}",
                        *task_state.instance_scope,
                        *task_state.next_instance_id.borrow(),
                    );
                    let instance = translation_instance::visible_instance(
                        &sentence,
                        direction,
                        &order,
                        instance_id,
                    )
                    .map_err(|error| format!("Unable to open translation session entry: {error:?}"))?;
                    *task_state.next_instance_id.borrow_mut() += 1;
                    *task_state.pending_instance.borrow_mut() = Some(instance);
                    task_state.bank_order.set(order);
                    *task_state.exercise_index.borrow_mut() += 1;
                    Ok(Stage::Answering(sentence))
                })(),
                Err(error) => Err(error),
            };
            if *task_state.request_epoch.borrow() == request_id {
                task_state.stage.set(result.unwrap_or_else(|error| Stage::Failed(error, None)));
            }
        });
    })
}

fn evaluate(state: TranslationState) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        let Stage::Answering(sentence) = &*state.stage else {
            return;
        };
        let Some(direction) = TranslationDirection::from_code(&sentence.direction) else {
            return;
        };
        let separator = if direction == TranslationDirection::EnglishToChinese { "" } else { " " };
        let Some(submitted_answer) = assembled_answer_with_separator(
            &sentence.word_bank,
            &state.selected_tiles,
            separator,
        ) else {
            return;
        };

        let sentence = sentence.clone();
        let endpoint = (*state.endpoint).clone();
        let model = (*state.model).clone();
        state.stage.set(Stage::Evaluating(sentence.clone()));
        let request_id = state.next_request_id();

        let task_state = state.clone();
        spawn_local(async move {
            let sentence_for_retry = sentence.clone();
            let (source, reference) = match direction {
                TranslationDirection::ChineseToEnglish => (sentence.chinese.as_str(), sentence.english.as_str()),
                TranslationDirection::EnglishToChinese => (sentence.english.as_str(), sentence.chinese.as_str()),
            };
            let result = match ai::complete(
                &endpoint,
                &model,
                evaluation_messages_for_direction(direction, source, reference, &submitted_answer),
            ).await {
                Ok(raw) => parse_evaluation(&raw).map(|evaluation| Stage::Feedback(sentence, evaluation)),
                Err(error) => Err(error),
            };
            if *task_state.request_epoch.borrow() != request_id {
                return;
            }
            let next_stage = match result {
                Ok(Stage::Feedback(sentence, evaluation)) => {
                    let correct = evaluation.accurate;
                    match task_state.pending_instance.borrow_mut().take() {
                        Some(instance) => {
                            task_state.quiz_session.submit(instance, ExerciseResult {
                                earned: u64::from(correct),
                                possible: 1,
                                mistakes: u64::from(!correct),
                            });
                            task_state.completed.set(*task_state.completed + 1);
                            Stage::Feedback(sentence, evaluation)
                        }
                        None => Stage::Failed(
                            "Missing pending translation exercise".to_owned(),
                            Some(sentence_for_retry),
                        ),
                    }
                }
                Ok(other) => other,
                Err(error) => Stage::Failed(error, Some(sentence_for_retry)),
            };
            task_state.stage.set(next_stage);
        });
    })
}

fn reset(state: TranslationState) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        state.invalidate_request();
        if let Stage::Failed(_, Some(sentence)) = &*state.stage {
            state.stage.set(Stage::Answering(sentence.clone()));
        } else {
            state.stage.set(Stage::Ready);
        }
    })
}
