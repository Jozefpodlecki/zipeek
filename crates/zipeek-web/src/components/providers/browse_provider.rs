use futures::future::{AbortHandle, Abortable};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::use_location;

use crate::{
    hooks::use_dictionary,
    reducers::browse::{BrowseAction, BrowseState},
};

#[derive(Properties, PartialEq)]
pub(crate) struct BrowseProviderProps {
    pub children: Children,
}

#[function_component(BrowseProvider)]
pub(crate) fn browse_provider(props: &BrowseProviderProps) -> Html {
    let initial_query = use_location()
        .and_then(|location| {
            location
                .query::<std::collections::HashMap<String, String>>()
                .ok()
        })
        .and_then(|query| query.get("query").cloned())
        .unwrap_or_default();
    let state = use_reducer_eq(move || BrowseState::from_search_query(&initial_query));
    let dictionary = use_dictionary();

    {
        let dispatch = state.dispatcher();
        let request = state.request.clone();
        let generation = state.generation;
        let blocked = state.blocked.clone();
        use_effect_with(
            (request, generation, dictionary, blocked),
            move |(request, generation, dictionary, blocked)| {
                let (abort, registration) = AbortHandle::new_pair();
                if blocked.is_none() {
                    let request = request.clone();
                    let dictionary = dictionary.clone();
                    let dispatch = dispatch.clone();
                    let generation = *generation;
                    spawn_local(async move {
                        let future = async {
                            let result = match dictionary.database() {
                                Ok(database) => database
                                    .browse_search(&request)
                                    .await
                                    .map_err(|error| error.to_string()),
                                Err(error) => Err(error.to_string()),
                            };
                            dispatch.dispatch(BrowseAction::Completed(generation, result));
                        };
                        let _ = Abortable::new(future, registration).await;
                    });
                }
                move || abort.abort()
            },
        );
    }

    html! {
        <ContextProvider<UseReducerHandle<BrowseState>> context={state}>
            {props.children.clone()}
        </ContextProvider<UseReducerHandle<BrowseState>>>
    }
}
