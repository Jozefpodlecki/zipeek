use crate::{
    components::Loader,
    contexts::database::{DatabaseContext, DatabaseState},
    services::database::Database,
};
use std::{cell::Cell, rc::Rc};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DatabaseProviderProps {
    pub children: Children,
}

#[function_component(DatabaseProvider)]
pub fn database_provider(props: &DatabaseProviderProps) -> Html {
    let state = use_state(|| DatabaseState::Loading);
    let attempt = use_state(|| 0u64);
    let retry = {
        let state = state.clone();
        let attempt = attempt.clone();
        Callback::from(move |_| {
            state.set(DatabaseState::Loading);
            attempt.set(attempt.wrapping_add(1));
        })
    };
    {
        let state = state.clone();
        use_effect_with(*attempt, move |_| {
            let cancelled = Rc::new(Cell::new(false));
            let pending = cancelled.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let next = match Database::open().await {
                    Ok(database) => DatabaseState::Ready(database),
                    Err(error) => {
                        log::error!("{error}");
                        DatabaseState::Failed(error.to_string())
                    }
                };
                if !pending.get() {
                    state.set(next);
                }
            });
            move || cancelled.set(true)
        });
    }
    let content = match &*state {
        DatabaseState::Loading => html! { <div data-database-state="loading"><Loader /></div> },
        DatabaseState::Ready(database) => {
            html! { <div data-database-state="ready" data-database-rows={database.info.row_count.to_string()} data-database-engine={database.info.engine.clone()}>{for props.children.iter()}</div> }
        }
        DatabaseState::Failed(_) => {
            let retry = retry.clone();
            html! { <section class="route-placeholder" data-database-state="failed"><h1>{"Dictionary unavailable"}</h1><p>{"The dictionary could not be loaded. Please try again."}</p><button type="button" onclick={Callback::from(move |_| retry.emit(()))} data-database-retry="">{"Retry"}</button></section> }
        }
    };
    html! { <ContextProvider<DatabaseContext> context={DatabaseContext { state, retry }}>{content}</ContextProvider<DatabaseContext>> }
}
