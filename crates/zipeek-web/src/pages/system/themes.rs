use crate::appearance::{Appearance, Density, Preset};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(ThemesPage)]
pub fn themes_page() -> Html {
    let Some(appearance) = use_context::<Appearance>() else {
        return html! { <p
            role="alert"
        >
            {"Theme preferences are unavailable."}
        </p> };
    };
    html! {
            <section
                data-component="themes-page"
                class="mx-auto flex w-full max-w-7xl flex-col gap-7 py-4"
            >
                <header
                    class="flex flex-wrap items-end justify-between gap-5"
                >
                    <div
                        class="space-y-2"
                    >
                        <h1
                            class="text-3xl font-bold tracking-tight"
                        >
                            {"Themes"}
                        </h1>
                        <p
                            class="text-muted"
                        >
                            {"Choose a color palette and the spacing that feels right for you."}
                        </p>
                    </div>
                    <div
                        role="group"
                        aria-label="Page layout density"
                        class="flex rounded-xl border border-line bg-surface p-1"
                    >
                        {for Density::ALL.into_iter().map(|density| {
                            let current = appearance.density == density;
                            let selected_density = appearance.select_density.clone();
                            let on_click = Callback::from(move |_: MouseEvent| selected_density.emit(density));
                            let element_classes_1 = classes!(
                                        "rounded-lg",
                                        "px-4",
                                        "py-2",
                                        "text-sm",
                                        "font-medium",
                                        "transition-colors",
                                        "focus-visible:outline-2",
                                        "focus-visible:outline-offset-2",
                                        "focus-visible:outline-accent",
                                        if current {"bg-brand-action text-white"} else {"text-muted hover:bg-raised"},
                                    );
                            html! { <button
                                type="button"
                                data-density={density.key()}
                                aria-pressed={current.to_string()}
                                onclick={on_click}
                                class={element_classes_1}
                            >
                                {density.name()}
                            </button> }
                        })}
                    </div>
                </header>
                <div
                    class="grid gap-5 md:grid-cols-2 xl:grid-cols-3"
                >
                    {for Preset::ALL.into_iter().map(|preset| {
                        let chosen = appearance.preset == preset;
                        let colors = preset.colors();
                        let selected_preset = appearance.select_preset.clone();
                        let on_click = Callback::from(move |_: MouseEvent| selected_preset.emit(preset));
                                            let chosen_theme_marker = {if chosen {html! {
                                        <span
                                            class="inline-flex items-center gap-1 rounded-full bg-brand-action px-2 py-1 text-xs font-bold text-white"
                                        >
                                            <Icon
                                                data={IconData::LUCIDE_CHECK}
                                                width="12px"
                                                height="12px"
                                            />
                                            {"Selected"}
                                        </span>}} else {Html::default()}};
    let element_classes_2 = classes!(
                                        "group",
                                        "rounded-2xl",
                                        "border-2",
                                        "p-3",
                                        "text-left",
                                        "transition-all",
                                        "hover:-translate-y-0.5",
                                        "hover:shadow-xl",
                                        "focus-visible:outline-2",
                                        "focus-visible:outline-offset-2",
                                        "focus-visible:outline-accent",
                                        if chosen {"border-accent bg-accent-soft"} else {"border-line bg-surface"},
                                    );
    html! {
                            <button
                                type="button"
                                data-theme-option={preset.key()}
                                aria-pressed={chosen.to_string()}
                                onclick={on_click}
                                class={element_classes_2}
                            >
                                <div
                                    class="overflow-hidden rounded-xl border border-line"
                                    style={format!("background:{}; color:{};", colors[2], colors[0])}
                                >
                                    <div
                                        class="flex h-6 items-center gap-1 border-b px-3"
                                        style={format!("border-color:{}40", colors[0])}
                                    >
                                        <span
                                            class="size-2 rounded-full"
                                            style={format!("background:{}", colors[0])}
                                        />
                                        <span
                                            class="size-2 rounded-full"
                                            style={format!("background:{}", colors[1])}
                                        />
                                        <span
                                            class="ml-2 text-[10px] font-bold"
                                        >
                                            {"Zipeek"}
                                        </span>
                                    </div>
                                    <div
                                        class="flex h-32 gap-2 p-3"
                                    >
                                        <div
                                            class="flex w-1/4 flex-col gap-2 rounded-md p-2"
                                            style={format!("background:{}35", colors[1])}
                                        >
                                            <div
                                                class="h-2 w-full rounded-full"
                                                style={format!("background:{}", colors[0])}
                                            />
                                            <div
                                                class="h-2 w-3/4 rounded-full"
                                                style={format!("background:{}90", colors[1])}
                                            />
                                            <div
                                                class="h-2 w-full rounded-full"
                                                style={format!("background:{}90", colors[1])}
                                            />
                                        </div>
                                        <div
                                            class="flex flex-1 flex-col gap-2"
                                        >
                                            <div
                                                class="flex h-12 flex-col gap-2 rounded-lg p-3"
                                                style={format!("background:{}85", colors[1])}
                                            >
                                                <div
                                                    class="h-2 w-2/3 rounded-full"
                                                    style={format!("background:{}", colors[0])}
                                                />
                                                <div
                                                    class="h-1.5 w-1/2 rounded-full"
                                                    style={format!("background:{}", colors[2])}
                                                />
                                            </div>
                                            <div
                                                class="grid flex-1 grid-cols-3 gap-2"
                                            >
                                                {for (0..3).map(|_| html! {
                                                <div
                                                    class="rounded-md p-2"
                                                    style={format!("background:{}70", colors[1])}
                                                >
                                                    <div
                                                        class="h-2 rounded-full"
                                                        style={format!("background:{}90", colors[0])}
                                                    />
                                                    </div>})}
                                                    </div>
                                        </div>
                                    </div>
                                </div>
                                <div
                                    class="flex items-center justify-between gap-3 px-1 pt-3"
                                >
                                    <div>
                                        <strong
                                            class="block text-base"
                                        >
                                        {preset.name()}
                                        </strong><span
                                            class="text-sm text-muted"
                                        >
                                            {preset.description()}
                                        </span>
                                        </div>
                                    {chosen_theme_marker}
                                </div>
                                <div
                                    class="flex gap-2 px-1 pt-3"
                                    aria-hidden="true"
                                >
                                    {for colors.into_iter().map(|color| html! {
                                    <span
                                        class="size-5 rounded-full border border-line"
                                        style={format!("background:{color}")}
                                    />})}
                                    </div>
                            </button>
                        }
                    })}
                </div>
                <p
                    class="text-sm text-muted"
                >
                    {"Changes apply immediately and are stored on this device. The top-bar light/dark toggle remains available; Chinese character fonts are managed separately under Settings."}
                </p>
            </section>
        }
}
