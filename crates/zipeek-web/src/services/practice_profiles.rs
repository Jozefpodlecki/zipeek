use serde::{Deserialize, Serialize};
use web_sys::Storage;
use zipeek_core::practice::ExerciseVariant;
use zipeek_exercises::{
    character_hunt::Difficulty,
    practice_draft::{
        DictionaryFilters, HskBand, MatchingColumnOrder, PracticeDraft, SessionLength,
        TranslationDirection, VocabularySource,
    },
};

use crate::models::PracticeExerciseOptions;

const KEY: &str = "zipeek.practice.profiles.v1";
const MAX_PROFILES: usize = 20;
const MAX_STORED_PROFILES_BYTES: usize = 262_144;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PracticeProfile {
    pub name: String,
    pub advanced: bool,
    exercises: Vec<String>,
    personal_set: Option<String>,
    hsk: Vec<(u8, u8)>,
    categories: Vec<String>,
    parts_of_speech: Vec<String>,
    fixed_rounds: Option<u16>,
    matching_formats: Vec<String>,
    multiple_choice_formats: Vec<String>,
    matching_reversed: bool,
    translation_direction: String,
    translation_word_bank: bool,
    translation_count: u16,
    hunt_difficulty: String,
    options: PracticeExerciseOptions,
}

impl PracticeProfile {
    pub fn capture(
        name: &str,
        advanced: bool,
        draft: &PracticeDraft,
        options: PracticeExerciseOptions,
    ) -> Result<Self, String> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 64 || name.chars().any(char::is_control) {
            return Err("Profile name must contain 1–64 printable characters".to_owned());
        }
        let (personal_set, hsk, categories, parts_of_speech) = match &draft.vocabulary {
            VocabularySource::PersonalSet { reference } => {
                (Some(reference.clone()), vec![], vec![], vec![])
            }
            VocabularySource::Dictionary(filters) => (
                None,
                filters
                    .hsk
                    .iter()
                    .map(|band| (band.system, band.level))
                    .collect(),
                filters.categories.iter().cloned().collect(),
                filters.parts_of_speech.iter().cloned().collect(),
            ),
        };
        let profile = Self {
            name: name.to_owned(),
            advanced,
            exercises: draft.exercise_ids.iter().cloned().collect(),
            personal_set,
            hsk,
            categories,
            parts_of_speech,
            fixed_rounds: match draft.length {
                SessionLength::Fixed(count) => Some(count),
                SessionLength::Endless => None,
            },
            matching_formats: draft
                .matching_formats
                .iter()
                .map(|variant| variant.seed_token().to_owned())
                .collect(),
            multiple_choice_formats: draft
                .multiple_choice_formats
                .iter()
                .map(|variant| variant.seed_token().to_owned())
                .collect(),
            matching_reversed: draft.matching_column_order == MatchingColumnOrder::Reversed,
            translation_direction: match draft.translation.direction {
                TranslationDirection::Mixed => "mixed",
                TranslationDirection::EnglishToChinese => "en-to-zh",
                TranslationDirection::ChineseToEnglish => "zh-to-en",
            }
            .to_owned(),
            translation_word_bank: draft.translation.use_word_bank,
            translation_count: draft.translation_count,
            hunt_difficulty: draft.hunt_difficulty.key().to_owned(),
            options,
        };
        profile.restore()?;
        Ok(profile)
    }

    pub fn exercise_count(&self) -> usize {
        self.exercises.len()
    }

    pub fn restore(&self) -> Result<(PracticeDraft, PracticeExerciseOptions, bool), String> {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        for id in &self.exercises {
            if !draft.exercise_ids.insert(id.clone())
                || zipeek_exercises::definition::definition(id).is_none()
            {
                return Err("Saved profile has an unknown or duplicate exercise".to_owned());
            }
        }
        draft.vocabulary = if let Some(reference) = &self.personal_set {
            if reference
                .parse::<crate::services::word_sets::WordSetId>()
                .is_err()
            {
                return Err("Saved profile references an invalid Word Set".to_owned());
            }
            VocabularySource::PersonalSet {
                reference: reference.clone(),
            }
        } else {
            let hsk = self
                .hsk
                .iter()
                .map(|(system, level)| {
                    HskBand::new(*system, *level)
                        .map_err(|_| "Saved profile has an invalid HSK band".to_owned())
                })
                .collect::<Result<_, _>>()?;
            VocabularySource::Dictionary(DictionaryFilters {
                hsk,
                categories: self.categories.iter().cloned().collect(),
                parts_of_speech: self.parts_of_speech.iter().cloned().collect(),
            })
        };
        draft.length = self
            .fixed_rounds
            .map_or(SessionLength::Endless, SessionLength::Fixed);
        draft.matching_formats = parse_formats(&self.matching_formats, false)?;
        draft.multiple_choice_formats = parse_formats(&self.multiple_choice_formats, true)?;
        draft.matching_column_order = if self.matching_reversed {
            MatchingColumnOrder::Reversed
        } else {
            MatchingColumnOrder::Standard
        };
        draft.translation.direction = match self.translation_direction.as_str() {
            "mixed" => TranslationDirection::Mixed,
            "en-to-zh" => TranslationDirection::EnglishToChinese,
            "zh-to-en" => TranslationDirection::ChineseToEnglish,
            _ => return Err("Saved profile has an invalid translation direction".to_owned()),
        };
        draft.translation.use_word_bank = self.translation_word_bank;
        draft.translation_count = self.translation_count;
        draft.hunt_difficulty = Difficulty::parse(&self.hunt_difficulty)
            .ok_or_else(|| "Saved profile has an invalid hunt difficulty".to_owned())?;
        draft
            .validate()
            .map_err(|error| format!("Invalid saved Practice configuration: {error:?}"))?;
        if draft.exercise_ids.is_empty() || self.exercises.len() > 16 {
            return Err("Saved profile requires at least one supported exercise".to_owned());
        }
        if !self.advanced
            && !draft.exercise_ids.iter().all(|id| {
                crate::reducers::practice_setup_flow::QUICK_EXERCISES.contains(&id.as_str())
            })
        {
            return Err("Saved Quick profile contains an Advanced exercise".to_owned());
        }
        if !self.options.sentence_transformation.valid() {
            return Err("Saved profile has an unavailable grammar pattern".to_owned());
        }
        Ok((draft, self.options, self.advanced))
    }
}

