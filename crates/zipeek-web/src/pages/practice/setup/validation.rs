use crate::contexts::service_management::ServiceAvailability;
use zipeek_core::practice::ExerciseVariant;
use zipeek_exercises::definition::ExternalService;
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
    pub message: String,
    pub missing_service: Option<ExternalService>,
}

pub(super) fn evaluate(
    draft: &PracticeDraft,
    image_readiness: &ServiceAvailability,
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
    let needs_image = draft.has_exercise("image-vocabulary");
    let missing_image = needs_image && *image_readiness != ServiceAvailability::Available;
    let supported = draft.supports_local_vocabulary_session();
    let can_start = !session_blocked
        && !missing_ai_model
        && !missing_image
        && supported
        && exclusions_ready
        && snapshot_valid
        && vocabulary_ready
        && draft.validate().is_ok();
    let (message, missing_service) = if session_blocked {
        (
            "Resolve the existing Practice session or saved-session error before starting another."
                .to_owned(),
            None,
        )
    } else if missing_ai_model {
        let message = if needs_image {
            "Picture vocabulary needs a text AI model to compose the scene as well as the image service. Select and save a text model in AI settings."
        } else {
            "Configure an AI model in Settings before starting generated exercises."
        };
        (message.to_owned(), Some(ExternalService::AiText))
    } else if missing_image {
        let message = match image_readiness {
            ServiceAvailability::Checking => "Checking the image service for Picture vocabulary...".to_owned(),
            ServiceAvailability::Failed(error) => format!("Picture vocabulary needs a ready image generation service. {error}"),
            _ => "Picture vocabulary needs a ready image generation service. Start it in Image Generation settings and retry.".to_owned(),
        };
        (message, Some(ExternalService::ImageGeneration))
    } else if !supported {
        (
            "Choose a session length and exercises supported by the session host.".to_owned(),
            None,
        )
    } else {
        (
            "Choose valid vocabulary and exercise settings before starting.".to_owned(),
            None,
        )
    };
    StartEligibility {
        can_start,
        message,
        missing_service,
    }
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
        let status = evaluate(
            &draft,
            &ServiceAvailability::Available,
            &Ok(Vec::new()),
            true,
            true,
            true,
            true,
        );
        assert!(!status.can_start);
        assert!(status.message.contains("existing Practice session"));
    }

    #[test]
    fn unavailable_ai_model_blocks_generated_content() {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        draft.exercise_ids.insert("sentence-translation".to_owned());
        let status = evaluate(
            &draft,
            &ServiceAvailability::Available,
            &Ok(Vec::new()),
            true,
            true,
            false,
            false,
        );
        assert!(!status.can_start);
        assert!(status.message.contains("AI model"));
    }

    #[test]
    fn picture_vocabulary_distinguishes_text_model_and_image_readiness() {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.clear();
        draft.exercise_ids.insert("image-vocabulary".to_owned());
        let text = evaluate(
            &draft,
            &ServiceAvailability::Available,
            &Ok(Vec::new()),
            true,
            true,
            false,
            false,
        );
        assert!(!text.can_start);
        assert_eq!(text.missing_service, Some(ExternalService::AiText));
        assert!(text.message.contains("compose the scene"));
        for readiness in [
            ServiceAvailability::Checking,
            ServiceAvailability::Unavailable,
            ServiceAvailability::Failed("Preview-only backend".into()),
        ] {
            let image = evaluate(&draft, &readiness, &Ok(Vec::new()), true, true, false, true);
            assert!(!image.can_start);
            assert_eq!(
                image.missing_service,
                Some(ExternalService::ImageGeneration)
            );
        }
        let ready = evaluate(
            &draft,
            &ServiceAvailability::Available,
            &Ok(Vec::new()),
            true,
            true,
            false,
            true,
        );
        assert!(ready.can_start);
        assert_eq!(ready.missing_service, None);
    }

    #[test]
    fn local_exercises_ignore_image_service_failures() {
        let draft = PracticeDraft::default();
        let status = evaluate(
            &draft,
            &ServiceAvailability::Failed("offline".into()),
            &Ok(Vec::new()),
            true,
            true,
            false,
            false,
        );
        assert!(status.can_start);
        assert_eq!(status.missing_service, None);
    }
}
