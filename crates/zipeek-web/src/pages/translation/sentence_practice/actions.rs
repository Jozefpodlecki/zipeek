use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_ai_protocol::{
    evaluation_messages_for_direction, generation_messages_for_vocabulary, parse_evaluation,
    parse_sentence_for_vocabulary,
};
use zipeek_core::sentence_tiles::{TileMove, assembled_answer_with_separator};
use zipeek_exercises::{session::ExerciseResult, translation::TranslationDirection};

use crate::services::{ai, translation_instance, translation_vocabulary};

use super::state::{SessionAction, Stage, TranslationController};

pub(super) struct Callbacks {
    pub generate: Callback<MouseEvent>,
    pub evaluate: Callback<MouseEvent>,
    pub reset: Callback<MouseEvent>,
    pub add_tile: Callback<usize>,
    pub remove_tile: Callback<usize>,
    pub reorder_tile: Callback<(usize, usize)>,
}

pub(super) fn callbacks(controller: &TranslationController) -> Callbacks {
    let add = controller.session.clone();
    let remove = controller.session.clone();
    let reorder = controller.session.clone();

    Callbacks {
        generate: generate(controller.clone()),
        evaluate: evaluate(controller.clone()),
        reset: reset(controller.clone()),
        add_tile: Callback::from(move |index| {
            add.dispatch(SessionAction::MoveTile(TileMove::Add(index)))
        }),
        remove_tile: Callback::from(move |index| {
            remove.dispatch(SessionAction::MoveTile(TileMove::Remove(index)))
        }),
        reorder_tile: Callback::from(move |(from, to)| {
            reorder.dispatch(SessionAction::MoveTile(TileMove::Before {
                tile: from,
                target: to,
            }));
        }),
    }
}

fn fail(controller: &TranslationController, message: String) {
    controller.session.dispatch(SessionAction::Fail {
        message,
        retry: None,
    });
}

fn generate(controller: TranslationController) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        let session = &*controller.session;
        if !matches!(&session.stage, Stage::Ready | Stage::Feedback(_, _)) {
            return;
        }
        if controller
            .round_limit
            .is_some_and(|limit| session.completed >= limit)
        {
            controller.session.dispatch(SessionAction::Complete);
            return;
        }
        if !controller.quiz_session.active() {
            fail(
                &controller,
                "This shared session is finished. Start a new session before generating exercises."
                    .to_owned(),
            );
            return;
        }
        let configured = ai::configuration();
        if configured.model.trim().is_empty() {
            fail(
                &controller,
                "Configure your AI model in Settings before starting translation.".to_owned(),
            );
            return;
        }

        controller.abandon_pending();
        let direction = session.direction.choose(
            controller.runtime.borrow().first_direction_is_chinese,
            session.exercise_index,
        );
        let endpoint = configured.url;
        let model = configured.model;
        let source = session.source.clone();
        let instance_scope = session.instance_scope.clone();
        controller.session.dispatch(SessionAction::BeginGeneration);
        let request_id = controller.next_request_id();

        let task = controller.clone();
        spawn_local(async move {
            let result = async {
                let vocabulary =
                    translation_vocabulary::load(&source, &task.dictionary, &task.storage)
                        .await?
                        .sampled(
                            ((js_sys::Math::random() * u32::MAX as f64) as u64) ^ request_id as u64,
                        );
                if !task.request_is_current(request_id) {
                    return Err("Generation cancelled".to_owned());
                }
                let mut characters = 0_usize;
                let mut last_report = 0_usize;
                let raw = ai::complete_with_updates(
                    &endpoint,
                    &model,
                    generation_messages_for_vocabulary(&vocabulary, direction),
                    |update| {
                        let reasoning =
                            matches!(&update, zipeek_ai_client::StreamUpdate::Reasoning { .. });
                        characters = match update {
                            zipeek_ai_client::StreamUpdate::Content(delta) => {
                                characters.saturating_add(delta.chars().count())
                            }
                            zipeek_ai_client::StreamUpdate::Reasoning { received_chars } => {
                                characters.max(received_chars)
                            }
                        };
                        if characters.saturating_sub(last_report) >= 128 {
                            last_report = characters;
                            if task.request_is_current(request_id) {
                                task.session.dispatch(SessionAction::Progress {
                                    received_chars: characters,
                                    reasoning,
                                });
                            }
                        }
                    },
                )
                .await?;
                parse_sentence_for_vocabulary(&raw, &vocabulary, direction)
            }
            .await;

            if !task.request_is_current(request_id) {
                return;
            }
            match result {
                Ok(sentence) => {
                    let order = translation_instance::shuffled_bank_order(&sentence, |limit| {
                        (js_sys::Math::random() * limit as f64) as usize
                    });
                    let instance_id = {
                        let mut runtime = task.runtime.borrow_mut();
                        let id =
                            format!("{instance_scope}:translation-{}", runtime.next_instance_id);
                        runtime.next_instance_id += 1;
                        id
                    };
                    match translation_instance::visible_instance(
                        &sentence,
                        direction,
                        &order,
                        instance_id,
                    ) {
                        Ok(instance) => {
                            task.runtime.borrow_mut().pending_instance = Some(instance);
                            task.session.dispatch(SessionAction::Generated {
                                sentence,
                                bank_order: order,
                            });
                        }
                        Err(error) => fail(
                            &task,
                            format!("Unable to open translation session entry: {error:?}"),
                        ),
                    }
                }
                Err(error) => fail(&task, error),
            }
        });
    })
}

