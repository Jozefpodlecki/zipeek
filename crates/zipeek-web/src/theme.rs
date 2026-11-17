use web_sys::Storage;
use yew::prelude::*;
use zipeek_browser::document::{BrowserDocument, BrowserDocumentError};

use crate::preferences::PreferencesStore;

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

    pub fn load(storage: &Storage) -> Self {
        if PreferencesStore::new(storage).read().dark {
            Self::Dark
        } else {
            Self::Light
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeManager {
    browser_document: BrowserDocument,
    storage: Storage,
}

impl ThemeManager {
    pub fn new(browser_document: BrowserDocument, storage: Storage) -> Self {
        Self {
            browser_document,
            storage,
        }
    }

    pub fn browser_document(&self) -> &BrowserDocument {
        &self.browser_document
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn load(&self) -> Theme {
        Theme::load(&self.storage)
    }

    pub fn save(&self, theme: Theme) -> Result<(), String> {
        PreferencesStore::new(&self.storage).update(|settings| {
            settings.dark = theme.is_dark();
        })
    }

    pub fn apply(&self, theme: Theme) -> Result<(), BrowserDocumentError> {
        self.browser_document
            .toggle_root_class("dark", theme.is_dark())?;
        self.browser_document
            .toggle_body_class("dark", theme.is_dark())?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_theme_preserves_binary_states() {
        assert_eq!(Theme::Dark.toggled(), Theme::Light);
        assert_eq!(Theme::Light.toggled(), Theme::Dark);
    }
}
