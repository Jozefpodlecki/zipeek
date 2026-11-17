use serde::Serialize;
use web_sys::{Document, Storage};

use crate::preferences::PreferencesStore;

use super::practice_session_store::{
    ExerciseOutcome, PracticeSession, PracticeSessionError, PracticeSessionStore,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecordingPreference {
    Enabled,
    Disabled,
}

pub(crate) fn recording_preference(storage: &Storage) -> Result<RecordingPreference, String> {
    let preferences = PreferencesStore::new(storage).read_checked()?;
    Ok(if preferences.learning_history_enabled {
        RecordingPreference::Enabled
    } else {
        RecordingPreference::Disabled
    })
}

pub(crate) fn notify_recording_preference_changed() {
    if let Some(window) = web_sys::window()
        && let Ok(event) = web_sys::Event::new("zipeek-recording-change")
    {
        let _ = window.dispatch_event(&event);
    }
}

pub(crate) fn set_recording_preference(storage: &Storage, enabled: bool) -> Result<(), String> {
    PreferencesStore::new(storage).update(|preferences| {
        preferences.learning_history_enabled = enabled;
    })?;
    notify_recording_preference_changed();
    Ok(())
}

#[derive(Serialize)]
struct SessionEvidence<'a> {
    schema_version: u8,
    session_id: &'a str,
    completed_at_ms: u64,
    steps: Vec<StepEvidence<'a>>,
}

#[derive(Serialize)]
struct StepEvidence<'a> {
    step_id: &'a str,
    exercise_id: &'a str,
    outcome: &'a ExerciseOutcome,
}

pub(crate) fn encode_evidence(session: &PracticeSession) -> Result<String, PracticeSessionError> {
    session.validate()?;
    if !session.is_completed() {
        return Err(PracticeSessionError::NotReady);
    }
    let completed_at_ms = u64::try_from(
        session
            .completed_at
            .ok_or(PracticeSessionError::InvalidResult)?
            .timestamp_millis(),
    )
    .map_err(|_| PracticeSessionError::InvalidResult)?;
    let steps = session
        .steps
        .iter()
        .map(|step| {
            Ok(StepEvidence {
                step_id: &step.id,
                exercise_id: &step.exercise_id,
                outcome: step
                    .outcome
                    .as_ref()
                    .ok_or(PracticeSessionError::InvalidResult)?,
            })
        })
        .collect::<Result<Vec<_>, PracticeSessionError>>()?;
    serde_json::to_string(&SessionEvidence {
        schema_version: 2,
        session_id: &session.id,
        completed_at_ms,
        steps,
    })
    .map_err(|_| PracticeSessionError::InvalidResult)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompletionDisposition {
    Recorded,
    NotRecorded,
}

pub(crate) async fn finalize_completed(
    storage: Storage,
    document: Document,
    session: PracticeSession,
) -> Result<CompletionDisposition, String> {
    let evidence = encode_evidence(&session)
        .map_err(|error| format!("Invalid completed session: {error:?}"))?;
    let preference = recording_preference(&storage)?;
    if preference == RecordingPreference::Enabled {
        super::learning_history::commit(&document, &session.id, &evidence)
            .await
            .map_err(|error| format!("Learning history was not committed: {error}"))?;
    }
    PracticeSessionStore::new(storage)
        .remove_completed(&session)
        .map_err(|error| {
            format!(
                "Learning history handling finished but temporary session cleanup failed: {error:?}"
            )
        })?;
    Ok(if preference == RecordingPreference::Enabled {
        CompletionDisposition::Recorded
    } else {
        CompletionDisposition::NotRecorded
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn session() -> PracticeSession {
        let exercises = BTreeSet::from(["type-pinyin".to_owned()]);
        PracticeSession::new(
            "123e4567-e89b-72d3-a456-426614174001".to_owned(),
            &exercises,
            1,
            42,
            BTreeMap::from([("source".to_owned(), "秘密".to_owned())]),
        )
        .unwrap()
    }

    #[test]
    fn recording_is_off_without_explicit_consent() {
        assert!(!crate::preferences::Preferences::default().learning_history_enabled);
        assert!(
            !crate::preferences::Preferences::parse(r#"{"preset":"jade"}"#)
                .unwrap()
                .learning_history_enabled
        );
    }

    #[test]
    fn timestamped_evidence_is_required_and_retry_stable() {
        let mut finished = session();
        let id = finished.current_step().unwrap().id.clone();
        finished.complete(&id, ExerciseOutcome::Completed).unwrap();
        assert_eq!(
            encode_evidence(&finished),
            Err(PracticeSessionError::InvalidResult)
        );
        finished
            .stamp_completed_at(
                (finished.created_at + chrono::Duration::seconds(10)).timestamp_millis() as u64,
            )
            .unwrap();
        let evidence = encode_evidence(&finished).unwrap();
        assert!(evidence.contains("\"schema_version\":2"));
        assert!(evidence.contains("\"completed_at_ms\":"));
        assert_eq!(evidence, encode_evidence(&finished).unwrap());
    }

    #[test]
    fn evidence_rejects_zero_completion_timestamp() {
        let mut finished = session();
        let id = finished.current_step().unwrap().id.clone();
        finished.complete(&id, ExerciseOutcome::Completed).unwrap();
        assert!(finished.stamp_completed_at(0).is_err());
        assert_eq!(
            encode_evidence(&finished),
            Err(PracticeSessionError::InvalidResult)
        );
    }

    #[test]
    fn refuses_incomplete_evidence() {
        assert_eq!(
            encode_evidence(&session()),
            Err(PracticeSessionError::NotReady)
        );
    }

    #[test]
    fn evidence_excludes_vocabulary_and_generated_material() {
        let mut finished = session();
        let id = finished.current_step().unwrap().id.clone();
        finished.complete(&id, ExerciseOutcome::Completed).unwrap();
        finished
            .stamp_completed_at(
                (finished.created_at + chrono::Duration::seconds(10)).timestamp_millis() as u64,
            )
            .unwrap();
        let evidence = encode_evidence(&finished).unwrap();
        assert!(evidence.contains("type-pinyin"));
        assert!(evidence.contains("completed"));
        assert!(!evidence.contains("秘密"));
        assert!(!evidence.contains("vocabulary"));
        assert!(!evidence.contains("generated_instance"));
    }
}
