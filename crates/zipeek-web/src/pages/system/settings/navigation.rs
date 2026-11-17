use yew::prelude::*;
use yew_icons::{Icon, IconData};

pub(super) fn is_valid_section(section: &str) -> bool {
    matches!(
        section,
        "appearance" | "characters" | "ai" | "images" | "practice" | "learning" | "speech"
    )
}

pub(super) fn view(active_tab: &str, select_tab: Callback<String>) -> Html {
    let tabs = [
        ("appearance", "Appearance", IconData::LUCIDE_SUN),
        (
            "characters",
            "Chinese Characters",
            IconData::LUCIDE_LANGUAGES,
        ),
        ("ai", "AI Translation", IconData::LUCIDE_FLAME),
        ("images", "Image Generation", IconData::LUCIDE_IMAGE),
        ("practice", "Practice", IconData::LUCIDE_VOLUME_2),
        ("speech", "Speech", IconData::LUCIDE_VOLUME_2),
        ("learning", "Learning Data", IconData::LUCIDE_BOOKMARK),
    ];
    html! {
        <nav
            aria-label="Settings sections"
            data-settings-tabs=""
            class="flex flex-wrap content-start gap-1 rounded-2xl border border-line bg-surface p-3 lg:flex-col"
        >
            {for tabs.into_iter().map(|(name, label, icon)| {
                let select_tab = select_tab.clone();
                let on_select_tab = Callback::from(move |_| select_tab.emit(name.to_owned()));
                let element_classes_1 = classes!(
                                "rounded-lg",
                                "px-3",
                                "py-3",
                                "text-left",
                                "text-sm",
                                "font-semibold",
                                "transition-colors",
                                "focus-visible:outline-2",
                                "focus-visible:outline-offset-2",
                                "focus-visible:outline-accent",
                                if active_tab == name {"bg-accent-soft text-accent ring-1 ring-accent/55"} else {"text-muted hover:bg-raised"},
                            );
                html! {
                    <button
                        key={name}
                        type="button"
                        data-settings-tab-button={name}
                        aria-current={(active_tab == name).to_string()}
                        aria-controls="settings-active-panel"
                        onclick={on_select_tab}
                        class={element_classes_1}
                    >
                        <span
                            class="inline-flex items-center gap-2"
                        >
                            <Icon
                                data={icon}
                                width="16px"
                                height="16px"
                            />{label}
                            </span>
                    </button>
                }
            })}
        </nav>
    }
}
