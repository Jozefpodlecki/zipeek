use std::collections::HashMap;

use yew_router::prelude::Location;
use zipeek_core::practice::{ParsedPracticeQuery, QueryWarning};

pub fn read_query(location: &Location) -> ParsedPracticeQuery {
    match location.query::<HashMap<String, String>>() {
        Ok(pairs) => ParsedPracticeQuery::from_pairs(
            pairs
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str())),
        ),
        Err(error) => {
            let mut query = ParsedPracticeQuery::default();
            query.warnings.push(QueryWarning {
                field: "query",
                value: error.to_string(),
            });
            query
        }
    }
}
