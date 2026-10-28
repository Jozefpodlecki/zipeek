use web_sys::HtmlElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;

use crate::{
    components::{AppFooter, Navigation, ThemeToggle},
    route::Route,
};

#[derive(Properties, PartialEq)]
pub struct AppShellProps {
    pub children: Children,
}

#[function_component(AppShell)]
pub fn app_shell(props: &AppShellProps) -> Html {
    let content = use_node_ref();
    let route = use_route::<Route>();
    {
        let content = content.clone();
        use_effect_with(route, move |route| {
            if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                document.set_title(&format!(
                    "{} · Zipeek",
                    route.as_ref().map_or("Page not found", Route::title)
                ));
            }
            if let Some(element) = content.cast::<HtmlElement>()
                && let Err(error) = element.focus()
            {
                log::warn!("Could not focus route content: {error:?}");
            }
        });
    }
    let expanded = use_state(|| true);
    let on_drawer_toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_| expanded.set(!*expanded))
    };
    let onskip = {
        let content = content.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            if let Some(element) = content.cast::<HtmlElement>()
                && let Err(error) = element.focus()
            {
                log::warn!("Could not focus main content: {error:?}");
            }
        })
    };
    html! {
        <div class={classes!("flex", "min-h-dvh", "flex-col", "items-stretch", "md:grid", "md:grid-rows-[var(--topbar-height)_1fr]", "md:items-start", if *expanded { "md:grid-cols-[15rem_minmax(0,1fr)]" } else { "md:grid-cols-1" })}>
            <a data-skip-link="" class="fixed top-4 left-4 z-[100] -translate-y-[200%] rounded-lg border border-accent bg-surface px-4 py-[.8rem] text-accent transition-transform duration-150 focus:translate-y-0 motion-reduce:transition-none" href="#main-content" onclick={onskip}>{"Skip to content"}</a>
            <header data-topbar="" class="sticky top-0 z-20 col-span-full flex h-(--topbar-height) w-full items-center justify-between gap-4 border-b border-line bg-surface px-3 backdrop-blur-[12px] md:px-5">
                <div class="flex items-center gap-3">
                    <button type="button" class="grid size-10 place-items-center rounded-lg bg-transparent text-muted transition-colors duration-150 hover:bg-raised hover:text-ink focus-visible:outline-offset-1 motion-reduce:transition-none" data-drawer-toggle=""
                        aria-label={if *expanded {"Collapse navigation"} else {"Expand navigation"}}
                        aria-controls="app-drawer" aria-expanded={expanded.to_string()} onclick={on_drawer_toggle}>
                        <span aria-hidden="true"><Icon data={IconData::LUCIDE_MENU} width="20px" height="20px" /></span>
                    </button>
                    <Link<Route> to={Route::Home} classes="flex items-center gap-[.8rem] text-ink no-underline md:px-2">
                        <img src="public/favicon-32x32.png" alt="" class="size-8 shrink-0" />
                        <strong>{"Zipeek"}</strong>
                    </Link<Route>>
                </div>
                <ThemeToggle />
            </header>
            <aside id="app-drawer" class="flex w-full flex-col gap-2 border-b border-line bg-surface px-5 py-3 backdrop-blur-[12px] md:sticky md:top-(--topbar-height) md:row-start-2 md:h-[calc(100dvh-var(--topbar-height))] md:gap-8 md:border-r md:border-b-0 md:p-5" aria-label="Zipeek" hidden={!*expanded}>
                <Navigation />
            </aside>
            <div class={classes!("flex", "min-w-0", "flex-1", "flex-col", "md:row-start-2", "md:min-h-[calc(100dvh-var(--topbar-height))]", if *expanded { "md:col-start-2" } else { "md:col-start-1" })}>
                <main id="main-content" class="w-full flex-1 px-5 py-6 md:px-[clamp(1.5rem,4vw,4.5rem)]" tabindex="-1" ref={content}>
                    {props.children.clone()}
                </main>
                <AppFooter />
            </div>
        </div>
    }
}
