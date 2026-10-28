use crate::{hooks::use_dictionary, services::exclusions};
use futures::future::{AbortHandle, Abortable};
use std::collections::BTreeSet;
use yew::prelude::*;
use zipeek_core::WordId;

#[derive(Clone, PartialEq)]
pub(crate) enum SavedExclusions {
    Loading,
    Ready(BTreeSet<WordId>),
    Failed,
}

pub(crate) struct ExclusionControls {
    pub status: SavedExclusions,
    pub retry: Callback<()>,
    pub reset: Callback<()>,
    pub reset_failed: bool,
}

#[hook]
pub(crate) fn use_saved_exclusions() -> ExclusionControls {
    let dictionary = use_dictionary();
    let status = use_state_eq(|| SavedExclusions::Loading);
    let attempt = use_state(|| 0u64);
    let generation = use_mut_ref(|| 0u64);
    let reset_failed = use_state(|| false);
    let retry = {
        let generation = generation.clone();
        let status = status.clone();
        use_callback(attempt.clone(), move |(), attempt| {
            let next = generation.borrow().wrapping_add(1);
            *generation.borrow_mut() = next;
            status.set(SavedExclusions::Loading);
            attempt.set(next);
        })
    };
    let reset = {
        let generation = generation.clone();
        let status = status.clone();
        let reset_failed = reset_failed.clone();
        use_callback((), move |(), _| match exclusions::reset_saved() {
            Ok(()) => {
                let next = generation.borrow().wrapping_add(1);
                *generation.borrow_mut() = next;
                reset_failed.set(false);
                status.set(SavedExclusions::Ready(BTreeSet::new()));
            }
            Err(error) => {
                log::warn!("{error}");
                reset_failed.set(true);
            }
        })
    };
    {
        let generation = generation.clone();
        let status = status.clone();
        use_effect_with((dictionary, *attempt), move |(dictionary, request)| {
            let request = *request;
            let (abort, registration) = AbortHandle::new_pair();
            let database = dictionary
                .as_ref()
                .ok_or_else(|| "Setup requires DatabaseProvider".to_owned())
                .and_then(|reader| reader.database().map_err(|error| error.to_string()));
            let ids = exclusions::read_saved(exclusions::storage().as_ref());
            let load = async move {
                let result = match database {
                    Ok(database) => database
                        .words_by_ids(&ids)
                        .await
                        .map(|words| words.into_iter().map(|word| word.id).collect())
                        .map_err(|error| error.to_string()),
                    Err(error) => Err(error),
                };
                if *generation.borrow() == request {
                    match result {
                        Ok(ids) => status.set(SavedExclusions::Ready(ids)),
                        Err(error) => {
                            log::error!("Excluded-word validation failed: {error}");
                            status.set(SavedExclusions::Failed);
                        }
                    }
                }
            };
            wasm_bindgen_futures::spawn_local(async move {
                let _ = Abortable::new(load, registration).await;
            });
            move || abort.abort()
        });
    }
    ExclusionControls {
        status: (*status).clone(),
        retry,
        reset,
        reset_failed: *reset_failed,
    }
}
