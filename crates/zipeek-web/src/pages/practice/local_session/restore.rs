use crate::services::practice_session_store::{
    PracticeSession, PracticeSessionError, PracticeSessionStore,
};

pub(super) fn restore_session(
    store: &PracticeSessionStore,
    id: &str,
) -> Result<PracticeSession, PracticeSessionError> {
    let mut session = store.load(id)?;
    if session.is_completed() {
        return Ok(session);
    }
    if session.recover_interrupted_generation() {
        store.save(&session)?;
    }
    if session.start_current()? {
        session.revision = session
            .revision
            .checked_add(1)
            .ok_or(PracticeSessionError::OutOfDate)?;
        store.save(&session)?;
    }
    Ok(session)
}
