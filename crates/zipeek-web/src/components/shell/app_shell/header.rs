mod dragon;
use crate::{components::ThemeToggle, route::Route};
use dragon::HeaderDragon;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

#[derive(Properties, PartialEq)]
pub(super) struct ShellHeaderProps {
    pub expanded: bool,
    pub recording_enabled: bool,
    pub game_workspace: bool,
    pub write_hanzi_workspace: bool,
    pub on_drawer_toggle: Callback<MouseEvent>,
}

#[function_component(ShellHeader)]
pub(super) fn shell_header(props: &ShellHeaderProps) -> Html {
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
                    class="flex shrink-0 items-center gap-3"
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
                <div class="min-w-0 flex-1 px-1 sm:px-3 lg:px-7"><HeaderDragon recording_enabled={props.recording_enabled} /></div>
                <div
                    class="flex shrink-0 items-center gap-3"
                >
                    <ThemeToggle />
                </div>
            </header>
    }
}
