use zipeek_core::practice::ExerciseVariant;
use zipeek_exercises::practice_draft::{PracticeDraft, VocabularySource};

use crate::services::{
    ai,
    word_sets::{self, UserWordSet},
};

pub(super) fn needs_local_tts(variants: &[ExerciseVariant]) -> bool {
    variants.iter().any(|variant| {
        matches!(
            variant,
            ExerciseVariant::MatchingAudioHanzi | ExerciseVariant::MatchingAudioMeaning
        )
    })
}

pub(super) struct StartEligibility {
    pub can_start: bool,
    pub message: &'static str,
}

pub(super) fn evaluate(
    draft: &PracticeDraft,
    _selected_variants: &[ExerciseVariant],
    personal_sets: &Result<Vec<UserWordSet>, String>,
    exclusions_ready: bool,
    snapshot_valid: bool,
    session_blocked: bool,
    ai_model_configured: bool,
) -> StartEligibility {
    let dictionary_filters_valid = match &draft.vocabulary {
        VocabularySource::Dictionary(filters) => {
            !filters.hsk.is_empty()
                || !filters.categories.is_empty()
                || !filters.parts_of_speech.is_empty()
        }
        VocabularySource::PersonalSet { .. } => true,
    };
    let vocabulary_ready = match &draft.vocabulary {
        VocabularySource::PersonalSet { reference } => personal_sets
            .as_ref()
            .ok()
            .and_then(|sets| word_sets::practice_snapshot(sets, reference).ok().flatten())
            .is_some_and(|ids| ids.len() >= word_sets::MIN_PRACTICE_WORDS),
        VocabularySource::Dictionary(_) => dictionary_filters_valid,
    };
    let missing_ai_model = !ai_model_configured
        && draft
            .exercise_ids
            .iter()
            .any(|id| zipeek_exercises::definition::needs_generated_content(id));
    let supported = draft.supports_local_vocabulary_session();
    let can_start = !session_blocked
        && !missing_ai_model
        && supported
        && exclusions_ready
        && snapshot_valid
        && vocabulary_ready
        && draft.validate().is_ok();
    let message = if session_blocked {
        "Resolve the existing Practice session or saved-session error before starting another."
    } else if missing_ai_model {
        "Configure an AI model in Settings before starting generated exercises."
    } else if !supported {
        "Choose a session length and exercises supported by the session host."
    } else {
        "Choose valid vocabulary and exercise settings before starting."
    };
    StartEligibility { can_start, message }
}

pub(super) fn ai_model_configured() -> bool {
    !ai::configuration().model.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_audio_matching_requires_local_tts() {
        assert!(!needs_local_tts(&[ExerciseVariant::MatchingHanziMeaning]));
        assert!(needs_local_tts(&[ExerciseVariant::MatchingAudioHanzi]));
        assert!(needs_local_tts(&[ExerciseVariant::MatchingAudioMeaning]));
    }

    #[test]
    fn existing_session_always_prevents_launch() {
        let draft = PracticeDraft::default();
        let status = evaluate(&draft, &[], &Ok(Vec::new()), true, true, true, true);
        assert!(!status.can_start);
        assert!(status.message.contains("existing Practice session"));
    }

    #[test]
    fn unavailable_ai_model_blocks_generated_content() {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        draft.exercise_ids.insert("sentence-translation".to_owned());
        let status = evaluate(&draft, &[], &Ok(Vec::new()), true, true, false, false);
        assert!(!status.can_start);
        assert!(status.message.contains("AI model"));
    }
}
