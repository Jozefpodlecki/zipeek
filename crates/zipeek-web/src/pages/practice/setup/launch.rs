use std::{cell::Cell, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::Navigator;
use zipeek_browser::local_speech;
use zipeek_exercises::practice_draft::PracticeDraft;

use crate::{
    models::PracticeExerciseOptions,
    reducers::practice_setup_flow::{PracticeSetupAction, PracticeSetupFlow},
    route::Route,
    services::{practice_launch, word_sets::UserWordSet},
};

pub(super) fn start(
    navigator: Navigator,
    draft: PracticeDraft,
    personal_sets: Result<Vec<UserWordSet>, String>,
    storage: web_sys::Storage,
    exercise_options: PracticeExerciseOptions,
    can_start: bool,
    needs_tts: bool,
    flow: UseReducerHandle<PracticeSetupFlow>,
) -> Callback<MouseEvent> {
    let pending = Rc::new(Cell::new(false));
    Callback::from(move |_: MouseEvent| {
        if !can_start || pending.replace(true) {
            return;
        }
        let navigator = navigator.clone();
        let draft = draft.clone();
        let personal_sets = personal_sets.clone();
        let storage = storage.clone();
        let flow = flow.clone();
        let pending = pending.clone();
        spawn_local(async move {
            let speech_ready = if needs_tts {
                match local_speech::service_capabilities().await {
                    Ok(capabilities) if capabilities.synthesis => true,
                    Ok(_) => {
                        log::error!("Local speech server has no available TTS engine");
                        false
                    }
                    Err(error) => {
                        log::error!("Local speech TTS preflight failed: {error:?}");
                        false
                    }
                }
            } else {
                true
            };
            if speech_ready {
                match practice_launch::create_session(
                    &draft,
                    &personal_sets,
                    &storage,
                    exercise_options,
                ) {
                    Ok(id) => {
                        navigator.push(&Route::LocalPracticeSession { id });
                        flow.dispatch(PracticeSetupAction::StartFailed(false));
                    }
                    Err(error) => {
                        log::error!("Could not create Practice session: {error}");
                        flow.dispatch(PracticeSetupAction::StartFailed(true));
                    }
                }
            } else {
                flow.dispatch(PracticeSetupAction::StartFailed(true));
            }
            pending.set(false);
        });
    })
}
