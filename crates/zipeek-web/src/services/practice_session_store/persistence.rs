use super::{
    MAX_STORED_SESSION_BYTES, PracticeSession, PracticeSessionError, SESSION_KEY_PREFIX,
    notify_session_change, session_storage_key,
};
use web_sys::Storage;

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

#[derive(Clone)]
pub(crate) struct PracticeSessionStore {
    storage: Storage,
}

impl PracticeSessionStore {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn recoverable_sessions(&self) -> Result<PracticeSessionInventory, PracticeSessionError> {
        let mut inventory = PracticeSessionInventory::default();
        let count = self
            .storage
            .length()
            .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?;
        for index in 0..count {
            let Some(key) = self
                .storage
                .key(index)
                .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?
            else {
                continue;
            };
            let Some(id) = key.strip_prefix(SESSION_KEY_PREFIX) else {
                continue;
            };
            let Some(raw) = self
                .storage
                .get_item(&key)
                .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?
            else {
                continue;
            };
            match decode_stored_session(id, &raw) {
                Ok(session) => inventory.sessions.push(session),
                Err(_) => inventory.invalid_records += 1,
            }
        }
        inventory.sessions.sort_by(|left, right| {
            right
                .is_completed()
                .cmp(&left.is_completed())
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(inventory)
    }

    pub fn load(&self, id: &str) -> Result<PracticeSession, PracticeSessionError> {
        let key = session_storage_key(id)?;
        let raw = self
            .storage
            .get_item(&key)
            .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?
            .ok_or(PracticeSessionError::NotFound)?;
        decode_stored_session(id, &raw)
    }

    pub fn create(&self, session: &PracticeSession) -> Result<(), PracticeSessionError> {
        session.validate()?;
        let key = session_storage_key(&session.id)?;
        if self
            .storage
            .get_item(&key)
            .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?
            .is_some()
        {
            return Err(PracticeSessionError::OutOfDate);
        }
        let inventory = self.recoverable_sessions()?;
        ensure_new_session_allowed(&inventory)?;
        self.write(session)
    }

    pub fn save(&self, session: &PracticeSession) -> Result<(), PracticeSessionError> {
        session.validate()?;
        let stored = self.load(&session.id)?;
        if stored.revision.checked_add(1) != Some(session.revision) {
            return Err(PracticeSessionError::OutOfDate);
        }
        self.write(session)
    }

    pub fn record_progress(&self, session: &PracticeSession) -> Result<(), PracticeSessionError> {
        self.save(session)
    }

    pub fn remove_completed(&self, session: &PracticeSession) -> Result<(), PracticeSessionError> {
        session.validate()?;
        let stored = self.load(&session.id)?;
        completed_removal_allowed(&stored, session)?;
        self.storage
            .remove_item(&session_storage_key(&session.id)?)
            .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?;
        notify_session_change();
        Ok(())
    }

    fn write(&self, session: &PracticeSession) -> Result<(), PracticeSessionError> {
        let json = serde_json::to_string(session)
            .map_err(|_| PracticeSessionError::InvalidStoredSession)?;
        let key = session_storage_key(&session.id)?;
        if json.len() > MAX_STORED_SESSION_BYTES {
            return Err(PracticeSessionError::InvalidStoredSession);
        }
        self.storage
            .set_item(&key, &json)
            .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?;
        notify_session_change();
        Ok(())
    }

    pub fn discard(&self, id: &str) -> Result<(), PracticeSessionError> {
        self.load(id)?;
        let key = session_storage_key(id)?;
        self.storage
            .remove_item(&key)
            .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?;
        notify_session_change();
        Ok(())
    }

    pub fn clear_invalid_session(&self, id: &str) -> Result<(), PracticeSessionError> {
        let key = session_storage_key(id)?;
        match self
            .storage
            .get_item(&key)
            .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?
        {
            None => Ok(()),
            Some(raw) if decode_stored_session(id, &raw).is_ok() => {
                Err(PracticeSessionError::OutOfDate)
            }
            Some(_) => {
                self.storage
                    .remove_item(&key)
                    .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?;
                notify_session_change();
                Ok(())
            }
        }
    }
}
