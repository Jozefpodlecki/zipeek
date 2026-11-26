use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "zipeek.time_weaver.options.v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum TileSize {
    Compact,
    Standard,
    #[default]
    Large,
}

impl TileSize {
    pub(super) fn font_class(self) -> &'static str {
        match self {
            Self::Compact => "text-xl",
            Self::Standard => "text-[1.375rem]",
            Self::Large => "text-2xl",
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Standard => "Standard",
            Self::Large => "Large",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GameOptions {
    pub(super) show_pinyin: bool,
    pub(super) audio_on_click: bool,
    pub(super) tile_size: TileSize,
    pub(super) question_count: usize,
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            show_pinyin: true,
            audio_on_click: false,
            tile_size: TileSize::Large,
            question_count: 15,
        }
    }
}

impl GameOptions {
    fn valid(self) -> bool {
        matches!(self.question_count, 10 | 15 | 20)
    }

    pub(super) fn load(storage: &web_sys::Storage) -> Self {
        storage
            .get_item(STORAGE_KEY)
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str::<Self>(&json).ok())
            .filter(|settings| settings.valid())
            .unwrap_or_default()
    }

    pub(super) fn save(self, storage: &web_sys::Storage) {
        if !self.valid() {
            return;
        }
        match serde_json::to_string(&self) {
            Ok(json) => {
                if storage.set_item(STORAGE_KEY, &json).is_err() {
                    log::warn!("Unable to store Time Weaver preferences");
                }
            }
            Err(error) => log::warn!("Unable to serialize Time Weaver preferences: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip_and_reject_invalid_length() {
        let options = GameOptions {
            question_count: 20,
            tile_size: TileSize::Compact,
            show_pinyin: false,
            audio_on_click: true,
        };
        let raw = serde_json::to_string(&options).unwrap();
        assert_eq!(serde_json::from_str::<GameOptions>(&raw).unwrap(), options);
        assert!(
            !GameOptions {
                question_count: 11,
                ..options
            }
            .valid()
        );
        assert!(serde_json::from_str::<GameOptions>("{\"question_count\":0}").is_err());
    }
}
