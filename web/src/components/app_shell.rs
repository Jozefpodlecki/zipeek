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
        <div class={classes!("app-shell", (!*expanded).then_some("is-collapsed"))}>
            <a class="skip-link" href="#main-content" onclick={onskip}>{"Skip to content"}</a>
            <header class="app-topbar">
                <div class="topbar-navigation">
                    <button type="button" class="drawer-toggle" data-drawer-toggle=""
                        aria-label={if *expanded {"Collapse navigation"} else {"Expand navigation"}}
                        aria-controls="app-drawer" aria-expanded={expanded.to_string()} onclick={on_drawer_toggle}>
                        <span aria-hidden="true"><Icon data={IconData::LUCIDE_MENU} width="20px" height="20px" /></span>
                    </button>
                    <Link<Route> to={Route::Home} classes="brand-link">
                        // <span class="brand-mark" aria-hidden="true">{"字"}</span>
                        <img src="public/favicon-32x32.png" tag="brand"/>
                        <strong>{"Zipeek"}</strong>
                    </Link<Route>>
                </div>
                <ThemeToggle />
            </header>
            <aside id="app-drawer" class="app-sidebar" aria-label="Zipeek" hidden={!*expanded}>
                <Navigation />
                <div class="sidebar-tools">
                    <a class="repository-link" href="https://github.com/Jozefpodlecki/zipeek"
                        aria-label="Zipeek repository on GitHub" target="_blank" rel="noopener noreferrer">
                        <span aria-hidden="true"><Icon data={IconData::SIMPLE_ICONS_GITHUB} width="18px" height="18px" /></span>
                        <span>{"GitHub"}</span>
                    </a>
                </div>
            </aside>
            <div class="app-main">
                <main id="main-content" class="app-content" tabindex="-1" ref={content}>
                    {props.children.clone()}
                </main>
                <AppFooter />
            </div>
        </div>
    }
}
