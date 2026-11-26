use std::collections::HashMap;

use zipeek_exercises::practice_draft::VocabularySource;

use super::word_sets::{self, UserWordSet};

fn personal_set_ids<'a>(
    sets: &'a Result<Vec<UserWordSet>, String>,
    reference: &str,
) -> Result<&'a [i32], String> {
    let snapshot = sets
        .as_ref()
        .map_err(|error| format!("Word Sets unavailable: {error}"))?;
    word_sets::practice_snapshot(snapshot, reference)?
        .ok_or_else(|| "Choose a saved Word Set".to_owned())
}

pub(crate) fn create_session(
    draft: &zipeek_exercises::practice_draft::PracticeDraft,
    sets: &Result<Vec<UserWordSet>, String>,
    storage: &web_sys::Storage,
    exercise_options: crate::models::PracticeExerciseOptions,
) -> Result<String, String> {
    if !draft.supports_local_vocabulary_session() {
        return Err("Select supported exercises and a fixed 1 to 20 step session".to_owned());
    }
    if draft
        .exercise_ids
        .iter()
        .any(|id| zipeek_exercises::definition::needs_generated_content(id))
        && super::ai::configuration().model.trim().is_empty()
    {
        return Err(
            "Configure an AI model in Settings before starting generated exercises".to_owned(),
        );
    }
    if draft.has_exercise("matching") && exercise_options.matching_ai_generated
        && super::ai::configuration().model.trim().is_empty() {
        return Err("Configure an AI model in Settings before generating Matching questions".to_owned());
    }
    let translation_only = draft.translation_session_only();
    let count = if translation_only {
        if draft.translation_count == 0 {
            2
        } else {
            usize::from(draft.translation_count)
        }
    } else {
        draft
            .length
            .resumable_rounds()
            .ok_or_else(|| "Local sessions require 1 to 20 exercises".to_owned())?
    };
    let mut vocabulary = session_vocabulary_query(&draft.vocabulary, sets)?;
    if draft.has_exercise("character-hunt") {
        vocabulary.insert(
            "difficulty".to_owned(),
            draft.hunt_difficulty.key().to_owned(),
        );
    }
    let uuid = uuid::Uuid::now_v7();
    let session_id = uuid.to_string();
    let seed = u64::from_le_bytes(uuid.as_bytes()[..8].try_into().expect("eight UUID bytes"));
    let mut session = super::practice_session_store::PracticeSession::new(
        session_id.clone(),
        &draft.exercise_ids,
        count,
        seed,
        vocabulary.into_iter().collect(),
    )
    .map_err(|error| format!("Cannot prepare local session: {error:?}"))?;
    if let Some(super::practice_session_store::ExerciseConfiguration::TypePinyin(options)) =
        session.exercise_configurations.get_mut("type-pinyin")
    {
        *options = exercise_options.type_pinyin;
    }
    if let Some(super::practice_session_store::ExerciseConfiguration::CharacterBuilder(options)) =
        session.exercise_configurations.get_mut("character-builder")
    {
        *options = exercise_options.character_builder;
    }
    if let Some(super::practice_session_store::ExerciseConfiguration::ListeningChoice(options)) =
        session.exercise_configurations.get_mut("listening-choice")
    {
        *options = exercise_options.listening_choice;
    }
    if let Some(super::practice_session_store::ExerciseConfiguration::WriteHanzi(options)) =
        session.exercise_configurations.get_mut("write-hanzi")
    {
        *options = exercise_options.write_hanzi;
    }
    if session
        .exercise_configurations
        .contains_key("sentence-transformation")
        && !exercise_options.sentence_transformation.valid()
    {
        return Err("Select an available Sentence Transformation pattern".to_owned());
    }
    if let Some(super::practice_session_store::ExerciseConfiguration::SentenceTransformation(
        options,
    )) = session
        .exercise_configurations
        .get_mut("sentence-transformation")
    {
        *options = exercise_options.sentence_transformation;
    }
    let direction = match draft.translation.direction {
        zipeek_exercises::practice_draft::TranslationDirection::ChineseToEnglish => "zh-to-en",
        zipeek_exercises::practice_draft::TranslationDirection::EnglishToChinese => "en-to-zh",
        zipeek_exercises::practice_draft::TranslationDirection::Mixed => "mixed",
    }
    .to_owned();
    if let Some(super::practice_session_store::ExerciseConfiguration::SentenceTranslation {
        direction: stored,
    }) = session
        .exercise_configurations
        .get_mut("sentence-translation")
    {
        *stored = direction;
    }
    session.endless_translation = translation_only && draft.translation_count == 0;
    session
        .configure_quiz_formats(
            &draft.matching_formats,
            &draft.multiple_choice_formats,
            draft.matching_column_order
                == zipeek_exercises::practice_draft::MatchingColumnOrder::Reversed,
        )
        .map_err(|error| format!("Cannot prepare exercise formats: {error:?}"))?;
    if let Some(super::practice_session_store::ExerciseConfiguration::Quiz { pair_count, ai_generated, show_pinyin, .. }) =
        session.exercise_configurations.get_mut("matching")
    {
        *pair_count = exercise_options.matching_pairs;
        *ai_generated = exercise_options.matching_ai_generated;
        *show_pinyin = exercise_options.matching_show_pinyin;
    }
    if exercise_options.matching_ai_generated {
        for step in &mut session.steps {
            if step.exercise_id == "matching" {
                step.readiness = super::practice_session_store::ExerciseReadiness::NeedsGeneration;
            }
        }
        session.queue_generation(2);
    }
    session.validate().map_err(|error| format!("Invalid prepared session: {error:?}"))?;
    super::practice_session_store::PracticeSessionStore::new(storage.clone())
        .create(&session)
        .map_err(|error| format!("Cannot save local session: {error:?}"))?;
    Ok(session_id)
}

pub(crate) fn session_vocabulary_query(
    source: &VocabularySource,
    sets: &Result<Vec<UserWordSet>, String>,
) -> Result<HashMap<String, String>, String> {
    let mut query = HashMap::new();
    match source {
        VocabularySource::Dictionary(filters) => {
            query.insert(
                "source".to_owned(),
                super::practice_filter_source::encode(filters),
            );
        }
        VocabularySource::PersonalSet { reference } => {
            let ids = personal_set_ids(sets, reference)?;
            query.insert(
                "set_ids".to_owned(),
                ids.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
    }
    Ok(query)
}
