mod components;
mod hooks;
mod model;
mod navigation;

use yew::prelude::*;
use yew_router::prelude::*;
use zipeek_docs::{DocumentEntry, DocumentHeading};

use crate::route::Route;
use components::article::DocsArticle;
use components::explorer::DocsExplorer;
use components::table_of_contents::DocsTableOfContents;
use hooks::{LoadState, use_index};
use model::DocumentToc;
use navigation::{HeadingQuery, heading_target};

#[function_component(DocsPage)]
pub fn docs_page() -> Html {
    let route = use_route::<Route>();
    let target = use_location()
        .and_then(|location| location.query::<HeadingQuery>().ok())
        .and_then(heading_target);
    let current = match route {
        Some(Route::DocsArticle { file }) => Some(file.replace('~', "/")),
        _ => None,
    };
    let index = use_index();
    let mobile_open = use_state(|| false);
    let opened_category = use_state(|| None::<String>);
    let toc = use_state(|| None::<DocumentToc>);
    let entries = match &*index {
        LoadState::Ready(items) => items.clone(),
        _ => Vec::<DocumentEntry>::new(),
    };
    let manifest_error = match &*index {
        LoadState::Error(error) => Some(error.clone()),
        _ => None,
    };
    let on_browse = {
        let mobile_open = mobile_open.clone();
        let opened_category = opened_category.clone();
        Callback::from(move |category: String| {
            opened_category.set(Some(category));
            mobile_open.set(true);
        })
    };
    let open_mobile = {
        let mobile_open = mobile_open.clone();
        Callback::from(move |_| mobile_open.set(!*mobile_open))
    };
    let on_toc = {
        let toc = toc.clone();
        Callback::from(move |value| toc.set(Some(value)))
    };
    let headings: Vec<DocumentHeading> = toc
        .as_ref()
        .as_ref()
        .filter(|toc| Some(&toc.path) == current.as_ref())
        .map(|toc| toc.headings.clone())
        .unwrap_or_default();
    html! {
        <div data-component="docs-page" class="min-h-dvh bg-surface text-ink">
            <header class="sticky top-0 z-30 flex items-center justify-between gap-4 border-b border-line bg-surface px-4 py-3 shadow-sm md:px-7">
                <div class="flex items-center gap-4">
                    <Link<Route> to={Route::Home} classes="inline-flex items-center gap-2 rounded-lg px-2 py-2 text-sm font-semibold text-ink no-underline hover:bg-raised">
                    <img src="public/favicon-32x32.png" alt="" class="h-7 w-7 rounded-md" />{"Zipeek"}
                    </Link<Route>>
                    <span aria-hidden="true" class="text-muted">{"/"}</span>
                    <Link<Route> to={Route::Docs} classes="text-sm font-semibold text-accent no-underline hover:underline">{"Documentation"}</Link<Route>>
                </div>
                <button type="button" class="rounded-lg border border-line px-3 py-2 text-sm md:hidden" onclick={open_mobile} aria-expanded={(*mobile_open).to_string()} aria-controls="docs-explorer">{"Browse documents"}</button>
            </header>
            <div class={classes!("md:grid", "md:grid-cols-[minmax(14rem,17rem)_minmax(0,1fr)]", if headings.is_empty() {"xl:grid-cols-[minmax(14rem,17rem)_minmax(0,1fr)]"} else {"xl:grid-cols-[minmax(14rem,17rem)_minmax(0,1fr)_minmax(11rem,14rem)]"})}>
                <DocsExplorer {entries} current={current.clone()} {manifest_error} mobile_open={*mobile_open} opened_category={(*opened_category).clone()} />
                <DocsArticle entries={match &*index {LoadState::Ready(items) => items.clone(), _ => Vec::new()}} {current} heading_target={target} {on_browse} {on_toc} />
                <DocsTableOfContents toc={headings} />
            </div>
        </div>
    }
}
