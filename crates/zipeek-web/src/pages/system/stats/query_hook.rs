use std::{cell::Cell, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use super::query::{QueryFuture, QueryLoad, QueryPage, QueryRequest};

#[hook]
pub(super) fn use_query_page<Q, T>(
    query: Q,
    request: Rc<dyn Fn(Q) -> QueryFuture<T>>,
) -> (Option<QueryPage<T>>, bool, Option<String>, Callback<()>)
where
    Q: Clone + PartialEq + QueryRequest + 'static,
    T: Clone + PartialEq + 'static,
{
    let loaded = use_state(|| None::<(Q, QueryLoad<T>)>);
    let retry = use_state(|| 0_u64);
    let generation = use_mut_ref(|| 0_u64);
    {
        let loaded = loaded.clone();
        let generation = generation.clone();
        use_effect_with((query.clone(), *retry), move |(query, _)| {
            let active = Rc::new(Cell::new(true));
            let request_generation = {
                let mut generation = generation.borrow_mut();
                *generation = generation.wrapping_add(1);
                *generation
            };
            loaded.set(Some((
                query.clone(),
                QueryLoad::Loading {
                    generation: request_generation,
                },
            )));
            let future = request(query.clone());
            let query = query.clone();
            let loaded = loaded.clone();
            let active_for_task = active.clone();
            spawn_local(async move {
                let result = future.await.and_then(|page| {
                    if let Some(expected) = query.page_request() {
                        page.validate(expected)?;
                    }
                    Ok(page)
                });
                if active_for_task.get() {
                    let mut state = QueryLoad::Loading {
                        generation: request_generation,
                    };
                    state.complete(request_generation, result);
                    loaded.set(Some((query, state)));
                }
            });
            move || active.set(false)
        });
    }
    let retry_request = {
        let retry = retry.clone();
        Callback::from(move |_: ()| retry.set((*retry).wrapping_add(1)))
    };
    let current = (*loaded)
        .as_ref()
        .filter(|(loaded_query, _)| loaded_query == &query);
    match current.map(|(_, result)| result) {
        Some(QueryLoad::Ready { page, .. }) => (Some(page.clone()), false, None, retry_request),
        Some(QueryLoad::Failed { error, .. }) => {
            (None, false, Some(error.to_string()), retry_request)
        }
        _ => (None, true, None, retry_request),
    }
}
