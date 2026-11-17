use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::hooks::{use_theme, use_theme_commands};

#[function_component(ThemeToggle)]
pub fn theme_toggle() -> Html {
    let theme = use_theme();
    let commands = use_theme_commands();
    let dark = theme.is_dark();
    let onclick = Callback::from(move |_| commands.toggle());
    let icon = if dark {
        IconData::LUCIDE_MOON
    } else {
        IconData::LUCIDE_SUN
    };
    let label = if dark { "Dark" } else { "Light" };

    let element_classes_1 = classes!(
        "ml-auto",
        "hidden",
        "h-[1.05rem]",
        "w-[1.85rem]",
        "rounded-2xl",
        "p-[.15rem]",
        "md:block",
        if dark { "bg-brand-action" } else { "bg-line" },
    );
    let element_classes_2 = classes!(
        "block",
        "size-3",
        "rounded-full",
        "bg-white",
        "shadow-[0_1px_2px_#0002]",
        "transition-transform",
        "duration-150",
        "motion-reduce:transition-none",
        dark.then_some("translate-x-[.8rem]"),
    );
    html! {
        <button
            type="button"
            class="flex min-h-10 w-10 items-center justify-center gap-[.7rem] rounded-[.65rem] border-0 bg-transparent p-0 text-[.8rem] text-muted transition-colors duration-150 hover:bg-raised hover:text-ink focus-visible:outline-offset-1 motion-reduce:transition-none md:w-auto md:px-3 md:py-2"
            data-component="theme-toggle"
            data-theme-toggle=""
            role="switch"
            aria-label="Dark mode"
            aria-checked={dark.to_string()}
            {onclick}
        >
            <span
                aria-hidden="true"
            >
                <Icon
                    data={icon}
                    width="18px"
                    height="18px"
                />
            </span>
            <span
                class="hidden md:inline"
            >
                {label}
            </span>
            <span
                class={element_classes_1}
                aria-hidden="true"
            >
                <span
                    class={element_classes_2}
                />
            </span>
        </button>
    }
}
