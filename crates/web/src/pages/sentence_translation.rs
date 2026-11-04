mod actions;
mod settings;
mod state;
mod view;

use std::collections::HashMap;

use yew::prelude::*;
use yew_router::prelude::use_location;
use zipeek_ai::starter_set;
use crate::contexts::database::Dictionary;
use zipeek_exercises::{session::ExerciseInstance, translation::DirectionSelection};

use crate::components::QuizSessionContext;

use state::TranslationState;

#[function_component(SentenceTranslation)]
pub fn sentence_translation() -> Html {
    let location = use_location();
    let initial_set = location
        .as_ref()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .and_then(|query| query.get("source").cloned().or_else(|| query.get("set").cloned().map(|set| format!("starter:{set}"))))
        .filter(|source| source.starts_with("hsk:") || source.starts_with("filters:") || source.starts_with("saved:") || source.strip_prefix("starter:").is_some_and(|id| starter_set(id).is_some()))
        .unwrap_or_else(|| "hsk:3:1".to_owned());

    let round_limit = location.as_ref()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .and_then(|query| query.get("count").and_then(|value| value.parse::<u32>().ok()))
        .filter(|count| matches!(count, 5 | 10));
    let set_id = use_state(move || initial_set);
    let completed = use_state(|| 0_u32);
    let endpoint = use_state(|| "http://127.0.0.1:1234/v1/".to_owned());
    let model = use_state(String::new);
    let direction_selection = use_state(|| DirectionSelection::Mixed);
    let first_direction_is_chinese = use_mut_ref(|| js_sys::Math::random() >= 0.5);
    let exercise_index = use_mut_ref(|| 0_u64);
    let next_instance_id = use_mut_ref(|| 0_u64);
    let selected_tiles = use_state(Vec::<usize>::new);
    let bank_order = use_state(Vec::<usize>::new);
    let stage = use_state(|| state::Stage::Ready);
    let request_epoch = use_mut_ref(|| 0_u64);
    let quiz_session = use_context::<QuizSessionContext>()
        .expect("QuizSessionProvider must wrap translation");
    let pending_instance = use_mut_ref(|| None::<ExerciseInstance>);
    let instance_scope = use_state(|| quiz_session.allocate_scope());

    let dictionary = use_context::<Dictionary>().expect("DictionaryProvider must wrap translation");
    let controller = TranslationState {
        set_id,
        dictionary,
        round_limit,
        completed,
        endpoint,
        model,
        direction_selection,
        first_direction_is_chinese,
        exercise_index,
        next_instance_id,
        selected_tiles,
        bank_order,
        stage,
        request_epoch,
        quiz_session,
        pending_instance,
        instance_scope,
    };

    {
        let controller = controller.clone();
        use_effect_with((), move |_| {
            move || {
                controller.invalidate_request();
                controller.abandon_pending();
            }
        });
    }

    let callbacks = actions::callbacks(&controller);
    view::render(&controller, callbacks)
}
