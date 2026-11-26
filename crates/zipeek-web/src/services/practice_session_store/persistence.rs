use super::codec::decode_stored_session;
use super::inventory::{
    PracticeSessionInventory, completed_removal_allowed, ensure_new_session_allowed,
    inventory_keys_match,
};
use super::{
    MAX_STORED_SESSION_BYTES, PracticeSession, PracticeSessionError, SESSION_KEY_PREFIX,
    notify_session_change, session_storage_key,
};
use web_sys::Storage;

#[derive(Clone)]
pub(crate) struct PracticeSessionStore {
    storage: Storage,
}

impl PracticeSessionStore {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    fn inventory_keys(&self) -> Result<Vec<String>, PracticeSessionError> {
        let count = self
            .storage
            .length()
            .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?;
        let mut keys = Vec::with_capacity(count as usize);
        for index in 0..count {
            let key = self
                .storage
                .key(index)
                .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?
                .ok_or(PracticeSessionError::OutOfDate)?;
            keys.push(key);
        }
        keys.sort_unstable();
        keys.dedup();
        if keys.len() != count as usize {
            return Err(PracticeSessionError::OutOfDate);
        }
        Ok(keys)
    }

    pub fn recoverable_sessions(&self) -> Result<PracticeSessionInventory, PracticeSessionError> {
        let keys = self.inventory_keys()?;
        let mut inventory = PracticeSessionInventory::default();
        for key in &keys {
            let Some(id) = key.strip_prefix(SESSION_KEY_PREFIX) else {
                continue;
            };
            let raw = self
                .storage
                .get_item(key)
                .map_err(|error| PracticeSessionError::Storage(format!("{error:?}")))?
                .ok_or(PracticeSessionError::OutOfDate)?;
            match decode_stored_session(id, &raw) {
                Ok(session) => inventory.sessions.push(session),
                Err(_) => inventory.invalid_records += 1,
            }
        }
        inventory_keys_match(&keys, &self.inventory_keys()?)?;
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
