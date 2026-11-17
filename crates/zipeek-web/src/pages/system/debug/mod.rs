mod database_diagnostics;
mod sql;
mod sql_examples;
mod sql_terminal;

use database_diagnostics::DatabaseDiagnostics;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;

use crate::{
    appearance::{Density, Preset},
    hooks::use_app_environment,
    preferences::PreferencesStore,
    route::Route,
    services::ai,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DebugTab {
    Overview,
    Database,
    Sql,
}

#[function_component(DebugPage)]
pub fn debug_page() -> Html {
    if !cfg!(debug_assertions) {
        return html! {
            <Redirect<Route>
                to={Route::Home}
            />
        };
    }
    let tab = use_state(|| DebugTab::Overview);
    let environment = use_app_environment();
    let config = ai::configuration();
    let storage = &environment.local_storage;
    let preferences = PreferencesStore::new(storage).read();
    let preset = Preset::parse(&preferences.preset).unwrap_or(Preset::Lavender);
    let density = Density::parse(&preferences.density).unwrap_or(Density::Grid);
    let preferences_saved = storage
        .get_item(crate::preferences::KEY)
        .ok()
        .flatten()
        .is_some();

    html! {
        <section
            data-component="debug-page"
            class="mx-auto max-w-3xl space-y-5 rounded-xl border border-line bg-surface p-6"
        >
            <h2
                class="flex items-center gap-2 text-2xl font-bold"
            >
                <Icon
                    data={IconData::LUCIDE_INFO}
                    width="24"
                    height="24"
                />
                {"Development diagnostics"}
            </h2>
            <p
                class="text-sm text-muted"
            >
                {"Only exposed by debug builds. No API keys or personal records are displayed. Dictionary SQL results are available only when explicitly queried."}
            </p>
            <nav aria-label="Debug sections" data-debug-tabs="" class="flex flex-wrap gap-2 border-b border-line pb-4">
                {for [
                    (DebugTab::Overview, "Overview"),
                    (DebugTab::Database, "DuckDB"),
                    (DebugTab::Sql, "SQL terminal"),
                ].into_iter().map(|(choice, label)| {
                    let active = *tab == choice;
                    let tab = tab.clone();
                    let onclick = Callback::from(move |_: MouseEvent| tab.set(choice));
                    let tab_classes = classes!(
                        "min-h-10",
                        "rounded-lg",
                        "border",
                        "px-4",
                        "text-sm",
                        "font-semibold",
                        "focus-visible:outline-2",
                        "focus-visible:outline-accent",
                        if active { "border-accent bg-accent-soft" } else { "border-line" },
                    );
                    html! {
                        <button key={label} type="button" data-debug-tab={label} aria-pressed={active.to_string()} {onclick} class={tab_classes}>
                            {label}
                        </button>
                    }
                })}
            </nav>
            {if *tab == DebugTab::Overview { html! {
            <dl
                class="grid grid-cols-[minmax(8rem,1fr)_2fr] gap-3 text-sm"
            >
                <dt>
                    {"Build"}
                </dt>
                <dd>
                    {"Debug"}
                </dd>
                <dt>
                    {"AI endpoint"}
                </dt>
                <dd
                    class="break-all"
                >
                    {config.url}
                </dd>
                <dt>
                    {"Configured model"}
                </dt>
                <dd
                    class="break-all"
                >
                    {if config.model.is_empty() { "None" } else { config.model.as_str() }}
                </dd>
                <dt>
                    {"API key present"}
                </dt>
                <dd>
                    {if config.key_present { "Yes, tab memory" } else { "No" }}
                </dd>
                <dt>
                    {"Background preset"}
                </dt>
                <dd>
                    {preset.name()}
                </dd>
                <dt>
                    {"Layout density"}
                </dt>
                <dd>
                    {density.name()}
                </dd>
                <dt>
                    {"Preferences schema"}
                </dt>
                <dd>
                    {"v1 JSON"}
                </dd>
                <dt>
                    {"localStorage accessible"}
                </dt>
                <dd>
                    {"Yes"}
                </dd>
                <dt>
                    {"Preferences saved"}
                </dt>
                <dd>
                    {if preferences_saved { "Yes" } else { "No" }}
                </dd>
            </dl>
            } } else { Html::default() }}
            {if *tab == DebugTab::Database { html! { <DatabaseDiagnostics /> } } else { Html::default() }}
            {if *tab == DebugTab::Sql { html! { <sql_terminal::SqlTerminal /> } } else { Html::default() }}
            <p
                class="text-xs text-muted"
            >
                {"AI requests can be inspected through Settings > AI Translation > Test connection. Diagnostic failures are reported without provider credentials."}
            </p>
        </section>
    }
}
