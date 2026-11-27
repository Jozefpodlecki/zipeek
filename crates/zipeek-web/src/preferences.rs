use serde::{Deserialize, Serialize};
use web_sys::Storage;

pub(crate) const KEY: &str = "zipeek.preferences.v1";

/// Persistent default for NEW dictionary Practice sessions. Existing sessions own their snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct HskPreferences {
    /// 2 = HSK 2.0, 3 = HSK 3.0, 0 = Mixed.
    pub standard: u8,
    pub bands: Vec<(u8, u8)>,
}

impl Default for HskPreferences {
    fn default() -> Self {
        Self {
            standard: 3,
            bands: vec![(3, 1)],
        }
    }
}

impl HskPreferences {
    pub fn selected_bands(
        &self,
    ) -> std::collections::BTreeSet<zipeek_exercises::practice_draft::HskBand> {
        use zipeek_exercises::practice_draft::HskBand;
        self.bands
            .iter()
            .filter_map(|&(system, level)| HskBand::new(system, level).ok())
            .filter(|band| self.standard == 0 || band.system == self.standard)
            .collect()
    }

    pub fn normalized(mut self) -> Self {
        if !matches!(self.standard, 0 | 2 | 3) {
            return Self::default();
        }
        let bands = self.selected_bands();
        self.bands = bands.iter().map(|band| (band.system, band.level)).collect();
        if self.bands.is_empty() {
            let system = if self.standard == 2 { 2 } else { 3 };
            self.bands.push((system, 1));
        }
        self
    }

    pub fn select_standard(&mut self, standard: u8) {
        if !matches!(standard, 0 | 2 | 3) {
            return;
        }
        self.standard = standard;
        *self = self.clone().normalized();
    }

    pub fn toggle(&mut self, band: zipeek_exercises::practice_draft::HskBand) {
        let key = (band.system, band.level);
        if self.standard != 0 && band.system != self.standard {
            return;
        }
        if self.bands.contains(&key) {
            // Do not leave the preference with no levels.
            if self.selected_bands().len() == 1 {
                return;
            }
            self.bands.retain(|value| *value != key);
        } else {
            self.bands.push(key);
        }
        *self = self.clone().normalized();
    }
}

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
    pub hsk_defaults: HskPreferences,
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
            hsk_defaults: HskPreferences::default(),
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
        assert_eq!(preferences.hsk_defaults, HskPreferences::default());
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

#[cfg(test)]
mod hsk_tests {
    use super::*;
    use zipeek_exercises::practice_draft::HskBand;
    #[test]
    fn mixed_preserves_two_standards_and_rejects_empty_selection() {
        let mut choice = HskPreferences::default();
        choice.select_standard(0);
        choice.toggle(HskBand::new(2, 5).unwrap());
        assert_eq!(choice.selected_bands().len(), 2);
        choice.select_standard(2);
        assert_eq!(choice.selected_bands().len(), 1);
        choice.toggle(HskBand::new(2, 5).unwrap());
        assert_eq!(choice.selected_bands().len(), 1);
    }
    #[test]
    fn hsk_defaults_round_trip_through_existing_preferences() {
        let mut preferences = Preferences::default();
        preferences.hsk_defaults.select_standard(3);
        preferences.hsk_defaults.toggle(HskBand::new(3, 6).unwrap());
        let restored = Preferences::parse(&serde_json::to_string(&preferences).unwrap()).unwrap();
        assert!(
            restored
                .hsk_defaults
                .selected_bands()
                .contains(&HskBand::new(3, 6).unwrap())
        );
        assert_eq!(restored.hsk_defaults.standard, 3);
    }

    #[test]
    fn corrupted_preferences_recover_to_valid_selection() {
        let prefs = HskPreferences {
            standard: 7,
            bands: vec![(9, 9)],
        }
        .normalized();
        assert_eq!(prefs, HskPreferences::default());
    }
}
