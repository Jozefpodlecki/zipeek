use super::{
    ai, appearance, characters, image_server, learning, navigation, practice, reset, services,
    speech,
};
use crate::hooks::use_app_environment;
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::{use_location, use_navigator};

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let environment = use_app_environment();
    let is_tauri = environment.is_tauri;
    let location = use_location();
    let navigator = use_navigator();
    let section = location
        .as_ref()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .and_then(|params| params.get("section").cloned())
        .filter(|section| navigation::is_valid_section(section, is_tauri))
        .unwrap_or_else(|| "appearance".to_owned());
    let active_tab = use_state(|| section.clone());
    {
        let active_tab = active_tab.clone();
        use_effect_with(section, move |section| {
            if *active_tab != *section {
                active_tab.set(section.clone());
            }
            || ()
        });
    }
    let select_tab = {
        let active_tab = active_tab.clone();
        let navigator = navigator.clone();
        Callback::from(move |name: String| {
            if !navigation::is_valid_section(&name, is_tauri) {
                return;
            }
            if let Some(navigator) = navigator.as_ref() {
                let params = HashMap::from([("section", name.clone())]);
                if let Err(error) =
                    navigator.push_with_query(&crate::route::Route::Settings, &params)
                {
                    log::warn!("Could not navigate to settings section: {error}");
                }
            }
            active_tab.set(name);
        })
    };
    let active_settings_panel = match active_tab.as_str() {
        "appearance" => html! { <appearance::AppearanceSettings /> },
        "characters" => html! { <characters::CharacterSettings /> },
        "practice" => html! { <practice::PracticeSettings /> },
        "ai" => html! { <ai::AiTranslationSettings /> },
        "images" => html! { <image_server::ImageServerSettings /> },
        "learning" => html! { <learning::LearningSettings /> },
        "speech" => html! { <speech::SpeechSettings /> },
        "services" if is_tauri => html! { <services::DesktopServicesSettings /> },
        "reset" => html! { <reset::ResetSettings /> },
        _ => Html::default(),
    };
    html! {
        <section
            data-component="settings-page"
            class="mx-auto flex w-full max-w-7xl flex-col justify-center gap-4 py-2 md:min-h-[calc(100dvh-var(--topbar-height)-3rem)]"
            data-settings-tab={(*active_tab).clone()}
        >
            <div
                class="grid min-h-0 gap-4 md:min-h-[44rem] lg:grid-cols-[12rem_minmax(0,1fr)] lg:items-start"
            >
                {navigation::view(active_tab.as_str(), select_tab, is_tauri)}
                <div
                    id="settings-active-panel"
                    class="min-w-0"
                    data-settings-active-panel=""
                    role="region"
                    aria-label="Selected settings section"
                >
                    {active_settings_panel}
                </div>
            </div>
        </section>
    }
}
