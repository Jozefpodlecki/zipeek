use serde::{Deserialize, Serialize};

const KEY: &str = "zipeek.user_word_sets.v1";
const MAX_SETS: usize = 50;
const MAX_WORDS: usize = 1000;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserWordSet {
    pub name: String,
    pub word_ids: Vec<i32>,
}

fn validate(sets: &[UserWordSet]) -> Result<(), String> {
    if sets.len() > MAX_SETS {
        return Err("Too many saved Word Sets".to_owned());
    }
    for (index, set) in sets.iter().enumerate() {
        if set.name.trim().is_empty() || set.name.chars().count() > 64 {
            return Err(format!("Invalid name for Word Set {index}"));
        }
        if set.word_ids.len() > MAX_WORDS
            || set.word_ids.iter().any(|id| *id <= 0)
            || set.word_ids.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(format!("Invalid IDs for Word Set {index}"));
        }
        if sets[..index].iter().any(|other| other.name == set.name) {
            return Err("Word Set names must be unique".to_owned());
        }
    }
    Ok(())
}

pub fn decode(raw: &str) -> Result<Vec<UserWordSet>, String> {
    let sets: Vec<UserWordSet> = serde_json::from_str(raw)
        .map_err(|error| format!("Invalid saved Word Set data: {error}"))?;
    validate(&sets)?;
    Ok(sets)
}

pub fn read() -> Result<Vec<UserWordSet>, String> {
    let storage = super::exclusions::storage().ok_or("Browser storage is unavailable")?;
    match storage.get_item(KEY).map_err(|_| "Could not read Word Sets")? {
        Some(raw) => decode(&raw),
        None => Ok(Vec::new()),
    }
}

pub fn write(sets: &[UserWordSet]) -> Result<(), String> {
    validate(sets)?;
    let storage = super::exclusions::storage().ok_or("Browser storage is unavailable")?;
    let raw = serde_json::to_string(sets).map_err(|error| error.to_string())?;
    storage.set_item(KEY, &raw).map_err(|_| "Could not save Word Sets".to_owned())
}

pub fn create(sets: &mut Vec<UserWordSet>, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 64 || sets.iter().any(|set| set.name == name) {
        return Err("Enter a unique name (1–64 characters)".to_owned());
    }
    if sets.len() >= MAX_SETS {
        return Err("Word Set limit reached".to_owned());
    }
    sets.push(UserWordSet { name: name.to_owned(), word_ids: Vec::new() });
    Ok(())
}

pub fn rename(sets: &mut [UserWordSet], index: usize, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 64 {
        return Err("Enter a name (1–64 characters)".to_owned());
    }
    if sets.iter().enumerate().any(|(other, set)| other != index && set.name == name) {
        return Err("Word Set names must be unique".to_owned());
    }
    let set = sets.get_mut(index).ok_or("Word Set not found")?;
    set.name = name.to_owned();
    Ok(())
}

pub fn delete(sets: &mut Vec<UserWordSet>, index: usize) -> Result<(), String> {
    if index >= sets.len() {
        return Err("Word Set not found".to_owned());
    }
    sets.remove(index);
    Ok(())
}

pub fn export(sets: &[UserWordSet]) -> Result<String, String> {
    validate(sets)?;
    serde_json::to_string_pretty(sets).map_err(|error| error.to_string())
}

pub fn merge_import(sets: &mut Vec<UserWordSet>, raw: &str) -> Result<usize, String> {
    let incoming = decode(raw)?;
    let mut merged = sets.clone();
    let mut added = 0;
    for new_set in incoming {
        if let Some(existing) = merged.iter_mut().find(|set| set.name == new_set.name) {
            let mut words = existing.word_ids.clone();
            words.extend(new_set.word_ids);
            words.sort_unstable();
            words.dedup();
            existing.word_ids = words;
        } else {
            merged.push(new_set);
            added += 1;
        }
    }
    validate(&merged)?;
    *sets = merged;
    Ok(added)
}

pub fn practice_snapshot<'a>(sets: &'a [UserWordSet], selected: &str) -> Result<Option<&'a [i32]>, String> {
    if selected.is_empty() {
        return Ok(None);
    }
    let index = selected.parse::<usize>().map_err(|_| "Invalid Word Set selection".to_owned())?;
    let set = sets.get(index).ok_or("Selected Word Set no longer exists")?;
    if set.word_ids.is_empty() {
        return Err("This Word Set has no words.".to_owned());
    }
    if set.word_ids.len() > 128 {
        return Err("Shared practice links currently support up to 128 words per set.".to_owned());
    }
    Ok(Some(&set.word_ids))
}

pub fn browse_snapshot(sets: &[UserWordSet], selected: &str) -> Result<Option<Vec<i32>>, String> {
    let Some(index) = selected.strip_prefix("saved:") else {
        return Ok(None);
    };
    let index = index.parse::<usize>().map_err(|_| "Invalid Word Set selection".to_owned())?;
    let set = sets.get(index).ok_or("Selected Word Set no longer exists")?;
    Ok(Some(set.word_ids.clone()))
}