fn parse_formats(tokens: &[String], multiple_choice: bool) -> Result<Vec<ExerciseVariant>, String> {
    let mut formats = Vec::new();
    for token in tokens {
        let variant = ExerciseVariant::from_seed_token(token)
            .ok_or_else(|| format!("Unknown saved question format: {token}"))?;
        if variant.is_mcq() != multiple_choice || formats.contains(&variant) {
            return Err("Saved profile has duplicate or incompatible question formats".to_owned());
        }
        formats.push(variant);
    }
    Ok(formats)
}

fn validate(profiles: &[PracticeProfile]) -> Result<(), String> {
    if profiles.len() > MAX_PROFILES {
        return Err("Too many saved Practice profiles".to_owned());
    }
    for (index, profile) in profiles.iter().enumerate() {
        if profile.name.trim().is_empty()
            || profile.name.chars().count() > 64
            || profile.name.chars().any(char::is_control)
        {
            return Err("Invalid saved Practice profile name".to_owned());
        }
        if profiles[..index]
            .iter()
            .any(|other| other.name.eq_ignore_ascii_case(&profile.name))
        {
            return Err("Practice profile names must be unique".to_owned());
        }
        profile.restore()?;
    }
    Ok(())
}

pub(crate) fn decode(raw: &str) -> Result<Vec<PracticeProfile>, String> {
    if raw.len() > MAX_STORED_PROFILES_BYTES {
        return Err("Saved Practice profiles exceed the storage limit".to_owned());
    }
    let profiles: Vec<PracticeProfile> =
        serde_json::from_str(raw).map_err(|error| format!("Invalid Practice profiles: {error}"))?;
    validate(&profiles)?;
    Ok(profiles)
}

