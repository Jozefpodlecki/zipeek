mod browse;
mod database;
mod exclusions;
mod service_management;
mod speech;
mod theme;
mod word_sets;

pub(crate) use browse::use_browse_state;
pub use database::{use_database_commands, use_database_status, use_dictionary};
pub(crate) use exclusions::{SavedExclusions, use_saved_exclusions};
pub use service_management::{use_ai_service, use_image_service};
pub use speech::{use_stt_service, use_tts_service};
pub use theme::{use_theme, use_theme_commands};
pub(crate) use word_sets::use_word_sets;

use yew::prelude::*;

#[hook]
pub fn use_app_environment() -> crate::AppEnvironment {
    use_context::<crate::AppEnvironment>()
        .expect("AppEnvironment provider must wrap the application")
}
