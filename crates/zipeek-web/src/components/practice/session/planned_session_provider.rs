use std::rc::Rc;
use yew::prelude::*;
use zipeek_exercises::planned_handoff::{HandoffError, PlannedRoundTicket};
use zipeek_exercises::planned_runtime::PlannedRuntime;
use zipeek_exercises::practice_draft::PracticeDraft;
use zipeek_exercises::session::ExerciseResult;

enum PlannedAction {
    Start(PracticeDraft),
    Publish {
        ticket: PlannedRoundTicket,
        visible_input: String,
        result: ExerciseResult,
    },
    Acknowledge {
        generation: u64,
        instance_id: String,
    },
    Cancel {
        generation: u64,
    },
}

#[derive(Clone, PartialEq, Eq, Default)]
struct PlannedUiState {
    runtime: PlannedRuntime,
}

impl Reducible for PlannedUiState {
    type Action = PlannedAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        let runtime = &mut next.runtime;
        let outcome = match action {
            PlannedAction::Start(draft) => runtime.start(&draft).map(|_| ()),
            PlannedAction::Publish {
                ticket,
                visible_input,
                result,
            } => runtime.publish_ticket(&ticket, visible_input, result),
            PlannedAction::Acknowledge {
                generation,
                instance_id,
            } => runtime.acknowledge(generation, &instance_id).map(|_| ()),
            PlannedAction::Cancel { generation } => runtime.cancel(generation),
        };
        if let Err(error) = outcome {
            log::warn!("Planned session transition rejected: {error:?}");
            return self;
        }
        Rc::new(next)
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct PlannedSessionContext {
    state: UseReducerHandle<PlannedUiState>,
}

impl PlannedSessionContext {
    pub(crate) fn runtime(&self) -> &PlannedRuntime {
        &self.state.runtime
    }

    pub(crate) fn current_ticket(&self) -> Result<PlannedRoundTicket, HandoffError> {
        PlannedRoundTicket::from_runtime(self.runtime())
    }

    pub(crate) fn start(&self, draft: PracticeDraft) {
        self.state.dispatch(PlannedAction::Start(draft));
    }

    pub(crate) fn publish(
        &self,
        ticket: PlannedRoundTicket,
        visible_input: String,
        result: ExerciseResult,
    ) {
        self.state.dispatch(PlannedAction::Publish {
            ticket,
            visible_input,
            result,
        });
    }

    pub(crate) fn acknowledge(&self, generation: u64, instance_id: String) {
        self.state.dispatch(PlannedAction::Acknowledge {
            generation,
            instance_id,
        });
    }

    pub(crate) fn cancel(&self, generation: u64) {
        self.state.dispatch(PlannedAction::Cancel { generation });
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct PlannedSessionProviderProps {
    pub children: Children,
}

#[function_component(PlannedSessionProvider)]
pub(crate) fn planned_session_provider(props: &PlannedSessionProviderProps) -> Html {
    let state = use_reducer_eq(PlannedUiState::default);
    html! {
        <ContextProvider<PlannedSessionContext> context={PlannedSessionContext { state }}>
            {props.children.clone()}
        </ContextProvider<PlannedSessionContext>>
    }
}
