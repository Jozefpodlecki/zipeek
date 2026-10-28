use crate::{
    contexts::database::{DatabaseCommands, DatabaseStatus, Dictionary},
    reducers::database::{DatabaseAction, DatabaseState},
    services::database::Database,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use yew::prelude::*;

#[derive(Default)]
struct RequestGate {
    generation: Cell<u64>,
    cancelled: Cell<bool>,
}

impl RequestGate {
    fn retry(&self) -> u64 {
        let generation = self.generation.get().wrapping_add(1);
        self.generation.set(generation);
        self.cancelled.set(false);
        generation
    }

    fn accepts(&self, generation: u64) -> bool {
        !self.cancelled.get() && self.generation.get() == generation
    }

    fn cancel(&self, generation: u64) -> bool {
        let current = self.accepts(generation);
        if current {
            self.cancelled.set(true);
        }
        current
    }
}

#[derive(Default)]
struct DatabaseOwner {
    gate: RequestGate,
    slot: Rc<RefCell<Option<Database>>>,
}

impl DatabaseOwner {
    fn retry(&self) -> u64 {
        let request = self.gate.retry();
        self.slot.borrow_mut().take();
        request
    }

    fn cancel(&self, request: u64) {
        if self.gate.cancel(request) {
            self.slot.borrow_mut().take();
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DatabaseProviderProps {
    pub children: Children,
}

#[function_component(DatabaseProvider)]
pub fn database_provider(props: &DatabaseProviderProps) -> Html {
    let state = use_reducer_eq(DatabaseState::default);
    let owner = use_mut_ref(DatabaseOwner::default);
    let retry = {
        let owner = owner.clone();
        use_callback(state.dispatcher(), move |(), dispatch| {
            let request = owner.borrow().retry();
            dispatch.dispatch(DatabaseAction::Retry(request));
        })
    };
    {
        let owner = owner.clone();
        let dispatch = state.dispatcher();
        use_effect_with(state.request, move |request| {
            let request = *request;
            let pending = owner.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = Database::open().await;
                if !pending.borrow().gate.accepts(request) {
                    return;
                }
                match result {
                    Ok(database) => {
                        let metadata = database.info.clone();
                        *pending.borrow().slot.borrow_mut() = Some(database);
                        dispatch.dispatch(DatabaseAction::Opened(request, metadata));
                    }
                    Err(error) => {
                        log::error!("{error}");
                        dispatch.dispatch(DatabaseAction::Failed(request, error.to_string()));
                    }
                }
            });
            move || owner.borrow().cancel(request)
        });
    }
    let dictionary = Dictionary::new(owner.borrow().slot.clone());
    html! {
        <ContextProvider<Dictionary> context={dictionary}>
            <ContextProvider<DatabaseCommands> context={DatabaseCommands::new(retry)}>
                <ContextProvider<DatabaseStatus> context={state.status.clone()}>
                    {props.children.clone()}
                </ContextProvider<DatabaseStatus>>
            </ContextProvider<DatabaseCommands>>
        </ContextProvider<Dictionary>>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_invalidates_pending_result_before_effect_cleanup() {
        let gate = RequestGate::default();
        assert!(gate.accepts(0));
        assert_eq!(gate.retry(), 1);
        assert!(!gate.accepts(0));
        assert!(!gate.cancel(0));
        assert!(gate.accepts(1));
        assert_eq!(gate.retry(), 2);
        assert!(!gate.accepts(1));
        assert!(gate.accepts(2));
    }

    #[test]
    fn unmount_rejects_pending_initialization_and_client_access() {
        let owner = DatabaseOwner::default();
        let dictionary = Dictionary::new(owner.slot.clone());
        assert!(dictionary.database().is_err());
        owner.cancel(0);
        assert!(!owner.gate.accepts(0));
        assert!(dictionary.database().is_err());
    }
}
