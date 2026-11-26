use crate::services::practice_session_store::{
    ExerciseOutcome, PracticeSession, PracticeSessionError, PracticeSessionStore,
};

pub(super) fn retry_generation(
    store: &PracticeSessionStore,
    session: &PracticeSession,
) -> Result<PracticeSession, PracticeSessionError> {
    let mut updated = session.clone();
    let step_id = updated
        .current_step()
        .ok_or(PracticeSessionError::AlreadyFinished)?
        .id
        .clone();
    updated.retry_generation(&step_id)?;
    persist_or_reload(store, session, updated)
}

pub(super) fn complete_step(
    store: &PracticeSessionStore,
    session: &PracticeSession,
    outcome: ExerciseOutcome,
) -> Result<PracticeSession, PracticeSessionError> {
    let mut updated = session.clone();
    let step_id = updated
        .current_step()
        .ok_or(PracticeSessionError::AlreadyFinished)?
        .id
        .clone();
    updated.complete(&step_id, outcome)?;
    if updated.is_completed() {
        updated.stamp_completed_at(chrono::Utc::now().timestamp_millis() as u64)?;
    } else {
        updated.start_current()?;
    }
    persist_or_reload_progress(store, session, updated)
}

pub(super) fn finish_endless(
    store: &PracticeSessionStore,
    session: &PracticeSession,
) -> Result<PracticeSession, PracticeSessionError> {
    let mut updated = session.clone();
    updated.finish_endless_translation()?;
    updated.stamp_completed_at(chrono::Utc::now().timestamp_millis() as u64)?;
    persist_or_reload_progress(store, session, updated)
}

fn persist_or_reload(
    store: &PracticeSessionStore,
    original: &PracticeSession,
    updated: PracticeSession,
) -> Result<PracticeSession, PracticeSessionError> {
    match store.save(&updated) {
        Ok(()) => Ok(updated),
        Err(PracticeSessionError::OutOfDate) => store.load(&original.id),
        Err(error) => Err(error),
    }
}

fn persist_or_reload_progress(
    store: &PracticeSessionStore,
    original: &PracticeSession,
    updated: PracticeSession,
) -> Result<PracticeSession, PracticeSessionError> {
    match store.record_progress(&updated) {
        Ok(()) => Ok(updated),
        Err(PracticeSessionError::OutOfDate) => store.load(&original.id),
        Err(error) => Err(error),
    }
}
