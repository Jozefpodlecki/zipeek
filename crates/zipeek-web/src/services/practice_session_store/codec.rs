use super::{MAX_STORED_SESSION_BYTES, PracticeSession, PracticeSessionError};

pub(super) fn decode_stored_session(
    id: &str,
    raw: &str,
) -> Result<PracticeSession, PracticeSessionError> {
    if raw.len() > MAX_STORED_SESSION_BYTES {
        return Err(PracticeSessionError::InvalidStoredSession);
    }
    let session: PracticeSession =
        serde_json::from_str(raw).map_err(|_| PracticeSessionError::InvalidStoredSession)?;
    session.validate()?;
    if session.id != id {
        return Err(PracticeSessionError::WrongSession);
    }
    Ok(session)
}
