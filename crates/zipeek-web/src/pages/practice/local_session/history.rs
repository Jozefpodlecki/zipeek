use std::{cell::Cell, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, Storage};
use yew::prelude::*;

use super::state::SessionState;

use crate::services::practice_history::{self, CompletionDisposition};

#[hook]
pub(super) fn use_history_finalization(
    session: UseReducerHandle<SessionState>,
    retry: u32,
    storage: Storage,
    document: Document,
) -> UseStateHandle<Option<Result<CompletionDisposition, String>>> {
    let result = use_state(|| None::<Result<CompletionDisposition, String>>);
    {
        let result = result.clone();
        use_effect_with((session.snapshot.clone(), retry), move |(snapshot, _)| {
            let active = Rc::new(Cell::new(true));
            if let Some(Ok(saved)) = snapshot {
                if saved.is_completed() {
                    let saved = saved.clone();
                    let result = result.clone();
                    let active_for_result = active.clone();
                    result.set(None);
                    spawn_local(async move {
                        let outcome =
                            practice_history::finalize_completed(storage, document, saved).await;
                        if active_for_result.get() {
                            result.set(Some(outcome));
                        }
                    });
                }
            }
            move || active.set(false)
        });
    }
    result
}
