use crate::{
    contexts::database::{DatabaseCommands, DatabaseStatus, Dictionary},
    theme::{Theme, ThemeCommands},
};
use yew::prelude::*;
mod exclusions;
mod practice;
pub(crate) use exclusions::{SavedExclusions, use_saved_exclusions};
pub(crate) use practice::use_prepared_practice;

#[hook]
pub fn use_dictionary() -> Option<Dictionary> {
    use_context::<Dictionary>()
}

#[hook]
pub fn use_database_status() -> Option<DatabaseStatus> {
    use_context::<DatabaseStatus>()
}

#[hook]
pub fn use_database_commands() -> Option<DatabaseCommands> {
    use_context::<DatabaseCommands>()
}

#[hook]
pub fn use_theme() -> Option<Theme> {
    use_context::<Theme>()
}

#[hook]
pub fn use_theme_commands() -> Option<ThemeCommands> {
    use_context::<ThemeCommands>()
}
