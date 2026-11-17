use web_sys::Document;

use crate::{
    AppEnvironment,
    appearance::{Density, Preset, update_background},
    font_preference::stored_font,
    preferences::PreferencesStore,
};

pub fn setup(environment: &AppEnvironment) {
    let storage = &environment.local_storage;
    let document = &environment.document;

    if let Err(error) = environment
        .theme_manager
        .apply(environment.theme_manager.load())
    {
        log::warn!("Could not apply initial theme: {error}");
    }

    let settings = PreferencesStore::new(storage).read();
    let preset = Preset::parse(&settings.preset).unwrap_or(Preset::Lavender);
    let density = Density::parse(&settings.density).unwrap_or(Density::Grid);

    let browser_document = &environment.browser_document;
    let font = stored_font(storage);
    let character_size = crate::font_preference::character_size(&settings.character_size);
    for (name, value) in [
        ("data-hanzi-font", font.as_str()),
        ("data-character-size", character_size),
        ("data-theme-preset", preset.key()),
        ("data-layout-density", density.key()),
        ("data-appearance-ready", "true"),
    ] {
        if let Err(error) = browser_document.set_root_attribute(name, value) {
            log::warn!("Could not apply {name}: {error:?}");
        }
    }
    update_background(document, preset);
}

pub fn show_fatal_error(message: &str) {
    web_sys::console::error_1(&message.into());
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    display_fatal_error(&document);
}

fn display_fatal_error(document: &Document) {
    if let Some(root) = document.get_element_by_id("root") {
        let _ = root.set_attribute("hidden", "");
    }
    if let Some(error) = document.get_element_by_id("error") {
        let _ = error.remove_attribute("hidden");
    }
}
