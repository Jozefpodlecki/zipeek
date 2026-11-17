use std::{cell::RefCell, rc::Rc};

use yew::prelude::*;
use zipeek_ai_protocol::{Evaluation, Sentence};
use zipeek_core::sentence_tiles::{TileMove, apply_tile_move};
use zipeek_exercises::{session::ExerciseInstance, translation::DirectionSelection};

use crate::{components::QuizSessionContext, contexts::database::Dictionary};

#[derive(Clone, PartialEq)]
pub(super) enum Stage {
    Ready,
    Generating,
    Answering(Sentence),
    Evaluating(Sentence),
    Feedback(Sentence, Evaluation),
    Failed(String, Option<Sentence>),
    Complete,
}

#[derive(Clone, PartialEq)]
pub(super) struct TranslationSession {
    pub source: String,
    pub direction: DirectionSelection,
    pub completed: u32,
    pub exercise_index: u64,
    pub selected_tiles: Vec<usize>,
    pub bank_order: Vec<usize>,
    pub stage: Stage,
    pub progress_chars: usize,
    pub progress_reasoning: bool,
    pub instance_scope: String,
}

#[derive(Clone)]
pub(super) enum SessionAction {
    BeginGeneration,
    Progress {
        received_chars: usize,
        reasoning: bool,
    },
    Generated {
        sentence: Sentence,
        bank_order: Vec<usize>,
    },
    BeginEvaluation(Sentence),
    Evaluated {
        sentence: Sentence,
        evaluation: Evaluation,
    },
    Fail {
        message: String,
        retry: Option<Sentence>,
    },
    Complete,
    Retry,
    MoveTile(TileMove),
}

impl TranslationSession {
    pub fn new(source: String, direction: DirectionSelection, instance_scope: String) -> Self {
        Self {
            source,
            direction,
            completed: 0,
            exercise_index: 0,
            selected_tiles: Vec::new(),
            bank_order: Vec::new(),
            stage: Stage::Ready,
            progress_chars: 0,
            progress_reasoning: false,
            instance_scope,
        }
    }
}

impl Reducible for TranslationSession {
    type Action = SessionAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            SessionAction::BeginGeneration => {
                next.selected_tiles.clear();
                next.bank_order.clear();
                next.stage = Stage::Generating;
                next.progress_chars = 0;
                next.progress_reasoning = false;
            }
            SessionAction::Progress {
                received_chars,
                reasoning,
            } => {
                if matches!(&next.stage, Stage::Generating | Stage::Evaluating(_)) {
                    next.progress_chars = received_chars;
                    next.progress_reasoning = reasoning;
                }
            }
            SessionAction::Generated {
                sentence,
                bank_order,
            } => {
                next.bank_order = bank_order;
                next.exercise_index += 1;
                next.stage = Stage::Answering(sentence);
            }
            SessionAction::BeginEvaluation(sentence) => {
                next.progress_chars = 0;
                next.progress_reasoning = false;
                next.stage = Stage::Evaluating(sentence);
            }
            SessionAction::Evaluated {
                sentence,
                evaluation,
            } => {
                next.completed += 1;
                next.stage = Stage::Feedback(sentence, evaluation);
            }
            SessionAction::Fail { message, retry } => next.stage = Stage::Failed(message, retry),
            SessionAction::Complete => next.stage = Stage::Complete,
            SessionAction::Retry => {
                next.stage = match &next.stage {
                    Stage::Failed(_, Some(sentence)) => Stage::Answering(sentence.clone()),
                    _ => Stage::Ready,
                };
            }
            SessionAction::MoveTile(movement) => {
                if let Stage::Answering(sentence) = &next.stage {
                    next.selected_tiles =
                        apply_tile_move(&next.selected_tiles, sentence.word_bank.len(), movement);
                }
            }
        }
        Rc::new(next)
    }
}

pub(super) struct TranslationRuntime {
    pub first_direction_is_chinese: bool,
    pub next_instance_id: u64,
    pub request_epoch: u64,
    pub pending_instance: Option<ExerciseInstance>,
}

impl TranslationRuntime {
    pub fn new() -> Self {
        Self {
            first_direction_is_chinese: js_sys::Math::random() >= 0.5,
            next_instance_id: 0,
            request_epoch: 0,
            pending_instance: None,
        }
    }
}

#[derive(Clone)]
pub(super) struct TranslationController {
    pub session: UseReducerHandle<TranslationSession>,
    pub runtime: Rc<RefCell<TranslationRuntime>>,
    pub dictionary: Dictionary,
    pub storage: web_sys::Storage,
    pub round_limit: Option<u32>,
    pub quiz_session: QuizSessionContext,
}

impl TranslationController {
    pub fn invalidate_request(&self) {
        self.runtime.borrow_mut().request_epoch += 1;
    }

    pub fn request_is_current(&self, request_id: u64) -> bool {
        self.runtime.borrow().request_epoch == request_id
    }

    pub fn next_request_id(&self) -> u64 {
        let mut runtime = self.runtime.borrow_mut();
        runtime.request_epoch += 1;
        runtime.request_epoch
    }

    pub fn abandon_pending(&self) {
        if let Some(instance) = self.runtime.borrow_mut().pending_instance.take() {
            self.quiz_session.abandon(instance.id);
        }
    }
}
