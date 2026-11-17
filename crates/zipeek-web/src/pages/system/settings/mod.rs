use std::collections::HashMap;

use wasm_bindgen::JsCast;
use yew_router::prelude::{use_location, use_navigator};
mod ai_state;
mod appearance;
mod image_server;
mod learning_data;
mod native_services;
mod navigation;
mod speech;

use crate::hooks::{use_ai_service, use_app_environment, use_image_service};
use crate::services::ai;
use ai_state::{AiSettingsAction, AiSettingsState};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::font_preference::{
    CHARACTER_SIZES, HANZI_FONTS, character_size, font_name, stored_font,
};
use crate::preferences::PreferencesStore;

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let environment = use_app_environment();
    let ai_service = use_ai_service();
    let image_service = use_image_service();
    let initial_storage = environment.local_storage.clone();
    let font = use_state(move || stored_font(&initial_storage));
    let size_storage = environment.local_storage.clone();
    let character_size_value = use_state(move || {
        character_size(&PreferencesStore::new(&size_storage).read().character_size).to_owned()
    });
    let on_character_size = {
        let character_size_value = character_size_value.clone();
        let document = environment.browser_document.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |selected: String| {
            if character_size(&selected) != selected {
                return;
            }
            if let Err(error) = document.set_root_attribute("data-character-size", &selected) {
                log::warn!("Could not apply character size: {error:?}");
                return;
            }
            if let Err(error) = PreferencesStore::new(&storage).update(|settings| {
                settings.character_size = selected.clone();
            }) {
                log::warn!("Could not save character size: {error}");
                return;
            }
            character_size_value.set(selected);
        })
    };
    let audio_storage = environment.local_storage.clone();
    let matching_autoplay =
        use_state(move || crate::services::practice_audio::autoplay_enabled(&audio_storage));
    let on_matching_autoplay = {
        let matching_autoplay = matching_autoplay.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |event: Event| {
            let selected = event.target_unchecked_into::<HtmlInputElement>().checked();
            match crate::services::practice_audio::set_autoplay(&storage, selected) {
                Ok(()) => matching_autoplay.set(selected),
                Err(error) => log::warn!("Could not save Matching audio preference: {error:?}"),
            }
        })
    };
    let location = use_location();
    let navigator = use_navigator();
    let section = location
        .as_ref()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .and_then(|params| params.get("section").cloned())
        .filter(|section| navigation::is_valid_section(section))
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
    let on_font = {
        let font = font.clone();
        let browser_document = environment.browser_document.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |selected: String| {
            if !HANZI_FONTS.iter().any(|option| option.key == selected) {
                return;
            }
            if let Err(error) = browser_document.set_root_attribute("data-hanzi-font", &selected) {
                log::warn!("Could not apply Chinese font: {error:?}");
            }
            if let Err(error) = PreferencesStore::new(&storage).update(|settings| {
                settings.font = selected.clone();
            }) {
                log::warn!("Chinese font preference was not saved: {error}");
            }
            font.set(selected);
        })
    };
    let ai_state = use_reducer(|| AiSettingsState::new(ai::configuration()));

    let test_epoch = use_mut_ref(|| 0_u64);
    let on_url = {
        let state = ai_state.clone();
        let epoch = test_epoch.clone();
        Callback::from(move |event: InputEvent| {
            *epoch.borrow_mut() += 1;
            state.dispatch(AiSettingsAction::UrlChanged(
                event.target_unchecked_into::<HtmlInputElement>().value(),
            ));
        })
    };
    let on_model = {
        let state = ai_state.clone();
        let epoch = test_epoch.clone();
        Callback::from(move |event: InputEvent| {
            *epoch.borrow_mut() += 1;
            state.dispatch(AiSettingsAction::ModelChanged(
                event.target_unchecked_into::<HtmlInputElement>().value(),
            ));
        })
    };
    let on_key = {
        let state = ai_state.clone();
        let epoch = test_epoch.clone();
        Callback::from(move |event: InputEvent| {
            *epoch.borrow_mut() += 1;
            state.dispatch(AiSettingsAction::KeyChanged(
                event.target_unchecked_into::<HtmlInputElement>().value(),
            ));
        })
    };
    let on_ai_save = {
        let state = ai_state.clone();
        let epoch = test_epoch.clone();
        Callback::from(move |_: MouseEvent| {
            *epoch.borrow_mut() += 1;
            match ai::save_configuration(&state.url, &state.model, &state.key) {
                Ok(()) => {
                    crate::contexts::service_management::notify_configuration_changed();
                    state.dispatch(AiSettingsAction::Saved {
                        has_key: !state.key.is_empty(),
                    });
                }
                Err(message) => state.dispatch(AiSettingsAction::SaveFailed(message)),
            }
        })
    };
    let on_ai_clear_key = {
        let state = ai_state.clone();
        let epoch = test_epoch.clone();
        Callback::from(move |_: MouseEvent| {
            *epoch.borrow_mut() += 1;
            ai::clear_key();
            state.dispatch(AiSettingsAction::KeyCleared);
        })
    };
    let on_ai_test = {
        let state = ai_state.clone();
        let epoch = test_epoch.clone();
        Callback::from(move |_: MouseEvent| {
            if state.test_busy {
                return;
            }
            let url = state.url.clone();
            let model = state.model.clone();
            let ticket = {
                let mut current = epoch.borrow_mut();
                *current += 1;
                *current
            };
            state.dispatch(AiSettingsAction::BeginTest);
            let state = state.clone();
            let epoch = epoch.clone();
            spawn_local(async move {
                let response = ai::test_connection_details(&url, &model).await;
                if *epoch.borrow() != ticket {
                    return;
                }
                match response {
                    Ok(trace) => state.dispatch(AiSettingsAction::TestSucceeded(trace)),
                    Err(message) => state.dispatch(AiSettingsAction::TestFailed(message)),
                }
            });
        })
    };
    {
        let epoch = test_epoch.clone();
        use_effect_with((), move |_| {
            move || {
                *epoch.borrow_mut() += 1;
            }
        });
    }
    let select_tab = {
        let active_tab = active_tab.clone();
        let navigator = navigator.clone();
        Callback::from(move |name: String| {
            if !navigation::is_valid_section(&name) {
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
    let ai_url = &ai_state.url;
    let ai_model = &ai_state.model;
    let ai_key = &ai_state.key;
    let ai_message = &ai_state.message;
    let ai_has_key = &ai_state.has_key;
    let ai_test_busy = &ai_state.test_busy;
    let ai_trace = &ai_state.trace;
    let ai_test_ok = &ai_state.test_ok;
    let ai_diagnostic_status = if !ai_message.is_empty() {
        let element_classes_1 = classes!(
            "rounded-xl",
            "border",
            "px-4",
            "py-3",
            "text-sm",
            if *ai_test_ok == Some(true) {
                "border-emerald-500/60"
            } else if *ai_test_ok == Some(false) {
                "border-red-500/60"
            } else {
                "border-line"
            },
        );
        html! {
            <section
                data-ai-diagnostic-status=""
                role={if *ai_test_ok == Some(false) {"alert"} else {"status"}}
                class={element_classes_1}
            >
                <p
                    class="font-semibold"
                >
                    {if *ai_test_ok == Some(true) {"✓ Connected"} else if *ai_test_ok == Some(false) {"Connection failed"} else {"AI configuration"}}
                </p>
                <p
                    class="mt-1 text-muted"
                >
                    {ai_message.clone()}
                </p>
            </section>
        }
    } else {
        Html::default()
    };
    let ai_diagnostic_trace = if let Some(trace) = &*ai_trace {
        html! {
            <details
                data-ai-connection-trace=""
                class="rounded-xl border border-line bg-raised p-4 text-sm"
            >
                <summary
                    class="cursor-pointer font-semibold"
                >
                    {"Inspect diagnostic request and response"}
                </summary>
                <p
                    class="mt-3 text-xs text-muted"
                >
                    {"Sent to the selected model. The API key and Authorization header are never included here."}
                </p>
                <p
                    class="mt-2 font-semibold"
                >
                    {format!("Model: {}", trace.model)}
                </p>
                {for trace.messages.iter().map(|message| html! {
                    <div
                        class="mt-2"
                    >
                        <p
                            class="text-xs font-semibold text-muted"
                        >
                        {&message.role}
                        </p><pre
                            class="whitespace-pre-wrap break-words text-xs"
                        >
                            {&message.content}
                        </pre>
                        </div>
                })}
                <p
                    class="mt-3 text-xs font-semibold text-muted"
                >
                    {"Model reply"}
                </p>
                <pre
                    class="whitespace-pre-wrap break-words text-xs"
                >
                    {&trace.reply}
                </pre>
                {if trace.truncated {html! {
                    <p
                        class="text-xs text-muted"
                    >
                        {"Reply truncated to 600 characters."}
                    </p>}} else {Html::default()}}
            </details>
        }
    } else {
        Html::default()
    };
    let active_settings_panel = {
        let ai_service_controls =
            html! { <native_services::NativeServicePanel service={ai_service.clone()} /> };
        let image_service_controls =
            html! { <native_services::NativeServicePanel service={image_service.clone()} /> };
        match active_tab.as_str() {
            "appearance" => html! { <appearance::AppearanceSettings /> },
            "practice" => html! {
                <section
                    data-settings-practice=""
                    class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8"
                >
                    <h2
                        class="text-xl font-semibold"
                    >
                        {"Practice preferences"}
                    </h2>
                    <div
                        class="border-t border-line pt-4"
                        data-settings-matching-audio=""
                    >
                        <h3
                            class="text-sm font-semibold"
                        >
                            {"Matching exercise audio"}
                        </h3>
                        <label
                            class="mt-3 flex items-center gap-3 text-sm"
                        >
                            <input
                                type="checkbox"
                                class="size-4 accent-brand-action"
                                checked={*matching_autoplay}
                                onchange={on_matching_autoplay.clone()}
                            />
                            {"Play pronunciation automatically when selecting a matching card"}
                        </label>
                        <p
                            class="mt-2 text-xs text-muted"
                        >
                            {"Off by default. Audio remains available through the speaker button."}
                        </p>
                    </div>
                </section>
            },
            "characters" => html! { <div
                                            data-settings-characters=""
                                        >
                        <div
                            class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8"
                        >
                            <h2
                                class="text-xl font-semibold"
                            >
                                {"Chinese characters"}
                            </h2>
                            <p
                                class="mt-2 text-sm text-muted"
                            >
                                {"Choose a Chinese typeface. Additional fonts require local installation; missing fonts use a readable fallback."}
                            </p>
                            <p
                                id="hanzi-font-label"
                                class="mt-6 block text-sm font-semibold"
                            >
                                {"Character font"}
                            </p>
                            <div
                                role="group"
                                aria-labelledby="hanzi-font-label"
                                data-hanzi-font-options=""
                                class="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-5"
                            >
                                {for HANZI_FONTS.iter().map(|option| {
                                    let value = option.key;
                                    let label = option.name;
                                    let sample = "你好";
                                    let selected = font.as_str() == value;
                                    let choose = on_font.clone();
                                    let on_choose_font = Callback::from(move |_| choose.emit(value.to_owned()));
                                    let element_classes_2 = classes!(
                                                    "flex",
                                                    "min-h-20",
                                                    "flex-col",
                                                    "items-center",
                                                    "justify-center",
                                                    "gap-1",
                                                    "rounded-xl",
                                                    "border",
                                                    "px-2",
                                                    "py-2",
                                                    "text-sm",
                                                    if selected {"border-accent bg-accent-soft text-ink ring-1 ring-accent/40"} else {"border-line bg-raised hover:border-accent"},
                                                );
                                    html! {
                                        <button
                                            key={value}
                                            type="button"
                                            data-hanzi-font-option={value}
                                            aria-pressed={selected.to_string()}
                                            onclick={on_choose_font}
                                            class={element_classes_2}
                                        >
                                        <span
                                            lang="zh-Hans"
                                            data-font-preview={value}
                                            class="text-2xl"
                                        >
                                            {sample}
                                        </span><span
                                                class="font-semibold"
                                            >
                                            {label}
                                            </span>
                                    </button>}
                                })}
                            </div>
                            <p
                                class="mt-2 text-xs text-muted"
                                role="status"
                            >
                                {format!("Selected: {}. This choice is saved in this browser.", font_name(&font))}
                            </p>
                            <fieldset
                                class="mt-5 space-y-2"
                                data-character-size-settings=""
                            >
                                <legend class="text-sm font-semibold">
                                    {"Practice character size"}
                                </legend>
                                <div class="flex flex-wrap gap-2">
                                    {for CHARACTER_SIZES.iter().map(|(key, label)| {
                                        let selected = *key == character_size_value.as_str();
                                        let value = (*key).to_owned();
                                        let on_character_size = on_character_size.clone();
                                        let on_choose_size = Callback::from(move |_| {
                                            on_character_size.emit(value.clone());
                                        });
                                        let element_classes_3 = classes!(
                                                    "rounded-lg",
                                                    "border",
                                                    "px-4",
                                                    "py-2",
                                                    if selected {
                                                        "border-accent bg-accent-soft"
                                                    } else {
                                                        "border-line bg-raised"
                                                    }
                                                );
                                        html! {
                                            <button
                                                key={*key}
                                                type="button"
                                                data-character-size-option={*key}
                                                aria-pressed={selected.to_string()}
                                                class={element_classes_3}
                                                onclick={on_choose_size}
                                            >
                                                {*label}
                                            </button>
                                        }
                                    })}
                                </div>
                                <p class="text-xs text-muted">
                                    {"Changes the Hanzi tiles in Character Builder, including their selected slots."}
                                </p>
                            </fieldset>
                            <div
                                class="mt-6 rounded-xl border border-line bg-raised p-6 text-center"
                            >
                                <span
                                    class="block text-xs font-semibold uppercase tracking-widest text-muted"
                                >
                                    {"Preview"}
                                </span>
                                <p
                                    lang="zh-Hans"
                                    class="mt-4 text-5xl font-medium leading-relaxed sm:text-6xl"
                                >
                                    {"你好世界 · 学习中文"}
                                </p>
                                <p
                                    lang="zh-Hant"
                                    class="mt-3 text-2xl text-muted"
                                >
                                    {"繁體中文"}
                                </p>
                            </div>
                        </div>
            </div> },
            "ai" => html! { <div
                                            data-settings-ai=""
                                        >
                        <div
                            data-component="ai-settings"
                            class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8"
                        >
                            <h2
                                class="text-xl font-semibold"
                            >
                                {"AI translation"}
                            </h2>
                            <p
                                class="mt-2 text-sm text-muted"
                            >
                                {"Connect an OpenAI-compatible chat completions endpoint, such as LM Studio or a compatible hosted provider. AI is optional."}
                            </p>
                            <div
                                class="mt-5 grid gap-4"
                            >
                                <label
                                    class="flex flex-col gap-2 text-sm font-semibold"
                                >
                                    {"API base URL"}
                                    <input
                                        data-ai-base-url=""
                                        type="url"
                                        value={ai_url.clone()}
                                        oninput={on_url}
                                        placeholder="http://127.0.0.1:1234/v1/"
                                        class="min-h-11 rounded-lg border border-line bg-raised px-3 font-normal text-ink"
                                    />
                                </label>
                                <label
                                    class="flex flex-col gap-2 text-sm font-semibold"
                                >
                                    {"Model ID"}
                                    <input
                                        data-ai-model-setting=""
                                        value={ai_model.clone()}
                                        oninput={on_model}
                                        placeholder="Loaded model identifier"
                                        class="min-h-11 rounded-lg border border-line bg-raised px-3 font-normal text-ink"
                                    />
                                </label>
                                <label
                                    class="flex flex-col gap-2 text-sm font-semibold"
                                >
                                    {"API key (optional for local models)"}
                                    <input
                                        data-ai-api-key=""
                                        type="password"
                                        autocomplete="off"
                                        value={ai_key.clone()}
                                        oninput={on_key}
                                        placeholder={if *ai_has_key {"Key is set for this tab — enter to replace"} else {"Leave blank for local model"}}
                                        class="min-h-11 rounded-lg border border-line bg-raised px-3 font-normal text-ink"
                                    />
                                </label>
                                <p
                                    class="text-xs text-muted"
                                >
                                    {"URL and model are saved in this browser. API keys are not stored in localStorage, OPFS or the personal database; they are kept in page memory and lost on reload. Browser requests expose the key to the endpoint and may be blocked by CORS. Never use a provider's secret administrative key in a public browser app."}
                                </p>
                                <div
                                    class="flex flex-wrap gap-2"
                                >
                                    <button
                                        type="button"
                                        data-ai-save=""
                                        onclick={on_ai_save}
                                        class="min-h-11 rounded-lg border border-brand-action bg-brand-action px-5 font-semibold text-white hover:bg-brand-hover"
                                    >
                                        {"Save AI settings"}
                                    </button>
                                    <button
                                        type="button"
                                        data-ai-test-connection=""
                                        onclick={on_ai_test}
                                        disabled={*ai_test_busy || ai_model.trim().is_empty()}
                                        class="min-h-11 rounded-lg border border-line px-4 hover:bg-raised disabled:opacity-50"
                                    >
                                        {if *ai_test_busy {"Testing…"} else {"Test connection"}}
                                    </button>
                                    <button
                                        type="button"
                                        onclick={on_ai_clear_key}
                                        class="min-h-11 rounded-lg border border-line px-4 hover:bg-raised"
                                    >
                                        {"Clear API key"}
                                    </button>
                                </div>
                                <p
                                    class="text-xs text-muted"
                                >
                                    {"Connection test sends a short prompt to the selected model and may incur provider usage charges. Save your key first; the test uses the currently entered URL and model."}
                                </p>
                                {ai_diagnostic_status}
                                {ai_diagnostic_trace}
                                {ai_service_controls}
                            </div>
                        </div>
            </div> },
            "images" => {
                html! { <> {image_service_controls} <image_server::ImageServerSettings /> </> }
            }
            "learning" => html! { <learning_data::LearningDataSettings /> },
            "speech" => html! { <speech::SpeechSettings /> },
            _ => Html::default(),
        }
    };
    html! {
        <section
            data-component="settings-page"
            class="mx-auto flex w-full max-w-7xl flex-col gap-4 py-2"
            data-settings-tab={(*active_tab).clone()}
        >
            <div
                class="grid min-h-0 gap-4 lg:grid-cols-[12rem_minmax(0,1fr)]"
            >
                {navigation::view(active_tab.as_str(), select_tab)}
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
