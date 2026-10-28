use std::collections::BTreeSet;
use web_sys::Storage;
use zipeek_core::WordId;

pub(crate) const EXCLUSIONS_KEY: &str = "zipeek.excluded_words";

pub(crate) fn storage() -> Option<Storage> {
    web_sys::window().and_then(|window| match window.local_storage() {
        Ok(storage) => storage,
        Err(error) => {
            log::warn!("Exclusion storage is unavailable: {error:?}");
            None
        }
    })
}

pub(crate) fn read_saved(storage: Option<&Storage>) -> BTreeSet<WordId> {
    let Some(storage) = storage else {
        return BTreeSet::new();
    };
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

pub(crate) fn reset_saved() -> Result<(), String> {
    let Some(storage) = storage() else {
        return Err("Saved exclusion storage is unavailable".to_owned());
    };
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
        for invalid in ["{}", "null", "not json", r#""1""#] {
            assert!(parse_saved(invalid).is_empty());
        }
    }
}
