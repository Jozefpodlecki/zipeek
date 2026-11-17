use std::{cell::Cell, rc::Rc};
use yew::prelude::*;

use zipeek_exercises::session::{ExerciseInstance, ExerciseResult, QuizSession, SessionStatus};

#[derive(Clone, PartialEq, Eq)]
struct SharedQuizState {
    session: QuizSession,
    generation: u64,
}

impl Default for SharedQuizState {
    fn default() -> Self {
        Self {
            session: QuizSession::new("quiz-0"),
            generation: 0,
        }
    }
}

enum SharedQuizAction {
    Submit(u64, ExerciseInstance, ExerciseResult),
    Abandon(u64, String),
    Finish,
    Restart,
}

impl Reducible for SharedQuizState {
    type Action = SharedQuizAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            SharedQuizAction::Submit(generation, instance, result) => {
                if generation != self.generation {
                    return self;
                }
                if let Err(error) = next.session.record_completed(instance, result) {
                    log::error!("Unable to publish quiz result: {error:?}");
                    return self;
                }
            }
            SharedQuizAction::Abandon(generation, instance_id) => {
                if generation != self.generation {
                    return self;
                }
                if let Err(error) = next.session.abandon(&instance_id) {
                    log::debug!("Unable to abandon quiz exercise: {error:?}");
                    return self;
                }
            }
            SharedQuizAction::Finish => {
                if let Err(error) = next.session.complete() {
                    log::error!("Unable to finish quiz session: {error:?}");
                    return self;
                }
            }
            SharedQuizAction::Restart => {
                next.generation = next.generation.wrapping_add(1);
                next.session = QuizSession::new(format!("quiz-{}", next.generation));
            }
        }
        Rc::new(next)
    }
}

#[derive(Clone)]
pub(crate) struct QuizSessionContext {
    state: UseReducerHandle<SharedQuizState>,
    next_scope: Rc<Cell<u64>>,
}

impl PartialEq for QuizSessionContext {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl QuizSessionContext {
    pub(crate) fn session(&self) -> &QuizSession {
        &self.state.session
    }

    pub(crate) fn active(&self) -> bool {
        self.state.session.status == SessionStatus::Active
    }

    pub(crate) fn allocate_scope(&self) -> String {
        let sequence = self.next_scope.get();
        self.next_scope.set(sequence.wrapping_add(1));
        format!("quiz-{}-source-{sequence}", self.state.generation)
    }

    pub(crate) fn abandon(&self, instance_id: String) {
        self.state.dispatch(SharedQuizAction::Abandon(
            self.state.generation,
            instance_id,
        ));
    }

    pub(crate) fn submit(&self, instance: ExerciseInstance, result: ExerciseResult) {
        self.state.dispatch(SharedQuizAction::Submit(
            self.state.generation,
            instance,
            result,
        ));
    }

    pub(crate) fn finish(&self) {
        self.state.dispatch(SharedQuizAction::Finish);
    }

    pub(crate) fn restart(&self) {
        self.state.dispatch(SharedQuizAction::Restart);
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct QuizSessionProviderProps {
    pub children: Children,
}

#[function_component(QuizSessionProvider)]
pub(crate) fn quiz_session_provider(props: &QuizSessionProviderProps) -> Html {
    let state = use_reducer_eq(SharedQuizState::default);
    let next_scope = use_mut_ref(|| Rc::new(Cell::new(0_u64)));
    html! {
        <ContextProvider<QuizSessionContext> context={QuizSessionContext { state, next_scope: next_scope.borrow().clone() }}>
            {props.children.clone()}
        </ContextProvider<QuizSessionContext>>
    }
}
