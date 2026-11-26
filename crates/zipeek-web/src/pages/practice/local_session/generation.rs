use crate::services::practice_session_store::{
    PracticeSession, PracticeSessionError, PracticeSessionStore,
};

pub(super) fn reconcile_generation(
    store: &PracticeSessionStore,
    session_id: &str,
    step_id: &str,
    attempt: u32,
    generated: Result<String, String>,
) -> Option<Result<PracticeSession, PracticeSessionError>> {
    let mut current = match store.load(session_id) {
        Ok(current) => current,
        Err(PracticeSessionError::NotFound) => return None,
        Err(error) => return Some(Err(error)),
    };
    let update = match generated {
        Ok(instance) => current.generation_succeeded(step_id, attempt, instance),
        Err(message) => current.generation_failed(step_id, attempt, message),
    };
    if update.is_ok() && !current.is_completed() {
        if let Err(error) = current.start_current() {
            return Some(Err(error));
        }
    }
    Some(match update {
        Ok(()) => match store.save(&current) {
            Ok(()) => Ok(current),
            Err(PracticeSessionError::OutOfDate) => store.load(session_id),
            Err(error) => Err(error),
        },
        Err(
            PracticeSessionError::StaleGeneration
            | PracticeSessionError::NotReady
            | PracticeSessionError::AlreadyFinished,
        ) => Ok(current),
        Err(error) => Err(error),
    })
}
