use crate::{
    hooks::use_dictionary,
    reducers::practice::{PracticeAction, PracticeRequest, PracticeState, PracticeStatus},
    services::practice_loader::prepare_practice,
};
use futures::future::{AbortHandle, Abortable};
use std::rc::Rc;
use yew::prelude::*;
use zipeek_core::practice::{ExerciseVariant, ParsedPracticeQuery};

#[hook]
pub(crate) fn use_prepared_practice(
    variant: ExerciseVariant,
    query: ParsedPracticeQuery,
) -> (PracticeStatus, Callback<()>) {
    let dictionary = use_dictionary();
    let attempt = use_state(|| 0u64);
    let generation = use_mut_ref(|| 0u64);
    let request = PracticeRequest {
        variant,
        query,
        attempt: *attempt,
    };
    let active = use_mut_ref(|| Some(request.clone()));
    *active.borrow_mut() = Some(request.clone());
    let state = {
        let request = request.clone();
        use_reducer_eq(move || PracticeState {
            request,
            status: PracticeStatus::Loading,
        })
    };
    let retry = {
        let active = active.clone();
        use_callback((attempt, generation), move |(), (attempt, generation)| {
            active.borrow_mut().take();
            let next = generation.borrow().wrapping_add(1);
            *generation.borrow_mut() = next;
            attempt.set(next);
        })
    };
    {
        let active = active.clone();
        let dispatch = state.dispatcher();
        use_effect_with(
            (request.clone(), dictionary),
            move |(request, dictionary)| {
                let request = request.clone();
                dispatch.dispatch(PracticeAction::Begin(request.clone()));
                let (abort, registration) = AbortHandle::new_pair();
                let pending = active.clone();
                let current = request.clone();
                let database = dictionary
                    .as_ref()
                    .ok_or_else(|| "Practice requires DatabaseProvider".to_owned())
                    .and_then(|reader| reader.database().map_err(|error| error.to_string()));
                let load = async move {
                    let status = match database {
                        Ok(database) => {
                            match prepare_practice(&database, current.variant, &current.query).await
                            {
                                Ok(prepared) => PracticeStatus::Ready(Rc::new(prepared)),
                                Err(error) => {
                                    log::error!("Practice loading failed: {error}");
                                    PracticeStatus::Failed(error.to_string())
                                }
                            }
                        }
                        Err(error) => {
                            log::error!("{error}");
                            PracticeStatus::Failed(error)
                        }
                    };
                    if pending.borrow().as_ref() == Some(&current) {
                        dispatch.dispatch(PracticeAction::Finished(current, status));
                    }
                };
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Abortable::new(load, registration).await;
                });
                move || {
                    abort.abort();
                    if active.borrow().as_ref() == Some(&request) {
                        active.borrow_mut().take();
                    }
                }
            },
        );
    }
    let status = if state.request == request {
        state.status.clone()
    } else {
        PracticeStatus::Loading
    };
    (status, retry)
}
