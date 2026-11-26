use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WordSetId(String);

impl WordSetId {
    pub fn generate() -> Result<Self, String> {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes)
            .map_err(|error| format!("Cannot generate Word Set ID: {error}"))?;
        Ok(Self(
            bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for WordSetId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() == 32
            && value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            Ok(Self(value.to_owned()))
        } else {
            Err("Invalid Word Set ID".to_owned())
        }
    }
}

impl TryFrom<String> for WordSetId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<WordSetId> for String {
    fn from(value: WordSetId) -> Self {
        value.0
    }
}

impl fmt::Display for WordSetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub fn find<'a>(sets: &'a [UserWordSet], id: &WordSetId) -> Option<&'a UserWordSet> {
    sets.iter().find(|set| &set.id == id)
}

fn find_mut<'a>(
    sets: &'a mut [UserWordSet],
    id: &WordSetId,
) -> Result<&'a mut UserWordSet, String> {
    sets.iter_mut()
        .find(|set| &set.id == id)
        .ok_or_else(|| "Word Set not found".to_owned())
}

const KEY: &str = "zipeek.user_word_sets.v1";
const ACTIVE_SET_KEY: &str = "zipeek.active_dictionary_word_set.v1";

pub fn active_dictionary_set(storage: &web_sys::Storage) -> Option<WordSetId> {
    storage
        .get_item(ACTIVE_SET_KEY)
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok())
}

pub fn set_active_dictionary_set(
    storage: &web_sys::Storage,
    id: Option<&WordSetId>,
) -> Result<(), String> {
    match id {
        Some(id) => storage.set_item(ACTIVE_SET_KEY, id.as_str()),
        None => storage.remove_item(ACTIVE_SET_KEY),
    }
    .map_err(|_| "Could not save the active Dictionary Word Set".to_owned())
}

const MAX_SETS: usize = 250;
const MAX_WORDS: usize = 1000;
pub const MIN_PRACTICE_WORDS: usize = 10;
pub const COURSE_WORD_SET_SIZE: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CourseWordSetSource {
    pub region_id: u16,
    pub set_index: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserWordSet {
    pub id: WordSetId,
    pub name: String,
    pub word_ids: Vec<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<CourseWordSetSource>,
}

fn valid_course_source(source: &CourseWordSetSource) -> bool {
    zipeek_pedagogy::curriculum::VOCABULARY_REGIONS
        .get(usize::from(source.region_id).saturating_sub(1))
        .is_some_and(|region| {
            region.id == source.region_id
                && source.set_index > 0
                && usize::from(source.set_index)
                    <= region.readings.len().div_ceil(COURSE_WORD_SET_SIZE)
        })
}

fn valid_set_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().count() <= 64 && !name.chars().any(char::is_control)
}

