use zipeek_exercises::cloud_match::CloudPair;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CloudPoolKey {
    pub source: String,
    pub standard: u8,
    pub level: u8,
    pub word_ids: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CloudPoolResult {
    pub key: CloudPoolKey,
    pub result: Result<Vec<CloudPair>, String>,
}

pub(crate) fn current_result<'a>(
    current: &CloudPoolKey,
    loaded: &'a Option<CloudPoolResult>,
) -> Option<&'a Result<Vec<CloudPair>, String>> {
    loaded
        .as_ref()
        .filter(|loaded| loaded.key == *current)
        .map(|loaded| &loaded.result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(source: &str, ids: Vec<i32>) -> CloudPoolKey {
        CloudPoolKey {
            source: source.to_owned(),
            standard: 3,
            level: 1,
            word_ids: ids,
        }
    }

    #[test]
    fn obsolete_hsk_result_is_not_available_after_changing_level() {
        let previous = key("hsk", vec![]);
        let mut next = previous.clone();
        next.level = 2;
        let loaded = Some(CloudPoolResult {
            key: previous.clone(),
            result: Ok(Vec::new()),
        });
        assert!(current_result(&previous, &loaded).is_some());
        assert!(current_result(&next, &loaded).is_none());
    }

    #[test]
    fn word_set_membership_changes_invalidate_loaded_pairs() {
        let previous = key("sets", vec![1, 2]);
        let loaded = Some(CloudPoolResult {
            key: previous.clone(),
            result: Err("storage error".into()),
        });
        assert!(current_result(&key("sets", vec![1, 2]), &loaded).is_some());
        assert!(current_result(&key("sets", vec![1, 3]), &loaded).is_none());
        assert!(current_result(&key("hsk", vec![1, 2]), &loaded).is_none());
    }
}
