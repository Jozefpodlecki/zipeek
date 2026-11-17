use crate::theme::{Theme, ThemeCommands};
use yew::prelude::*;

#[hook]
pub fn use_theme() -> Theme {
    use_context::<Theme>().expect("Theme provider is missing; mount ThemeProvider")
}

#[hook]
pub fn use_theme_commands() -> ThemeCommands {
    use_context::<ThemeCommands>().expect("ThemeCommands provider is missing; mount ThemeProvider")
}
