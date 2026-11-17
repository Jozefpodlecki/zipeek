use crate::{components::ThemeToggle, route::Route};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

#[derive(Properties, PartialEq)]
pub(super) struct ShellHeaderProps {
    pub centered_title: Option<String>,
    pub expanded: bool,
    pub recording_enabled: bool,
    pub game_workspace: bool,
    pub write_hanzi_workspace: bool,
    pub on_drawer_toggle: Callback<MouseEvent>,
}

#[function_component(ShellHeader)]
pub(super) fn shell_header(props: &ShellHeaderProps) -> Html {
    let centered_header_title = {
        if let Some(title) = props.centered_title.as_ref() {
            html! {
                <div
                    data-centered-route-title=""
                    class="pointer-events-none absolute left-1/2 flex max-w-[42vw] -translate-x-1/2 items-center gap-2 text-center"
                >
                    <h1
                        class="truncate text-sm font-semibold sm:text-base"
                    >
                        {title.clone()}
                    </h1>

                </div>
            }
        } else {
            Html::default()
        }
    };
    let dragon = if props.recording_enabled {
        html! {
            <span
                data-learning-dragon=""
                role="img"
                aria-label="Learning activity recording enabled; proficiency is not yet assessed"
                title="Learning activity recording is on. Proficiency is not yet assessed."
                class="inline-flex size-9 items-center justify-center rounded-lg border border-line bg-raised text-accent motion-safe:animate-[zipeek-dragon-enter_260ms_ease-out_both] motion-reduce:animate-none"
            >
                <svg aria-hidden="true" viewBox="0 0 32 32" class="size-7" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M9 11Q4 15 9 20Q14 25 21 19Q26 15 22 11Q19 8 16 12Q13 16 17 18" />
                    <path d="M21 12Q26 8 28 12L25 16M23 10L24 5M26 10L29 7M12 12L9 7M10 19L6 23M16 21L14 26M21 18L25 22" />
                    <circle cx="25" cy="13" r="1" fill="currentColor" stroke="none" />
                </svg>
            </span>
        }
    } else {
        Html::default()
    };
    let element_classes_1 = classes!(
        "grid",
        "size-10",
        "place-items-center",
        "rounded-lg",
        "bg-transparent",
        "text-muted",
        "transition-colors",
        "duration-150",
        "hover:bg-raised",
        "hover:text-ink",
        "focus-visible:outline-offset-1",
        "motion-reduce:transition-none",
        if props.game_workspace { "hidden" } else { "" },
    );
    html! {
            <header
                data-component="app-header"
                data-topbar=""
                class="sticky top-0 z-20 col-span-full flex h-(--topbar-height) w-full items-center justify-between gap-4 border-b border-line bg-surface px-3 backdrop-blur-[12px] md:px-5"
            >
                <div
                    class="flex items-center gap-3"
                >
                    <button
                        type="button"
                        class={element_classes_1}
                        data-drawer-toggle=""
                        aria-label={if props.expanded {"Show navigation icons only"} else {"Expand navigation"}}
                        aria-controls="app-drawer"
                        aria-expanded={props.expanded.to_string()}
                        onclick={props.on_drawer_toggle.clone()}
                    >
                        <span
                            aria-hidden="true"
                        >
                            <Icon
                                data={IconData::LUCIDE_MENU}
                                width="20px"
                                height="20px"
                            />
                            </span>
                    </button>
                    <span
                        data-brand-link=""
                        class="inline-flex"
                    >
                        <Link<Route> to={Route::Home} classes="flex items-center gap-[.8rem] text-ink no-underline md:px-2">
                            <img
                                src="public/favicon-32x32.png"
                                alt=""
                                class="size-8 shrink-0"
                            />
                            <strong>
                                {"Zipeek"}
                            </strong>
                        </Link<Route>>
                    </span>
                </div>
                {centered_header_title}
                <div
                    class="flex min-w-0 items-center gap-3"
                >
                    {dragon}
                    <ThemeToggle />
                </div>
            </header>
    }
}