pub(crate) fn read(storage: &Storage) -> Result<Vec<PracticeProfile>, String> {
    match storage
        .get_item(KEY)
        .map_err(|_| "Could not read Practice profiles")?
    {
        Some(raw) => decode(&raw),
        None => Ok(Vec::new()),
    }
}

fn write(storage: &Storage, profiles: &[PracticeProfile]) -> Result<(), String> {
    validate(profiles)?;
    let data = serde_json::to_string(profiles).map_err(|error| error.to_string())?;
    if data.len() > MAX_STORED_PROFILES_BYTES {
        return Err("Saved Practice profiles exceed the storage limit".to_owned());
    }
    storage
        .set_item(KEY, &data)
        .map_err(|_| "Could not save Practice profiles".to_owned())
}

pub(crate) fn save(storage: &Storage, profile: PracticeProfile) -> Result<(), String> {
    let mut profiles = read(storage)?;
    if let Some(existing) = profiles
        .iter_mut()
        .find(|item| item.name.eq_ignore_ascii_case(&profile.name))
    {
        *existing = profile;
    } else {
        profiles.push(profile);
    }
    write(storage, &profiles)
}

fn remove_named(profiles: &mut Vec<PracticeProfile>, name: &str) -> Result<(), String> {
    let before = profiles.len();
    profiles.retain(|profile| !profile.name.eq_ignore_ascii_case(name));
    if before == profiles.len() {
        return Err("Practice profile not found".to_owned());
    }
    Ok(())
}

pub(crate) fn remove(storage: &Storage, name: &str) -> Result<(), String> {
    let mut profiles = read(storage)?;
    remove_named(&mut profiles, name)?;
    write(storage, &profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_round_trip_preserves_exercises_filters_and_options() {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.insert("character-builder".to_owned());
        let mut options = PracticeExerciseOptions::default();
        options.character_builder.answer_policy = crate::models::AnswerPolicy::RetryUntilCorrect;
        options.character_builder.automatic_check = true;
        options.type_pinyin.show_hint = true;
        let profile = PracticeProfile::capture(" Reading drill ", false, &draft, options).unwrap();
        let saved = decode(&serde_json::to_string(&vec![profile.clone()]).unwrap()).unwrap();
        let (restored, settings, advanced) = saved[0].restore().unwrap();
        assert_eq!(restored, draft);
        assert_eq!(settings, options);
        assert!(!advanced);
        assert_eq!(profile.name, "Reading drill");
    }

    #[test]
    fn profile_removal_uses_same_case_insensitive_identity_as_profile_save() {
        let draft = PracticeDraft::default();
        let profile = PracticeProfile::capture(
            "Daily Review",
            false,
            &draft,
            PracticeExerciseOptions::default(),
        )
        .unwrap();
        let mut profiles = vec![profile.clone()];
        assert!(remove_named(&mut profiles, "missing").is_err());
        assert_eq!(profiles, vec![profile]);
        remove_named(&mut profiles, "DAILY REVIEW").unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn malformed_saved_question_formats_are_rejected() {
        assert!(parse_formats(&["not-a-format".to_owned()], false).is_err());
        for token in ["hanzi-eng", "eng-hanzi"] {
            if let Some(format) = ExerciseVariant::from_seed_token(token) {
                let tokens = [
                    format.seed_token().to_owned(),
                    format.seed_token().to_owned(),
                ];
                assert!(parse_formats(&tokens, format.is_mcq()).is_err());
            }
        }
    }

    #[test]
    fn oversized_profile_inventory_is_rejected_before_deserialization() {
        assert!(decode(&" ".repeat(MAX_STORED_PROFILES_BYTES + 1)).is_err());
    }

    #[test]
    fn invalid_or_duplicate_profiles_are_rejected() {
        let draft = PracticeDraft::default();
        assert!(
            PracticeProfile::capture(" ", false, &draft, PracticeExerciseOptions::default())
                .is_err()
        );
        let profile =
            PracticeProfile::capture("Daily", false, &draft, PracticeExerciseOptions::default())
                .unwrap();
        let other =
            PracticeProfile::capture("daily", false, &draft, PracticeExerciseOptions::default())
                .unwrap();
        assert!(validate(&[profile, other]).is_err());
    }
}
