use std::collections::BTreeSet;
use web_sys::Storage;
use zipeek_core::WordId;

pub(crate) const EXCLUSIONS_KEY: &str = "zipeek.excluded_words";

pub(crate) fn read_saved(storage: &Storage) -> BTreeSet<WordId> {
    match storage.get_item(EXCLUSIONS_KEY) {
        Ok(Some(value)) => parse_saved(&value),
        Ok(None) => BTreeSet::new(),
        Err(error) => {
            log::warn!("Could not read saved exclusions: {error:?}");
            BTreeSet::new()
        }
    }
}

fn parse_saved(value: &str) -> BTreeSet<WordId> {
    if value.len() > 2 * 1024 * 1024 {
        log::warn!("Saved exclusions exceed the supported size limit");
        return BTreeSet::new();
    }
    let Ok(serde_json::Value::Array(values)) = serde_json::from_str(value) else {
        if cfg!(debug_assertions) {
            log::warn!("Saved exclusions must be a JSON array of WordIds");
        }
        return BTreeSet::new();
    };
    values
        .into_iter()
        .filter_map(|value| {
            let id = value.as_i64().and_then(|id| WordId::try_from(id).ok());
            if id.is_none() && cfg!(debug_assertions) {
                log::warn!("Ignoring invalid saved exclusion: {value}");
            }
            id
        })
        .collect()
}

pub(crate) fn write_saved(storage: &Storage, ids: &BTreeSet<WordId>) -> Result<(), String> {
    let values: Vec<i32> = ids.iter().map(|id| id.as_i32()).collect();
    let json = serde_json::to_string(&values)
        .map_err(|error| format!("Cannot serialize excluded words: {error}"))?;
    storage
        .set_item(EXCLUSIONS_KEY, &json)
        .map_err(|error| format!("Could not save excluded words: {error:?}"))
}

pub(crate) fn reset_saved(storage: &Storage) -> Result<(), String> {
    storage
        .remove_item(EXCLUSIONS_KEY)
        .map_err(|error| format!("Could not reset saved exclusions: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_ids_are_deduplicated_without_coercing_bad_values() {
        let ids = parse_saved(r#"[2,1,2,0,-1,"3",1.5,null,{},2147483648]"#);
        assert_eq!(
            ids.into_iter().map(|id| id.as_i32()).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert!(parse_saved(&"[1,".repeat(800_000)).is_empty());
        for invalid in ["{}", "null", "not json", r#""1""#] {
            assert!(parse_saved(invalid).is_empty());
        }
    }
}
