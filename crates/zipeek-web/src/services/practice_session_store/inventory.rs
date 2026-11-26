use super::{PracticeSession, PracticeSessionError};

pub(super) fn completed_removal_allowed(
    stored: &PracticeSession,
    expected: &PracticeSession,
) -> Result<(), PracticeSessionError> {
    if !expected.is_completed() {
        return Err(PracticeSessionError::NotReady);
    }
    if !stored.is_completed() || stored != expected {
        return Err(PracticeSessionError::OutOfDate);
    }
    Ok(())
}

#[derive(Debug, Default)]
pub(crate) struct PracticeSessionInventory {
    pub sessions: Vec<PracticeSession>,
    pub invalid_records: usize,
}

pub(super) fn ensure_new_session_allowed(
    inventory: &PracticeSessionInventory,
) -> Result<(), PracticeSessionError> {
    if inventory.invalid_records > 0 {
        return Err(PracticeSessionError::InvalidStoredSession);
    }
    if !inventory.sessions.is_empty() {
        return Err(PracticeSessionError::ExistingSession);
    }
    Ok(())
}

pub(super) fn inventory_keys_match(
    before: &[String],
    after: &[String],
) -> Result<(), PracticeSessionError> {
    if before == after {
        Ok(())
    } else {
        Err(PracticeSessionError::OutOfDate)
    }
}
