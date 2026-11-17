mod actions;
mod state;
mod view;

use std::collections::HashMap;

use crate::hooks::use_dictionary;
use yew::prelude::*;
use yew_router::prelude::use_location;
use zipeek_ai_protocol::starter_set;
use zipeek_exercises::translation::DirectionSelection;

use crate::{components::QuizSessionContext, hooks::use_app_environment};

use state::{TranslationController, TranslationRuntime, TranslationSession};

fn source_from_query(query: &HashMap<String, String>) -> String {
    query
        .get("source")
        .cloned()
        .or_else(|| query.get("set").map(|set| format!("starter:{set}")))
        .filter(|source| {
            source.starts_with("hsk:")
                || source.starts_with("filters:")
                || source.starts_with("saved:")
                || source
                    .strip_prefix("starter:")
                    .is_some_and(|id| starter_set(id).is_some())
        })
        .unwrap_or_else(|| "hsk:3:1".to_owned())
}

fn direction_from_query(query: &HashMap<String, String>) -> DirectionSelection {
    match query.get("direction").map(String::as_str) {
        Some("zh-to-en") => DirectionSelection::ChineseToEnglish,
        Some("en-to-zh") => DirectionSelection::EnglishToChinese,
        _ => DirectionSelection::Mixed,
    }
}

#[function_component(SentencePracticePage)]
pub fn sentence_translation() -> Html {
    let location = use_location();
    let query = location
        .as_ref()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .unwrap_or_default();
    let initial_source = source_from_query(&query);
    let round_limit = query
        .get("count")
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|count| matches!(count, 5 | 10));

    let direction = direction_from_query(&query);

    let environment = use_app_environment();
    let quiz_session =
        use_context::<QuizSessionContext>().expect("QuizSessionProvider must wrap translation");
    let dictionary = use_dictionary();

    let session = {
        let quiz_session = quiz_session.clone();
        use_reducer(move || {
            TranslationSession::new(initial_source, direction, quiz_session.allocate_scope())
        })
    };
    let runtime = use_mut_ref(TranslationRuntime::new);
    let controller = TranslationController {
        session,
        runtime,
        dictionary,
        storage: environment.local_storage.clone(),
        round_limit,
        quiz_session,
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

    view::render(&controller, actions::callbacks(&controller))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_direction_queries_have_safe_defaults() {
        let mut query = HashMap::new();
        assert_eq!(direction_from_query(&query), DirectionSelection::Mixed);
        query.insert("direction".to_owned(), "zh-to-en".to_owned());
        assert_eq!(
            direction_from_query(&query),
            DirectionSelection::ChineseToEnglish
        );
        query.insert("direction".to_owned(), "en-to-zh".to_owned());
        assert_eq!(
            direction_from_query(&query),
            DirectionSelection::EnglishToChinese
        );
        query.insert("direction".to_owned(), "invalid".to_owned());
        assert_eq!(direction_from_query(&query), DirectionSelection::Mixed);
    }
}
