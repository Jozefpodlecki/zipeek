use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;
use zipeek_docs::{DocumentEntry, document_headings, render_document};

use super::super::hooks::{LoadState, use_document};
use super::super::model::{
    DocumentToc, category, group_entries, ordered_categories, reading_neighbours, topic,
};
use super::super::navigation::{asset_url, doc_route, scroll_to_heading};
use crate::{hooks::use_app_environment, route::Route};

#[derive(Properties, PartialEq)]
pub(in crate::pages::system::docs) struct ArticleProps {
    pub entries: Vec<DocumentEntry>,
    pub current: Option<String>,
    pub heading_target: Option<String>,
    pub on_browse: Callback<String>,
    pub on_toc: Callback<DocumentToc>,
}

#[function_component(DocsArticle)]
pub(in crate::pages::system::docs) fn docs_article(props: &ArticleProps) -> Html {
    let current = &props.current;
    let show_raw = use_state(|| false);
    let source = props.current.as_ref().map(|path| html! { <LoadedArticle key={path.clone()} path={path.clone()} heading_target={props.heading_target.clone()} show_raw={*show_raw} on_toc={props.on_toc.clone()} /> });
    let selected_entry = current
        .as_deref()
        .and_then(|path| props.entries.iter().find(|entry| entry.path == path));
    let (previous, next) = current.as_deref().map_or((None, None), |path| {
        reading_neighbours(&props.entries, path)
    });
    let heading = selected_entry.map_or("Documentation", |entry| entry.title.as_str());
    let current_category = current.as_deref().map(category);
    let current_topic = current.as_deref().map(topic);
    let groups = group_entries(&props.entries, "");
    let toggle_raw = {
        let show_raw = show_raw.clone();
        Callback::from(move |_| show_raw.set(!*show_raw))
    };
    let on_browse = props.on_browse.clone();
    html! {
                <main id="main-content" tabindex="-1" class="min-w-0 px-5 py-8 sm:px-10 lg:px-14" data-docs-article="">
                    <div class="mx-auto max-w-4xl">
                        <nav aria-label="Breadcrumb" class="mb-5 flex flex-wrap gap-2 text-xs text-muted">
                            <Link<Route> to={Route::Docs}>{"Docs"}</Link<Route>>
                            {if let Some(domain) = current_category {html! { <><span>{"/"}</span><span>{domain}</span></> }} else {Html::default()}}
                            {if let Some(subject) = current_topic {html! { <><span>{"/"}</span><span>{subject}</span></> }} else {Html::default()}}
                        </nav>
                        <h1 class="mb-4 text-3xl font-bold sm:text-4xl">{heading}</h1>
                        {if let Some(path) = &current {html! {
                            <div class="mb-6 flex flex-wrap items-center justify-between gap-3">
                                <p class="font-mono text-xs text-muted break-all">{format!("docs/{path}")}</p>
                                <div class="flex items-center gap-2">
                                    <button type="button" data-docs-source-toggle="" class="inline-flex h-10 w-10 items-center justify-center rounded-lg border border-line bg-raised text-ink hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent" title={if *show_raw {"Show rendered article"} else {"View Markdown source"}} aria-label={if *show_raw {"Show rendered article"} else {"View Markdown source"}} onclick={toggle_raw}><Icon data={IconData::LUCIDE_CODE} width="17px" height="17px" /></button>
                                    <a class="inline-flex h-10 w-10 items-center justify-center rounded-lg border border-line bg-raised text-ink hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent" title="View on GitHub" aria-label="View on GitHub (opens a new tab)" target="_blank" rel="noopener noreferrer" href={format!("https://github.com/Jozefpodlecki/zipeek/blob/master/docs/{path}")}><Icon data={IconData::LUCIDE_EXTERNAL_LINK} width="17px" height="17px" /></a>
                                </div>
                            </div>
                        }} else {Html::default()}}
                        {if let Some(content) = source { content } else {html! {
                            <div class="space-y-6"><p class="max-w-2xl text-base text-muted">{"Start with the project overview, then explore the architecture and development guides."}</p><div class="grid gap-3 sm:grid-cols-2"><Link<Route> to={doc_route("README.md")} classes="flex flex-col gap-2 rounded-xl border border-accent/50 bg-raised p-5 text-ink no-underline hover:border-accent"><span class="text-xs font-semibold uppercase tracking-wide text-accent">{"Start here"}</span><strong class="text-lg">{"Project overview"}</strong><span class="text-sm text-muted">{"What Zipeek does, where to start, and how the docs are organised."}</span></Link<Route>><Link<Route> to={doc_route("architecture/architecture.md")} classes="flex flex-col gap-2 rounded-xl border border-line bg-raised p-5 text-ink no-underline hover:border-accent"><span class="text-xs font-semibold uppercase tracking-wide text-muted">{"Next"}</span><strong class="text-lg">{"Architecture"}</strong><span class="text-sm text-muted">{"Understand the workspace, services and runtime boundaries."}</span></Link<Route>></div><h2 class="text-lg font-semibold">{"Browse by topic"}</h2>
                                <div class="grid gap-3 sm:grid-cols-2">
                                    {for ordered_categories(&groups).into_iter().map(|domain| {
                                        let domain_key = domain.clone();
                                        let on_browse = on_browse.clone();
                                        let on_open = Callback::from(move |_| on_browse.emit(domain_key.clone()));
                                        html! {
                                            <button
                                                key={domain.clone()}
                                                type="button"
                                                onclick={on_open}
                                                class="group flex min-h-20 items-center justify-between rounded-xl border border-line bg-raised p-5 text-left font-semibold transition-colors hover:border-accent hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                                            >
                                                {domain}
                                                <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="18px" height="18px" />
                                            </button>
                                        }
                                    })}
                                </div>
                            </div>
                        }}}
                        {if current.is_some() {html! {
                            <nav aria-label="Document reading order" class="mt-12 grid grid-cols-1 gap-3 border-t border-line pt-6 sm:grid-cols-2">
                                <div>
                                    {if let Some(entry) = previous {html! {
                                        <Link<Route> to={doc_route(&entry.path)} classes="group flex items-center gap-3 rounded-lg border border-line bg-surface px-4 py-3 text-sm no-underline transition-colors hover:border-accent hover:bg-raised" >
                                            <Icon data={IconData::LUCIDE_ARROW_LEFT} width="18px" height="18px" />
                                            <span class="min-w-0"><span class="block text-xs text-muted">{"Previous article"}</span>
                                            <span class="mt-1 block truncate font-semibold">{&entry.title}</span></span>
                                        </Link<Route>>
                                    }} else {Html::default()}}
                                </div>
                                <div>
                                    {if let Some(entry) = next {html! {
                                        <Link<Route> to={doc_route(&entry.path)} classes="group flex items-center gap-3 rounded-lg border border-line bg-surface px-4 py-3 text-sm no-underline transition-colors hover:border-accent hover:bg-raised" >
                                            <span class="min-w-0 flex-1 text-right"><span class="block text-xs text-muted">{"Next article"}</span>
                                            <span class="mt-1 block truncate font-semibold">{&entry.title}</span></span>
                                            <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="18px" height="18px" />
                                        </Link<Route>>
                                    }} else {Html::default()}}
                                </div>
                            </nav>
                        }} else {Html::default()}}
                    </div>
                </main>
    }
}

