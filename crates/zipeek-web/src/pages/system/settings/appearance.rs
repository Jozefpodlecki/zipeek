use yew::prelude::*;

use crate::appearance::{Appearance, Density, Preset};
use crate::hooks::{use_theme, use_theme_commands};

#[function_component(AppearanceSettings)]
pub(super) fn appearance_settings() -> Html {
    let appearance = use_context::<Appearance>()
        .expect("Appearance provider is missing; mount AppearanceProvider");
    let theme = use_theme();
    let theme_commands = use_theme_commands();

    let presets = Preset::ALL.into_iter().map(|preset| {
        let active = appearance.preset == preset;
        let colors = preset.colors();
        let choose = appearance.select_preset.clone();
        let onclick = Callback::from(move |_| choose.emit(preset));
        let element_classes_1 = classes!(
            "rounded-xl",
            "border",
            "p-3",
            "text-left",
            "text-sm",
            "focus-visible:outline-2",
            "focus-visible:outline-offset-2",
            "focus-visible:outline-accent",
            if active {
                "border-accent bg-accent-soft ring-1 ring-accent/40"
            } else {
                "border-line hover:bg-raised"
            },
        );
        html! {
            <button
                key={preset.key()}
                type="button"
                data-background-preset={preset.key()}
                aria-pressed={active.to_string()}
                {onclick}
                class={element_classes_1}
            >
                <img
                    src={format!("public/backgrounds/{}", preset.background())}
                    alt={format!("{}: {}", preset.name(), preset.description())}
                    loading="lazy"
                    decoding="async"
                    class="mb-2 h-24 w-full rounded-lg object-cover"
                />
                <div
                    class="mb-2 flex gap-1"
                >
                    {for colors.into_iter().map(|color| html! {
                        <span
                            class="h-5 flex-1 rounded"
                            style={format!("background:{color}")}
                        />
                    })}
                </div>
                <strong>
                    {preset.name()}
                </strong>
            </button>
        }
    });
    let modes = [(false, "Light"), (true, "Dark")]
        .into_iter()
        .map(|(dark, name)| {
            let is_selected = theme.is_dark() == dark;
            let commands = theme_commands.clone();
            let onclick = Callback::from(move |_| {
                if !is_selected {
                    commands.toggle();
                }
            });
            let element_classes_2 = classes!(
                "rounded-lg",
                "border",
                "px-5",
                "py-2",
                "text-sm",
                "focus-visible:outline-2",
                "focus-visible:outline-offset-2",
                "focus-visible:outline-accent",
                if is_selected {
                    "border-accent bg-accent-soft text-accent font-semibold"
                } else {
                    "border-line"
                },
            );
            html! {
                <button
                    key={name}
                    type="button"
                    aria-pressed={is_selected.to_string()}
                    class={element_classes_2}
                    {onclick}
                >
                    {name}
                </button>
            }
        });
    let densities = Density::ALL.into_iter().map(|density| {
        let selected = appearance.density == density;
        let change = appearance.select_density.clone();
        let onclick = Callback::from(move |_| change.emit(density));
        let element_classes_3 = classes!(
            "rounded-lg",
            "border",
            "px-4",
            "py-2",
            "text-sm",
            "focus-visible:outline-2",
            "focus-visible:outline-offset-2",
            "focus-visible:outline-accent",
            if selected {
                "border-accent bg-accent-soft text-accent font-semibold"
            } else {
                "border-line"
            },
        );
        html! {
            <button
                type="button"
                key={density.key()}
                aria-pressed={selected.to_string()}
                class={element_classes_3}
                {onclick}
            >
                {density.name()}
            </button>
        }
    });
    html! {
        <div
            class="space-y-4 rounded-2xl border border-line bg-surface p-4 shadow-sm sm:p-6"
            data-settings-appearance=""
        >
            <div>
                <h2
                    class="text-xl font-semibold"
                >
                    {"Appearance"}
                </h2>
                <p
                    class="text-sm text-muted"
                >
                    {"Color palette and light/dark display mode are independent settings."}
                </p>
            </div>
            <h3
                class="text-sm font-semibold"
            >
                {"Theme preset"}
            </h3>
            <div
                class="grid gap-2 sm:grid-cols-2 xl:grid-cols-3"
            >
                {for presets}
            </div>
            <div
                class="border-t border-line pt-4"
            >
                <h3
                    class="text-sm font-semibold"
                >
                    {"Display mode"}
                </h3>
                <p
                    class="mt-1 text-xs text-muted"
                >
                    {"Changes only when you use this control or the top-bar switch."}
                </p>
                <div
                    class="mt-3 flex gap-2"
                    role="group"
                    aria-label="Display mode"
                >
                    {for modes}
                </div>
            </div>
            <div
                class="border-t border-line pt-4"
            >
                <h3
                    class="text-sm font-semibold"
                >
                    {"Layout density"}
                </h3>
                <div
                    class="mt-3 flex flex-wrap gap-2"
                    role="group"
                    aria-label="Layout density"
                >
                    {for densities}
                </div>
            </div>
        </div>
    }
}
