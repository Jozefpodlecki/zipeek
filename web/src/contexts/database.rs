use crate::services::database::Database;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum DatabaseState {
    Loading,
    Ready(Database),
    Failed(String),
}

#[derive(Clone, PartialEq)]
pub struct DatabaseContext {
    pub state: UseStateHandle<DatabaseState>,
    pub retry: Callback<()>,
}
