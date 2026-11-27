use serde::{Deserialize, Serialize};
use web_sys::Storage;

pub(crate) const KEY: &str = "zipeek.preferences.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Preferences {
    pub preset: String,
    pub density: String,
    pub dark: bool,
    pub font: String,
    pub character_size: String,
    pub matching_autoplay: bool,
    pub learning_history_enabled: bool,
    pub hidden_drawer_items: Vec<String>,
    pub image_server_url: Option<String>,
    pub ai_url: String,
    pub ai_model: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            preset: "lavender".to_owned(),
            density: "grid".to_owned(),
            dark: true,
            font: "system".to_owned(),
            character_size: "normal".to_owned(),
            matching_autoplay: false,
            learning_history_enabled: false,
            hidden_drawer_items: Vec::new(),
            image_server_url: None,
            ai_url: "http://127.0.0.1:1234/v1/".to_owned(),
            ai_model: String::new(),
        }
    }
}

impl Preferences {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

pub(crate) struct PreferencesStore<'a> {
    storage: &'a Storage,
}

impl<'a> PreferencesStore<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    pub fn read(&self) -> Preferences {
        match self.storage.get_item(KEY) {
            Ok(Some(raw)) => match Preferences::parse(&raw) {
                Ok(settings) => settings,
                Err(error) => {
                    log::warn!("Invalid preferences JSON, using defaults: {error}");
                    Preferences::default()
                }
            },
            Ok(None) => Preferences::default(),
            Err(error) => {
                log::warn!("Cannot read browser preferences: {error:?}");
                Preferences::default()
            }
        }
    }

    pub fn read_checked(&self) -> Result<Preferences, String> {
        match self.storage.get_item(KEY) {
            Ok(Some(raw)) => {
                Preferences::parse(&raw).map_err(|error| format!("Invalid preferences: {error}"))
            }
            Ok(None) => Ok(Preferences::default()),
            Err(error) => Err(format!("Cannot read preferences: {error:?}")),
        }
    }

    pub fn ensure_initialized(&self) -> Result<(), String> {
        match self.storage.get_item(KEY) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => self.write(&Preferences::default()),
            Err(error) => Err(format!("Cannot read preferences: {error:?}")),
        }
    }

    pub fn update(&self, change: impl FnOnce(&mut Preferences)) -> Result<(), String> {
        let mut settings = self.read_checked()?;
        change(&mut settings);
        self.write(&settings)
    }

    fn write(&self, settings: &Preferences) -> Result<(), String> {
        let json = serde_json::to_string(settings)
            .map_err(|error| format!("Cannot serialize preferences: {error}"))?;
        self.storage
            .set_item(KEY, &json)
            .map_err(|error| format!("Cannot save preferences: {error:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences_choose_lavender() {
        let preferences = Preferences::default();
        assert_eq!(preferences.preset, "lavender");
        assert!(preferences.dark);
        assert!(!preferences.learning_history_enabled);
    }

    #[test]
    fn settings_round_trip_as_one_json_document() {
        let preferences = Preferences {
            preset: "maple".into(),
            matching_autoplay: true,
            learning_history_enabled: true,
            image_server_url: Some("http://localhost:1431".into()),
            ..Preferences::default()
        };
        let serialized = serde_json::to_string(&preferences).unwrap();
        assert_eq!(Preferences::parse(&serialized).unwrap(), preferences);
    }

    #[test]
    fn missing_fields_receive_defaults() {
        let preferences = Preferences::parse(r#"{"preset":"jade"}"#).unwrap();
        assert_eq!(preferences.preset, "jade");
        assert_eq!(preferences.font, "system");
        assert_eq!(preferences.character_size, "normal");
        assert!(!preferences.learning_history_enabled);
    }

    #[test]
    fn recording_consent_round_trips_with_other_settings() {
        let mut settings = Preferences::default();
        settings.preset = "jade".into();
        settings.learning_history_enabled = true;
        let encoded = serde_json::to_string(&settings).unwrap();
        let decoded = Preferences::parse(&encoded).unwrap();
        assert_eq!(decoded.preset, "jade");
        assert!(decoded.learning_history_enabled);
    }

    #[test]
    fn hidden_drawer_items_round_trip() {
        let mut preferences = Preferences::default();
        preferences.hidden_drawer_items = vec!["/games".into(), "/record".into()];
        let json = serde_json::to_string(&preferences).unwrap();
        assert_eq!(Preferences::parse(&json).unwrap(), preferences);
    }

    #[test]
    fn malformed_settings_are_not_accepted() {
        assert!(Preferences::parse("not-json").is_err());
    }
}
