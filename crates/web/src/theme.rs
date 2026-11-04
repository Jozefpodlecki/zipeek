use web_sys::{Document, Storage};
use yew::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn toggled(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    pub fn is_dark(self) -> bool {
        self == Self::Dark
    }

    pub fn load(storage: Option<&Storage>) -> Self {
        let Some(storage) = storage else {
            return Self::default();
        };
        match storage.get_item("dark") {
            Ok(None) => Self::default(),
            Ok(Some(value)) if value == "true" => Self::Dark,
            Ok(Some(value)) if value == "false" => Self::Light,
            Ok(Some(value)) => {
                log::warn!("Invalid saved theme {value:?}; using dark");
                Self::default()
            }
            Err(error) => {
                log::warn!("Could not read saved theme; using dark: {error:?}");
                Self::default()
            }
        }
    }

    pub(crate) fn save(self, storage: Option<&Storage>) {
        let Some(storage) = storage else {
            log::warn!("Theme preference cannot persist because storage is unavailable");
            return;
        };
        if let Err(error) = storage.set_item("dark", if self.is_dark() { "true" } else { "false" })
        {
            log::warn!("Could not save theme preference: {error:?}");
        }
    }

    pub fn apply(self, document: &Document) {
        if let Some(root) = document.document_element()
            && let Err(error) = root.class_list().toggle_with_force("dark", self.is_dark())
        {
            log::warn!("Could not apply document theme: {error:?}");
        }
        if let Some(body) = document.body()
            && let Err(error) = body.class_list().toggle_with_force("dark", self.is_dark())
        {
            log::warn!("Could not apply body theme: {error:?}");
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ThemeCommands(Callback<()>);

impl ThemeCommands {
    pub(crate) fn new(toggle: Callback<()>) -> Self {
        Self(toggle)
    }

    pub fn toggle(&self) {
        self.0.emit(());
    }
}
