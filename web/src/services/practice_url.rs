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

pub(crate) fn fresh_offset(current: u64, pool_size: u64, sample: u64) -> u64 {
    if pool_size <= 1 {
        return 0;
    }
    if current >= pool_size {
        return sample % pool_size;
    }
    let offset = sample % (pool_size - 1);
    if offset >= current {
        offset + 1
    } else {
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restart_offset_is_in_pool_and_differs_from_current_even_with_same_entropy() {
        for current in 0..5 {
            for sample in [0, 1, 4, u64::MAX] {
                let next = fresh_offset(current, 5, sample);
                assert!(next < 5 && next != current);
            }
        }
        assert!(fresh_offset(u64::MAX, 5, 9) < 5);
        assert_eq!(fresh_offset(0, 0, 9), 0);
    }
}