pub fn toggle_word(sets: &mut [UserWordSet], index: usize, id: i32) -> Result<(), String> {
    if id <= 0 {
        return Err("Invalid word ID".to_owned());
    }
    let set = sets.get_mut(index).ok_or("Word Set not found")?;
    match set.word_ids.binary_search(&id) {
        Ok(position) => { set.word_ids.remove(position); }
        Err(position) if set.word_ids.len() < MAX_WORDS => { set.word_ids.insert(position, id); }
        Err(_) => return Err("Word Set contains too many words".to_owned()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_deduplication_and_roundtrip() {
        let mut sets = vec![];
        create(&mut sets, " Travel ").unwrap();
        assert!(create(&mut sets, "Travel").is_err());
        toggle_word(&mut sets, 0, 9).unwrap();
        toggle_word(&mut sets, 0, 2).unwrap();
        assert_eq!(sets[0].word_ids, vec![2, 9]);
        let json = serde_json::to_string(&sets).unwrap();
        assert_eq!(decode(&json).unwrap(), sets);
        toggle_word(&mut sets, 0, 2).unwrap();
        assert_eq!(sets[0].word_ids, vec![9]);
    }

    #[test]
    fn renaming_deleting_and_index_validation() {
        let mut sets = vec![];
        create(&mut sets, "Original").unwrap();
        create(&mut sets, "Other").unwrap();
        assert!(rename(&mut sets, 0, "Other").is_err());
        rename(&mut sets, 0, "Renamed").unwrap();
        assert_eq!(sets[0].name, "Renamed");
        assert!(rename(&mut sets, 99, "Missing").is_err());
        assert!(delete(&mut sets, 99).is_err());
        delete(&mut sets, 0).unwrap();
        assert_eq!(sets[0].name, "Other");
    }

    #[test]
    fn backup_merges_safely_without_overwriting_existing_words() {
        let mut sets = vec![];
        create(&mut sets, "Travel").unwrap();
        toggle_word(&mut sets, 0, 3).unwrap();
        let backup = r#"[{"name":"Travel","word_ids":[1,3]},{"name":"Review","word_ids":[9]}]"#;
        assert_eq!(merge_import(&mut sets, backup).unwrap(), 1);
        assert_eq!(sets[0].word_ids, vec![1, 3]);
        assert_eq!(sets[1].word_ids, vec![9]);
        assert_eq!(decode(&export(&sets).unwrap()).unwrap(), sets);
        assert_eq!(merge_import(&mut sets, backup).unwrap(), 0);
        assert_eq!(sets.len(), 2);
    }

    #[test]
    fn failed_merge_does_not_modify_saved_sets() {
        let mut sets = vec![];
        create(&mut sets, "Existing").unwrap();
        let original = sets.clone();
        assert!(merge_import(&mut sets, "not json").is_err());
        assert_eq!(sets, original);
        let too_many = format!("[{{\"name\":\"Existing\",\"word_ids\":[{}]}}]", (1..=1001).map(|n| n.to_string()).collect::<Vec<_>>().join(","));
        assert!(merge_import(&mut sets, &too_many).is_err());
        assert_eq!(sets, original);
    }

    #[test]
    fn practice_selection_never_defaults_to_unrestricted_on_bad_ids() {
        let sets = vec![UserWordSet { name: "Review".to_owned(), word_ids: vec![3, 8] }];
        assert!(practice_snapshot(&sets, "").unwrap().is_none());
        assert_eq!(practice_snapshot(&sets, "0").unwrap(), Some(&[3, 8][..]));
        for invalid in ["-1", "missing", "1", "9999999999999999999999"] {
            assert!(practice_snapshot(&sets, invalid).is_err(), "{invalid}");
        }
        assert!(practice_snapshot(&[], "0").is_err());
        let empty = vec![UserWordSet { name: "Empty".to_owned(), word_ids: vec![] }];
        assert!(practice_snapshot(&empty, "0").is_err());
        let oversized = vec![UserWordSet { name: "Big".to_owned(), word_ids: (1..=129).collect() }];
        assert!(practice_snapshot(&oversized, "0").is_err());
    }

    #[test]
    fn browse_selection_rejects_missing_sets_instead_of_dropping_the_filter() {
        let sets = vec![UserWordSet { name: "Travel".into(), word_ids: vec![3, 8] }];
        assert_eq!(browse_snapshot(&sets, ""), Ok(None));
        assert_eq!(browse_snapshot(&sets, "travel"), Ok(None));
        assert_eq!(browse_snapshot(&sets, "saved:0"), Ok(Some(vec![3, 8])));
        assert_eq!(browse_snapshot(&[UserWordSet { name: "Empty".into(), word_ids: vec![] }], "saved:0"), Ok(Some(vec![])));
        for invalid in ["saved:", "saved:bad", "saved:-1", "saved:1", "saved:999999999999999999"] {
            assert!(browse_snapshot(&sets, invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn invalid_and_unsorted_saved_data_is_rejected() {
        for raw in ["{}", "null", "[{\"name\":\"X\",\"word_ids\":[3,2]}]", "[{\"name\":\"X\",\"word_ids\":[-1]}]", "[{\"name\":\"\",\"word_ids\":[]}]", "[{\"name\":\"X\",\"word_ids\":[],\"surprise\":true}]"] {
            assert!(decode(raw).is_err(), "{raw}");
        }
    }
}
