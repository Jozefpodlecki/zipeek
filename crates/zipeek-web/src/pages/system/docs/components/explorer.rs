use std::collections::BTreeSet;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;
use zipeek_docs::DocumentEntry;

use super::super::model::{category, group_entries, matches_search, ordered_categories, topic, visible};
use super::super::navigation::{asset_url, doc_route};
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub(in crate::pages::system::docs) struct ExplorerProps {
    pub entries: Vec<DocumentEntry>,
    pub current: Option<String>,
    pub manifest_error: Option<String>,
    pub mobile_open: bool,
    pub opened_category: Option<String>,
}

#[function_component(DocsExplorer)]
pub(in crate::pages::system::docs) fn docs_explorer(props: &ExplorerProps) -> Html {
    let query = use_state(String::new);
    let expanded = use_state(|| None::<String>);
    let expanded_topics = use_state(BTreeSet::<String>::new);
    let search = query.trim().to_lowercase();
    let filtered = props
        .entries
        .iter()
.filter(|entry| visible(entry) && matches_search(entry, &search))
        .count();
    let groups = group_entries(&props.entries, &search);
    let current = &props.current;
    let expanded_category = (*expanded).as_ref().or(props.opened_category.as_ref());
    let current_category = current.as_deref().map(category);
    let current_topic = current.as_deref().map(topic);
    let manifest_error = &props.manifest_error;
    let mobile_open = props.mobile_open;
    let onchange = {
        let query = query.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            query.set(input.value());
        })
    };
    html! {
                <aside id="docs-explorer" aria-label="Documentation explorer" class={classes!("border-r", "border-line", "bg-surface", "md:sticky", "md:top-[4.5rem]", "md:h-[calc(100dvh-4.5rem)]", "md:overflow-y-auto", if mobile_open {"block"}else{"hidden"}, if mobile_open {""}else{"md:block"})}>
                    <div class="space-y-3 p-3 sm:p-4">
                        <div class="relative">
                            <span class="pointer-events-none absolute inset-y-0 left-3 flex items-center text-muted" aria-hidden="true">
                                <Icon data={IconData::LUCIDE_SEARCH} width="16px" height="16px" />
                            </span>
                            <input
                                data-docs-search=""
                                type="search"
                                aria-label="Search documentation"
                                placeholder="Search documentation"
                                value={(*query).clone()}
                                oninput={onchange}
                                class="w-full min-h-11 appearance-none rounded-md border border-transparent bg-raised py-2 pl-10 pr-3 text-sm text-ink outline-none transition-colors placeholder:text-muted hover:bg-surface focus-visible:bg-surface focus-visible:ring-0 focus-visible:shadow-sm"
                            />
                        </div>
                        {if !search.is_empty() {html! {
                            <p class="px-2 text-xs text-muted" role="status">{format!("{} matching documents", filtered)}</p>
                        }} else {Html::default()}}
                        { for ordered_categories(&groups).into_iter().map(|domain| {
                            let topics = &groups[domain];
                            let is_open = !search.is_empty() || expanded_category.map_or(current_category.as_ref() == Some(domain), |value| value == domain);
                            let domain_key = domain.clone();
                            let expanded = expanded.clone();
                            let on_domain_toggle = Callback::from(move |_| {
                                expanded.set(if is_open { Some(String::new()) } else { Some(domain_key.clone()) });
                            });
                            html! {
                                <section class="space-y-1" key={domain.clone()}>
                                    <button type="button" aria-expanded={is_open.to_string()} class="flex w-full items-center justify-between rounded-lg px-2 py-2 text-left text-sm font-semibold hover:bg-raised" onclick={on_domain_toggle}>
                                        <span>{domain}</span><Icon data={if is_open {IconData::LUCIDE_CHEVRON_DOWN} else {IconData::LUCIDE_CHEVRON_RIGHT}} width="16px" height="16px" />
                                    </button>
                                    { if is_open { html! { <div class="ml-3 border-l border-line pl-3 space-y-2">
                                        {for topics.iter().map(|(subject, docs)| {
                                            let topic_key = format!("{domain}/{subject}");
                                            let topic_open = subject.is_empty() || !search.is_empty() || expanded_topics.contains(&topic_key) ||
                                                (current_category.as_ref() == Some(domain) && current_topic.as_ref() == Some(subject) && !expanded_topics.contains(&format!("!{topic_key}")));
                                            let expanded_topics = expanded_topics.clone();
                                            let toggle_key = topic_key.clone();
                                            let on_topic_toggle = Callback::from(move |_| {
                                                let mut state = (*expanded_topics).clone();
                                                if topic_open { state.remove(&toggle_key); state.insert(format!("!{toggle_key}")); }
                                                else { state.remove(&format!("!{toggle_key}")); state.insert(toggle_key.clone()); }
                                                expanded_topics.set(state);
                                            });
                                            html! {<div class="space-y-1" key={subject.clone()}>
                                                {if subject.is_empty() {Html::default()} else {html! {
                                                <button type="button" aria-expanded={topic_open.to_string()} onclick={on_topic_toggle} class="flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-xs font-semibold text-muted hover:bg-raised hover:text-ink">
                                                    <span>{subject}</span><Icon data={if topic_open {IconData::LUCIDE_CHEVRON_DOWN} else {IconData::LUCIDE_CHEVRON_RIGHT}} width="14px" height="14px" />
                                                </button>
                                                }}}
                                                {if topic_open {html! { <div class="space-y-1">
                                                {for docs.iter().map(|entry| {
                                                    let selected = current.as_ref() == Some(&entry.path);
                                                    html! { <div key={entry.path.clone()}>
                                                        {if entry.markdown {html! {
                                                            <Link<Route> to={doc_route(&entry.path)} classes={classes!("block", "rounded-md", "px-2", "py-1.5", "text-sm", "break-words", "hover:bg-raised", if selected { "bg-raised" } else { "text-ink" }, if selected { "font-semibold" } else { "" }, if selected { "text-accent" } else { "" })}>{&entry.title}</Link<Route>>
                                                        }} else {html! {
                                                            <a href={asset_url(&format!("docs/{}", entry.path)).unwrap_or_default()} class="block rounded-md px-2 py-1.5 text-sm text-muted hover:bg-raised">{format!("{} ↗",entry.title)}</a>
                                                        }}}
                                                    </div> }
                                                })}
                                                </div> }} else {Html::default()}}
                                            </div>}
                                        })}
                                    </div> }} else {Html::default()} }
                                </section>
                            }
                        })}
                        {if let Some(error) = manifest_error.as_ref() { html! { <p role="alert" class="text-sm text-red-600">{error}</p> } } else {Html::default()}}
                    </div>
                </aside>
    }
}