fn validate(sets: &[UserWordSet]) -> Result<(), String> {
    if sets.len() > MAX_SETS {
        return Err("Too many saved Word Sets".to_owned());
    }
    for (index, set) in sets.iter().enumerate() {
        if !valid_set_name(&set.name) {
            return Err(format!("Invalid name for Word Set {index}"));
        }
        if set.word_ids.len() > MAX_WORDS
            || set.word_ids.iter().any(|id| *id <= 0)
            || set.word_ids.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(format!("Invalid IDs for Word Set {index}"));
        }
        if sets[..index].iter().any(|other| other.id == set.id) {
            return Err("Word Set IDs must be unique".to_owned());
        }
        if let Some(source) = &set.source {
            if !valid_course_source(source) {
                return Err("Invalid course Word Set source".to_owned());
            }
            if sets[..index]
                .iter()
                .any(|other| other.source.as_ref() == Some(source))
            {
                return Err("Course Word Set already acquired".to_owned());
            }
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

pub fn read(storage: &web_sys::Storage) -> Result<Vec<UserWordSet>, String> {
    match storage
        .get_item(KEY)
        .map_err(|_| "Could not read Word Sets")?
    {
        Some(raw) => decode(&raw),
        None => Ok(Vec::new()),
    }
}

pub fn write(storage: &web_sys::Storage, sets: &[UserWordSet]) -> Result<(), String> {
    validate(sets)?;
    let raw = serde_json::to_string(sets).map_err(|error| error.to_string())?;
    storage
        .set_item(KEY, &raw)
        .map_err(|_| "Could not save Word Sets".to_owned())
}

pub fn create(sets: &mut Vec<UserWordSet>, name: &str) -> Result<(), String> {
    let name = name.trim();
    if !valid_set_name(name) || sets.iter().any(|set| set.name == name) {
        return Err("Enter a unique name (1–64 characters)".to_owned());
    }
    if sets.len() >= MAX_SETS {
        return Err("Word Set limit reached".to_owned());
    }
    sets.push(UserWordSet {
        id: WordSetId::generate()?,
        name: name.to_owned(),
        word_ids: Vec::new(),
        source: None,
    });
    Ok(())
}

pub fn create_with_words(
    sets: &mut Vec<UserWordSet>,
    name: &str,
    word_ids: &[i32],
) -> Result<(), String> {
    if word_ids.is_empty() || word_ids.len() > MAX_WORDS {
        return Err("Course vocabulary is empty or exceeds the Word Set limit".to_owned());
    }
    let mut sorted = word_ids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.len() != word_ids.len() || sorted.iter().any(|id| *id <= 0) {
        return Err("Course vocabulary contains invalid or repeated dictionary IDs".to_owned());
    }
    create(sets, name)?;
    if let Some(set) = sets.last_mut() {
        set.word_ids = sorted;
    }
    Ok(())
}

pub fn create_from_course(
    sets: &mut Vec<UserWordSet>,
    name: &str,
    word_ids: &[i32],
    source: CourseWordSetSource,
) -> Result<(), String> {
    if !valid_course_source(&source) {
        return Err("Invalid course Word Set source".to_owned());
    }
    if sets.iter().any(|set| set.source.as_ref() == Some(&source)) {
        return Err("Course Word Set already acquired".to_owned());
    }
    let mut candidate = sets.clone();
    create_with_words(&mut candidate, name, word_ids)?;
    if let Some(set) = candidate.last_mut() {
        set.source = Some(source);
    }
    validate(&candidate)?;
    *sets = candidate;
    Ok(())
}

pub fn acquired_source(sets: &[UserWordSet], source: &CourseWordSetSource) -> bool {
    sets.iter().any(|set| set.source.as_ref() == Some(source))
}

pub fn rename(sets: &mut [UserWordSet], id: &WordSetId, name: &str) -> Result<(), String> {
    let name = name.trim();
    if !valid_set_name(name) {
        return Err("Enter a name (1–64 characters)".to_owned());
    }
    if sets.iter().any(|set| &set.id != id && set.name == name) {
        return Err("Word Set names must be unique".to_owned());
    }
    let set = find_mut(sets, id)?;
    set.name = name.to_owned();
    Ok(())
}

pub fn delete(sets: &mut Vec<UserWordSet>, id: &WordSetId) -> Result<(), String> {
    let index = sets
        .iter()
        .position(|set| &set.id == id)
        .ok_or("Word Set not found")?;
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
        if let Some(existing) = merged.iter_mut().find(|set| set.id == new_set.id) {
            if existing.name != new_set.name || existing.source != new_set.source {
                return Err("Imported Word Set ID has a conflicting name".to_owned());
            }
            let mut words = existing.word_ids.clone();
            words.extend(new_set.word_ids);
            words.sort_unstable();
            words.dedup();
            existing.word_ids = words;
        } else if merged.iter().any(|set| set.name == new_set.name) {
            return Err("Imported Word Set name belongs to a different ID".to_owned());
        } else {
            merged.push(new_set);
            added += 1;
        }
    }
    validate(&merged)?;
    *sets = merged;
    Ok(added)
}

pub fn practice_snapshot<'a>(
    sets: &'a [UserWordSet],
    selected: &str,
) -> Result<Option<&'a [i32]>, String> {
    if selected.is_empty() {
        return Ok(None);
    }
    let id: WordSetId = selected.parse()?;
    let set = find(sets, &id).ok_or("Selected Word Set no longer exists")?;
    if set
        .word_ids
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>()
        .len()
        < MIN_PRACTICE_WORDS
    {
        return Err(format!(
            "Practice requires at least {MIN_PRACTICE_WORDS} distinct playable words in the selected Word Set."
        ));
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
    let id: WordSetId = index.parse()?;
    let set = find(sets, &id).ok_or("Selected Word Set no longer exists")?;
    Ok(Some(set.word_ids.clone()))
}

