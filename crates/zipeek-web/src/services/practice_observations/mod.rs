mod model;
mod storage;

use crate::services::practice_session_store::{PracticeSession, PracticeSessionError};
use web_sys::Storage;

pub(crate) use model::PracticeObservation;

fn recording_enabled(storage: &Storage) -> Result<bool, String> {
    use crate::services::practice_history::{RecordingPreference, recording_preference};
    Ok(recording_preference(storage)? == RecordingPreference::Enabled)
}

pub(crate) fn reconcile(storage: &Storage, session: &PracticeSession) -> Result<(), String> {
    if !recording_enabled(storage)? {
        return Ok(());
    }
    session
        .validate()
        .map_err(|error| format!("Invalid Practice session for observations: {error:?}"))?;
    for step in session.steps.iter().take(session.current) {
        storage::record(storage, &PracticeObservation::from_step(&session.id, step)?)?;
    }
    Ok(())
}

pub(crate) fn reconcile_progress(
    storage: &Storage,
    session: &PracticeSession,
) -> Result<(), PracticeSessionError> {
    if !recording_enabled(storage).map_err(PracticeSessionError::Storage)? {
        return Ok(());
    }
    session.validate()?;
    if session.current > 0 {
        if let Some(step) = session.steps.get(session.current - 1) {
            if step.outcome.is_some() {
                storage::record(
                    storage,
                    &PracticeObservation::from_step(&session.id, step)
                        .map_err(PracticeSessionError::Storage)?,
                )
                .map_err(PracticeSessionError::Storage)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn read_inbox(storage: &Storage) -> Result<Vec<PracticeObservation>, String> {
    if !recording_enabled(storage)? {
        return Ok(Vec::new());
    }
    storage::read_inbox(storage)
}

pub(crate) fn erase(storage: &Storage) -> Result<(), String> {
    storage::erase(storage)
}

#[cfg(test)]
mod tests;
