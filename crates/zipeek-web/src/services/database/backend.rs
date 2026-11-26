use std::cell::OnceCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatabaseBackend {
    Browser,
    Native,
}

thread_local! {
    static BACKEND: OnceCell<DatabaseBackend> = const { OnceCell::new() };
}

pub fn initialize(backend: DatabaseBackend) -> Result<(), String> {
    BACKEND.with(|selected| match selected.get() {
        Some(current) if *current == backend => Ok(()),
        Some(_) => Err("Database backend cannot change during application lifetime".into()),
        None => selected
            .set(backend)
            .map_err(|_| "Database backend initialization failed".into()),
    })
}

pub fn selected() -> Result<DatabaseBackend, String> {
    BACKEND.with(|selected| {
        selected
            .get()
            .copied()
            .ok_or_else(|| "Database backend is not initialized".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_selection_requires_initialization_and_cannot_change() {
        assert!(selected().is_err());
        initialize(DatabaseBackend::Native).unwrap();
        assert_eq!(selected().unwrap(), DatabaseBackend::Native);
        initialize(DatabaseBackend::Native).unwrap();
        assert!(initialize(DatabaseBackend::Browser).is_err());
        assert_eq!(selected().unwrap(), DatabaseBackend::Native);
    }
}
