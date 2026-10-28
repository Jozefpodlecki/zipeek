use crate::services::database::{Database, DatabaseInfo, DbError};
use std::{cell::RefCell, rc::Rc};
use yew::Callback;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseStatus {
    Loading,
    Ready(DatabaseInfo),
    Failed(String),
}

#[derive(Clone)]
pub struct Dictionary(Rc<RefCell<Option<Database>>>);

impl Dictionary {
    pub fn database(&self) -> Result<Database, DbError> {
        self.0.borrow().clone().ok_or(DbError::NotReady)
    }

    pub(crate) fn new(slot: Rc<RefCell<Option<Database>>>) -> Self {
        Self(slot)
    }
}

impl PartialEq for Dictionary {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, PartialEq)]
pub struct DatabaseCommands(Callback<()>);

impl DatabaseCommands {
    pub(crate) fn new(retry: Callback<()>) -> Self {
        Self(retry)
    }

    pub fn retry(&self) {
        self.0.emit(());
    }
}
