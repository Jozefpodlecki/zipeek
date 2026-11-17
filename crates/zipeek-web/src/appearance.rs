use web_sys::{Document, Storage};
use yew::prelude::*;
use zipeek_browser::document::BrowserDocument;

use crate::preferences::PreferencesStore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Lavender,
    Jade,
    Sunset,
    Midnight,
    Ink,
    Maple,
}

impl Preset {
    pub const ALL: [Self; 6] = [
        Self::Lavender,
        Self::Jade,
        Self::Sunset,
        Self::Midnight,
        Self::Ink,
        Self::Maple,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Lavender => "lavender",
            Self::Jade => "jade",
            Self::Sunset => "sunset",
            Self::Midnight => "midnight",
            Self::Ink => "ink",
            Self::Maple => "maple",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Lavender => "Lavender Classic",
            Self::Jade => "Jade Light",
            Self::Sunset => "Sunset Warm",
            Self::Midnight => "Midnight Blue",
            Self::Ink => "Ink Minimal",
            Self::Maple => "Autumn Maple",
        }
    }

    pub const fn background(self) -> &'static str {
        match self {
            Self::Lavender => "lavender.webp",
            Self::Jade => "jade.webp",
            Self::Sunset => "sunset.webp",
            Self::Midnight => "midnight.webp",
            Self::Ink => "ink.webp",
            Self::Maple => "maple.webp",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Lavender => "Wisteria, lavender mist, spring mountain lake",
            Self::Jade => "Bamboo forest, green mountain valley, morning light",
            Self::Sunset => "Peach sunset, mountain lake, waterside pavilion",
            Self::Midnight => "Moonlit lake, indigo mountains, night sky",
            Self::Ink => "Monochrome brush painting, mist, negative space",
            Self::Maple => "Golden maples, mountain stream, stone bridge",
        }
    }

    pub const fn colors(self) -> [&'static str; 3] {
        match self {
            Self::Lavender => ["#5e44c9", "#b7a5e9", "#e8e2fb"],
            Self::Jade => ["#117b59", "#83c5a8", "#daf2e5"],
            Self::Sunset => ["#bc4250", "#ec9272", "#ffe0c7"],
            Self::Midnight => ["#1d4ed8", "#3c74b6", "#172a4c"],
            Self::Ink => ["#313743", "#949ba7", "#e4e7ed"],
            Self::Maple => ["#a71e3e", "#e96d50", "#fbc6aa"],
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|preset| preset.key() == value)
    }
}

pub(crate) fn background_source(preset: Preset, current: &str) -> Option<String> {
    if preset == Preset::Lavender && current.starts_with("data:image/webp;base64,") {
        return None;
    }

    let source = format!("public/backgrounds/{}", preset.background());
    (source != current).then_some(source)
}

pub(crate) fn update_background(document: &Document, preset: Preset) {
    let Some(image) = document.get_element_by_id("app-background") else {
        return;
    };
    let current = image.get_attribute("src").unwrap_or_default();
    if let Some(source) = background_source(preset, &current) {
        if let Err(error) = image.set_attribute("src", &source) {
            log::warn!("Could not update the background image: {error:?}");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Density {
    Grid,
    Comfortable,
    Compact,
}

impl Density {
    pub const ALL: [Self; 3] = [Self::Grid, Self::Comfortable, Self::Compact];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Grid => "grid",
            Self::Comfortable => "comfortable",
            Self::Compact => "compact",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Grid => "Grid",
            Self::Comfortable => "Comfortable",
            Self::Compact => "Compact",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|density| density.key() == value)
    }
}

#[derive(Clone, PartialEq)]
pub struct Appearance {
    pub preset: Preset,
    pub density: Density,
    pub select_preset: Callback<Preset>,
    pub preview_preset: Callback<Preset>,
    pub select_density: Callback<Density>,
}

#[derive(Properties, PartialEq)]
pub struct AppearanceProviderProps {
    pub storage: Storage,
    pub document: Document,
    pub browser_document: BrowserDocument,
    pub children: Children,
}

#[function_component(AppearanceProvider)]
pub fn appearance_provider(props: &AppearanceProviderProps) -> Html {
    let initial_storage = props.storage.clone();
    let preset = use_state(move || {
        Preset::parse(&PreferencesStore::new(&initial_storage).read().preset)
            .unwrap_or(Preset::Lavender)
    });
    let initial_storage = props.storage.clone();
    let density = use_state(move || {
        Density::parse(&PreferencesStore::new(&initial_storage).read().density)
            .unwrap_or(Density::Grid)
    });
    {
        let browser_document = props.browser_document.clone();
        use_effect_with((*preset, *density), move |(selected, layout)| {
            for (name, value) in [
                ("data-theme-preset", selected.key()),
                ("data-layout-density", layout.key()),
            ] {
                if let Err(error) = browser_document.set_root_attribute(name, value) {
                    log::warn!("Could not set {name}: {error:?}");
                }
            }
            update_background(browser_document.document(), *selected);
        });
    }
    let select_preset = {
        let preset = preset.clone();
        let storage = props.storage.clone();
        Callback::from(move |new_preset: Preset| {
            if let Err(error) = PreferencesStore::new(&storage).update(|settings| {
                settings.preset = new_preset.key().to_owned();
            }) {
                log::warn!("Could not save background preset: {error}");
            }
            preset.set(new_preset);
        })
    };
    let preview_preset = {
        let preset = preset.clone();
        Callback::from(move |new_preset: Preset| preset.set(new_preset))
    };
    let select_density = {
        let density = density.clone();
        let storage = props.storage.clone();
        Callback::from(move |new_density: Density| {
            if let Err(error) = PreferencesStore::new(&storage).update(|settings| {
                settings.density = new_density.key().to_owned();
            }) {
                log::warn!("Could not save layout density: {error}");
            }
            density.set(new_density);
        })
    };
    html! {
        <ContextProvider<Appearance> context={Appearance { preset: *preset, density: *density, select_preset, preview_preset, select_density }}>
            {props.children.clone()}
        </ContextProvider<Appearance>>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_have_stable_distinct_keys() {
        let keys: Vec<_> = Preset::ALL.iter().map(|preset| preset.key()).collect();
        assert_eq!(keys.len(), 6);
        for key in keys {
            assert_eq!(Preset::parse(key).unwrap().key(), key);
        }
    }
    #[test]
    fn each_preset_has_a_distinct_background() {
        let names: std::collections::HashSet<_> = Preset::ALL
            .iter()
            .map(|preset| preset.background())
            .collect();
        assert_eq!(names.len(), Preset::ALL.len());
    }

    #[test]
    fn embedded_lavender_remains_the_first_paint_background() {
        assert_eq!(
            background_source(Preset::Lavender, "data:image/webp;base64,abc"),
            None
        );
    }

    #[test]
    fn saved_presets_replace_the_first_paint_background() {
        assert_eq!(
            background_source(Preset::Jade, "data:image/webp;base64,abc").as_deref(),
            Some("public/backgrounds/jade.webp")
        );
        assert_eq!(
            background_source(Preset::Jade, "public/backgrounds/jade.webp"),
            None
        );
    }

    #[test]
    fn density_rejects_invalid_values() {
        assert_eq!(Density::parse("unknown"), None);
        assert_eq!(Density::parse("compact"), Some(Density::Compact));
    }
}
