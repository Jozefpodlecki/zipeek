use yew::prelude::*;
use yew_router::prelude::Navigator;
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
    flow: UseReducerHandle<PracticeSetupFlow>,
) -> Callback<MouseEvent> {
    Callback::from(move |_: MouseEvent| {
        if !can_start {
            return;
        }
        match practice_launch::create_session(&draft, &personal_sets, &storage, exercise_options) {
            Ok(id) => {
                navigator.push(&Route::LocalPracticeSession { id });
                flow.dispatch(PracticeSetupAction::StartFailed(false));
            }
            Err(error) => {
                log::error!("Could not create Practice session: {error}");
                flow.dispatch(PracticeSetupAction::StartFailed(true));
            }
        }
    })
}
