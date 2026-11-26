use yew::prelude::*;
use yew_icons::{Icon, IconData};

pub(super) fn is_valid_section(section: &str, is_tauri: bool) -> bool {
    if section == "services" {
        return is_tauri;
    }
    matches!(
        section,
        "appearance"
            | "characters"
            | "ai"
            | "images"
            | "practice"
            | "learning"
            | "speech"
            | "reset"
    )
}

pub(super) fn view(active_tab: &str, select_tab: Callback<String>, is_tauri: bool) -> Html {
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
        ("learning", "Learning", IconData::LUCIDE_BOOKMARK),
        ("reset", "Reset app", IconData::LUCIDE_TRASH_2),
        ("services", "Services", IconData::LUCIDE_SERVER),
    ];
    html! {
        <nav
            aria-label="Settings sections"
            data-settings-tabs=""
            class="flex flex-wrap content-start gap-1 rounded-2xl border border-line bg-surface p-3 lg:flex-col"
        >
            {for tabs.into_iter().filter(|(name, _, _)| is_tauri || *name != "services").map(|(name, label, icon)| {
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

#[cfg(test)]
mod tests {
    use super::is_valid_section;

    #[test]
    fn services_section_is_desktop_only() {
        assert!(is_valid_section("services", true));
        assert!(!is_valid_section("services", false));
        assert!(is_valid_section("speech", false));
        assert!(!is_valid_section("unknown", true));
    }
}