#[derive(Properties, PartialEq)]
struct LoadedProps {
    path: String,
    heading_target: Option<String>,
    show_raw: bool,
    on_toc: Callback<DocumentToc>,
}

#[function_component(LoadedArticle)]
fn loaded_article(props: &LoadedProps) -> Html {
    let environment = use_app_environment();
    let state = use_document(props.path.clone());
    let target = props.heading_target.clone();
    {
        let on_toc = props.on_toc.clone();
        let path = props.path.clone();
        let content = match &*state {
            LoadState::Ready(text) => Some(text.clone()),
            _ => None,
        };
        use_effect_with(content, move |content| {
            let headings = content
                .as_ref()
                .map(|text| {
                    document_headings(text)
                        .into_iter()
                        .filter(|heading| heading.level <= 3)
                        .collect()
                })
                .unwrap_or_default();
            on_toc.emit(DocumentToc {
                path: path.clone(),
                headings,
            });
            || ()
        });
    }
    let raw = props.show_raw;
    let ready = match &*state {
        LoadState::Ready(text) => Some(text.clone()),
        _ => None,
    };
    {
        let ready = ready.clone();
        let document = environment.document.clone();
        use_effect_with((target, ready, raw), move |(target, ready, raw)| {
            if !raw && ready.is_some() {
                if let Some(target) = target {
                    scroll_to_heading(&document, target);
                }
            }
            || ()
        });
    }
    let click_document = environment.document.clone();
    let on_article_click = Callback::from(move |event: MouseEvent| {
        let Some(link) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .and_then(|element| element.closest(r##"a[href^="#"]"##).ok().flatten())
        else {
            return;
        };
        let Some(fragment) = link.get_attribute("href") else {
            return;
        };
        if fragment.len() <= 1 || fragment.starts_with("#/docs/") {
            return;
        }
        if scroll_to_heading(&click_document, &fragment[1..]) {
            event.prevent_default();
        }
    });
    match &*state {
        LoadState::Loading => {
            html! { <p role="status" class="text-muted">{"Loading document..."}</p> }
        }
        LoadState::Error(error) => {
            html! { <p role="alert" class="rounded-lg border border-line p-4">{error}</p> }
        }
        LoadState::Ready(text) if props.show_raw => html! {
            <pre data-docs-source="" class="whitespace-pre-wrap break-words overflow-x-auto rounded-xl border border-line bg-raised p-5 font-mono text-sm">{text}</pre>
        },
        LoadState::Ready(text) => {
            let rendered = render_document(
                text,
                &props.path,
                &asset_url(&environment.window, "docs/").unwrap_or_default(),
            );
            html! {
                <article data-docs-markdown="" onclick={on_article_click} class="text-base leading-7 max-w-none [&_a]:text-accent [&_a]:underline [&_h1]:text-3xl [&_h2]:text-2xl [&_h3]:text-xl [&_p]:my-4 [&_ul]:list-disc [&_ol]:list-decimal [&_ul]:pl-6 [&_ol]:pl-6 [&_li]:my-1 [&_pre]:my-4 [&_pre]:overflow-x-auto [&_pre]:rounded-lg [&_pre]:bg-raised [&_pre]:p-4 [&_code]:break-words [&_pre_code]:break-normal [&_blockquote]:border-l-4 [&_blockquote]:border-line [&_blockquote]:pl-4 [&_blockquote]:italic [&_table]:my-5 [&_table]:w-full [&_th]:border [&_td]:border [&_th]:border-line [&_td]:border-line [&_th]:p-2 [&_td]:p-2 [&_th]:text-left [&_img]:max-w-full [&_img]:rounded-lg">
                    {Html::from_html_unchecked(AttrValue::from(rendered))}
                </article>
            }
        }
    }
}
