use crate::contexts::database::{DatabaseCommands, DatabaseStatus, Dictionary};
use yew::prelude::*;

#[hook]
pub fn use_dictionary() -> Dictionary {
    use_context::<Dictionary>().expect("Dictionary provider is missing; mount DatabaseProvider")
}

#[hook]
pub fn use_database_status() -> DatabaseStatus {
    use_context::<DatabaseStatus>()
        .expect("DatabaseStatus provider is missing; mount DatabaseProvider")
}

#[hook]
pub fn use_database_commands() -> DatabaseCommands {
    use_context::<DatabaseCommands>()
        .expect("DatabaseCommands provider is missing; mount DatabaseProvider")
}
