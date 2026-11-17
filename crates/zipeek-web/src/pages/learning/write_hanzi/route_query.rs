use yew_router::prelude::Location;
use zipeek_core::WordId;

pub(super) fn selected_word_id(location: Option<&Location>) -> Option<WordId> {
    location
        .and_then(|location| {
            location
                .query::<std::collections::HashMap<String, String>>()
                .ok()
        })
        .and_then(|query| query.get("word_id").and_then(|id| id.parse::<i64>().ok()))
        .and_then(|id| WordId::try_from(id).ok())
}