fn evaluate(controller: TranslationController) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        let Stage::Answering(sentence) = &controller.session.stage else {
            return;
        };
        let Some(direction) = TranslationDirection::from_code(&sentence.direction) else {
            return;
        };
        let separator = if direction == TranslationDirection::EnglishToChinese {
            ""
        } else {
            " "
        };
        let Some(answer) = assembled_answer_with_separator(
            &sentence.word_bank,
            &controller.session.selected_tiles,
            separator,
        ) else {
            return;
        };

        let sentence = sentence.clone();
        let configured = ai::configuration();
        let endpoint = configured.url;
        let model = configured.model;
        controller
            .session
            .dispatch(SessionAction::BeginEvaluation(sentence.clone()));
        let request_id = controller.next_request_id();

        let task = controller.clone();
        spawn_local(async move {
            let (source, reference) = match direction {
                TranslationDirection::ChineseToEnglish => {
                    (sentence.chinese.as_str(), sentence.english.as_str())
                }
                TranslationDirection::EnglishToChinese => {
                    (sentence.english.as_str(), sentence.chinese.as_str())
                }
            };
            let mut characters = 0_usize;
            let mut last_report = 0_usize;
            let result = match ai::complete_with_updates(
                &endpoint,
                &model,
                evaluation_messages_for_direction(direction, source, reference, &answer),
                |update| {
                    let reasoning =
                        matches!(&update, zipeek_ai_client::StreamUpdate::Reasoning { .. });
                    characters = match update {
                        zipeek_ai_client::StreamUpdate::Content(delta) => {
                            characters.saturating_add(delta.chars().count())
                        }
                        zipeek_ai_client::StreamUpdate::Reasoning { received_chars } => {
                            characters.max(received_chars)
                        }
                    };
                    if characters.saturating_sub(last_report) >= 128 {
                        last_report = characters;
                        if task.request_is_current(request_id) {
                            task.session.dispatch(SessionAction::Progress {
                                received_chars: characters,
                                reasoning,
                            });
                        }
                    }
                },
            )
            .await
            {
                Ok(raw) => parse_evaluation(&raw),
                Err(error) => Err(error),
            };

            if !task.request_is_current(request_id) {
                return;
            }
            match result {
                Ok(evaluation) => {
                    let instance = task.runtime.borrow_mut().pending_instance.take();
                    if let Some(instance) = instance {
                        let correct = evaluation.accurate;
                        task.quiz_session.submit(
                            instance,
                            ExerciseResult {
                                earned: u64::from(correct),
                                possible: 1,
                                mistakes: u64::from(!correct),
                            },
                        );
                        task.session.dispatch(SessionAction::Evaluated {
                            sentence,
                            evaluation,
                        });
                    } else {
                        task.session.dispatch(SessionAction::Fail {
                            message: "Missing pending translation exercise".to_owned(),
                            retry: Some(sentence),
                        });
                    }
                }
                Err(message) => task.session.dispatch(SessionAction::Fail {
                    message,
                    retry: Some(sentence),
                }),
            }
        });
    })
}

fn reset(controller: TranslationController) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        controller.invalidate_request();
        controller.session.dispatch(SessionAction::Retry);
    })
}