pub fn toggle_word(sets: &mut [UserWordSet], set_id: &WordSetId, id: i32) -> Result<(), String> {
    if id <= 0 {
        return Err("Invalid word ID".to_owned());
    }
    let set = find_mut(sets, set_id)?;
    match set.word_ids.binary_search(&id) {
        Ok(position) => {
            set.word_ids.remove(position);
        }
        Err(position) if set.word_ids.len() < MAX_WORDS => {
            set.word_ids.insert(position, id);
        }
        Err(_) => return Err("Word Set contains too many words".to_owned()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_source_requires_real_region_and_existing_set_index() {
        assert!(valid_course_source(&CourseWordSetSource {
            region_id: 1,
            set_index: 1
        }));
        assert!(!valid_course_source(&CourseWordSetSource {
            region_id: 0,
            set_index: 1
        }));
        assert!(!valid_course_source(&CourseWordSetSource {
            region_id: 101,
            set_index: 1
        }));
        assert!(!valid_course_source(&CourseWordSetSource {
            region_id: 1,
            set_index: 100
        }));
    }

    #[test]
    fn failed_course_acquisition_leaves_collection_unmodified() {
        let mut sets = Vec::new();
        assert!(
            create_from_course(
                &mut sets,
                "Invalid",
                &[1],
                CourseWordSetSource {
                    region_id: 1,
                    set_index: 100
                }
            )
            .is_err()
        );
        assert!(sets.is_empty());
    }

    #[test]
    fn course_identity_survives_rename_and_rejects_duplicates() {
        let mut sets = Vec::new();
        let source = CourseWordSetSource {
            region_id: 5,
            set_index: 2,
        };
        create_from_course(&mut sets, "Actions", &[4, 2, 3], source.clone()).unwrap();
        assert!(acquired_source(&sets, &source));
        let id = sets[0].id.clone();
        rename(&mut sets, &id, "My renamed actions").unwrap();
        assert!(acquired_source(&sets, &source));
        assert!(create_from_course(&mut sets, "Different", &[7], source.clone()).is_err());
        assert_eq!(sets.len(), 1);
        let saved = export(&sets).unwrap();
        assert!(acquired_source(&decode(&saved).unwrap(), &source));
        toggle_word(&mut sets, &id, 3).unwrap();
        assert!(acquired_source(&sets, &source));
        delete(&mut sets, &id).unwrap();
        assert!(!acquired_source(&sets, &source));
    }

    #[test]
    fn imported_duplicate_course_source_is_rejected() {
        let source = CourseWordSetSource {
            region_id: 1,
            set_index: 1,
        };
        let mut sets = Vec::new();
        create_from_course(&mut sets, "One", &[1], source.clone()).unwrap();
        let mut other = Vec::new();
        create_from_course(&mut other, "Two", &[2], source).unwrap();
        assert!(merge_import(&mut sets, &export(&other).unwrap()).is_err());
    }

    #[test]
    fn personal_word_sets_can_hold_the_complete_vocabulary_curriculum() {
        let mut sets = Vec::new();
        for index in 0..130 {
            create_with_words(
                &mut sets,
                &format!("HSK thematic set {}", index + 1),
                &[1, 2, 3],
            )
            .unwrap();
        }
        assert_eq!(sets.len(), 130);
        assert!(validate(&sets).is_ok());
        let serialized = export(&sets).unwrap();
        assert_eq!(decode(&serialized).unwrap(), sets);
    }

    #[test]
    fn course_snapshot_is_sorted_and_rejects_invalid_entries() {
        let mut sets = Vec::new();
        create_with_words(&mut sets, "Time", &[9, 2, 5]).unwrap();
        assert_eq!(sets[0].word_ids, vec![2, 5, 9]);
        assert!(create_with_words(&mut sets, "Bad", &[2, 2]).is_err());
        assert_eq!(sets.len(), 1);
    }

    #[test]
    fn identities_survive_reordering_and_deletion() {
        let mut sets = Vec::new();
        create(&mut sets, "Travel").unwrap();
        create(&mut sets, "Review").unwrap();
        let travel = sets[0].id.clone();
        let review = sets[1].id.clone();
        toggle_word(&mut sets, &travel, 3).unwrap();
        sets.swap(0, 1);
        assert!(practice_snapshot(&sets, travel.as_str()).is_err());
        assert_eq!(find(&sets, &review).unwrap().name, "Review");
        delete(&mut sets, &review).unwrap();
        assert!(find(&sets, &review).is_none());
        assert_eq!(find(&sets, &travel).unwrap().word_ids, vec![3]);
    }

    #[test]
    fn practice_requires_ten_distinct_playable_words() {
        let mut sets = Vec::new();
        create(&mut sets, "Practice").unwrap();
        let id = sets[0].id.clone();
        for word_id in 1..10 {
            toggle_word(&mut sets, &id, word_id).unwrap();
        }
        assert!(
            practice_snapshot(&sets, id.as_str())
                .unwrap_err()
                .contains("10 distinct")
        );
        toggle_word(&mut sets, &id, 10).unwrap();
        assert_eq!(
            practice_snapshot(&sets, id.as_str()),
            Ok(Some(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..]))
        );
        sets[0].word_ids[9] = 9;
        assert!(practice_snapshot(&sets, id.as_str()).is_err());
    }

    #[test]
    fn imports_preserve_id_and_reject_identity_collisions() {
        let mut sets = Vec::new();
        create(&mut sets, "Travel").unwrap();
        let original = sets.clone();
        assert_eq!(
            merge_import(&mut sets, &export(&original).unwrap()).unwrap(),
            0
        );
        assert_eq!(sets, original);
        let mut other = original.clone();
        other[0].id = WordSetId::generate().unwrap();
        assert!(merge_import(&mut sets, &export(&other).unwrap()).is_err());
        let mut renamed = original.clone();
        renamed[0].name = "Different".into();
        assert!(merge_import(&mut sets, &export(&renamed).unwrap()).is_err());
        assert_eq!(sets, original);
    }

    #[test]
    fn malformed_or_duplicate_ids_fail_before_persistence() {
        let mut sets = Vec::new();
        create(&mut sets, "One").unwrap();
        let mut second = sets[0].clone();
        second.name = "Two".into();
        sets.push(second);
        assert!(validate(&sets).is_err());
        assert!(practice_snapshot(&sets, "0").is_err());
        assert!(browse_snapshot(&sets, "saved:0").is_err());
        assert!("ABC".parse::<WordSetId>().is_err());
    }
}

#[cfg(test)]
mod r593_name_boundary_tests {
    use super::*;

    #[test]
    fn names_must_not_contain_control_characters() {
        assert!(!valid_set_name("Control\nName"));
        assert!(!valid_set_name("\t"));
        assert!(valid_set_name("Review words"));
        let mut sets = Vec::new();
        assert!(create(&mut sets, "Bad\rName").is_err());
        assert!(sets.is_empty());
    }
}
