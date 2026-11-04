use std::{cell::RefCell, rc::Rc};

use yew::prelude::*;
use zipeek_ai::{Evaluation, Sentence};
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

#[derive(Clone)]
pub(super) struct TranslationState {
    pub set_id: UseStateHandle<String>,
    pub dictionary: Dictionary,
    pub round_limit: Option<u32>,
    pub completed: UseStateHandle<u32>,
    pub endpoint: UseStateHandle<String>,
    pub model: UseStateHandle<String>,
    pub direction_selection: UseStateHandle<DirectionSelection>,
    pub first_direction_is_chinese: Rc<RefCell<bool>>,
    pub exercise_index: Rc<RefCell<u64>>,
    pub next_instance_id: Rc<RefCell<u64>>,
    pub selected_tiles: UseStateHandle<Vec<usize>>,
    pub bank_order: UseStateHandle<Vec<usize>>,
    pub stage: UseStateHandle<Stage>,
    pub request_epoch: Rc<RefCell<u64>>,
    pub quiz_session: QuizSessionContext,
    pub pending_instance: Rc<RefCell<Option<ExerciseInstance>>>,
    pub instance_scope: UseStateHandle<String>,
}

impl TranslationState {
    pub fn invalidate_request(&self) {
        *self.request_epoch.borrow_mut() += 1;
    }

    pub fn abandon_pending(&self) {
        if let Some(instance) = self.pending_instance.borrow_mut().take() {
            self.quiz_session.abandon(instance.id);
        }
    }

    pub fn reset_configuration(&self) {
        self.invalidate_request();
        self.abandon_pending();
        *self.exercise_index.borrow_mut() = 0;
        self.completed.set(0);
        self.selected_tiles.set(Vec::new());
        self.bank_order.set(Vec::new());
        self.stage.set(Stage::Ready);
    }

    pub fn next_request_id(&self) -> u64 {
        let mut epoch = self.request_epoch.borrow_mut();
        *epoch += 1;
        *epoch
    }
}
