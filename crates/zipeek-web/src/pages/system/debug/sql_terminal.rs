use gloo::events::EventListener;
use serde_json::{Map, Value};
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::hooks::{use_app_environment, use_dictionary};

use super::sql::{MAX_ROWS, as_csv, bounded_select};
use super::sql_examples::EXAMPLES;

const MAX_QUERY_TABS: usize = 8;
const INITIAL_QUERY: &str = "SELECT id, simplified, pinyin FROM words LIMIT 10";

#[derive(Clone, PartialEq)]
enum QueryResult {
    Running,
    Failed(String),
    Ready(Vec<Map<String, Value>>),
}

#[derive(Clone, PartialEq)]
struct QueryTab {
    id: u64,
    statement: String,
    result: Option<QueryResult>,
}

#[derive(Clone, PartialEq)]
struct WorkspaceState {
    selected: u64,
    tabs: Vec<QueryTab>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            selected: 1,
            tabs: vec![QueryTab {
                id: 1,
                statement: INITIAL_QUERY.to_owned(),
                result: None,
            }],
        }
    }
}

enum WorkspaceAction {
    New(u64),
    Select(u64),
    Close(u64),
    Edit(u64, String),
    Begin(u64),
    Complete(u64, String, Result<Vec<Map<String, Value>>, String>),
}

impl Reducible for WorkspaceState {
    type Action = WorkspaceAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            WorkspaceAction::New(id) => {
                if next.tabs.len() == MAX_QUERY_TABS {
                    next.tabs.remove(0);
                }
                next.tabs.push(QueryTab {
                    id,
                    statement: String::new(),
                    result: None,
                });
                next.selected = id;
            }
            WorkspaceAction::Select(id) => {
                if next.tabs.iter().any(|tab| tab.id == id) {
                    next.selected = id;
                }
            }
            WorkspaceAction::Close(id) => {
                if next.tabs.len() > 1 {
                    next.tabs.retain(|tab| tab.id != id);
                    if next.selected == id {
                        next.selected = next.tabs.last().map(|tab| tab.id).unwrap_or(next.selected);
                    }
                }
            }
            WorkspaceAction::Edit(id, statement) => {
                if let Some(tab) = next.tabs.iter_mut().find(|tab| tab.id == id) {
                    tab.statement = statement;
                    tab.result = None;
                }
            }
            WorkspaceAction::Begin(id) => {
                if let Some(tab) = next.tabs.iter_mut().find(|tab| tab.id == id) {
                    tab.result = Some(QueryResult::Running);
                }
            }
            WorkspaceAction::Complete(id, submitted, outcome) => {
                if let Some(tab) = next.tabs.iter_mut().find(|tab| tab.id == id) {
                    if tab.statement == submitted
                        && matches!(&tab.result, Some(QueryResult::Running))
                    {
                        tab.result = Some(match outcome {
                            Ok(rows) => QueryResult::Ready(rows),
                            Err(message) => QueryResult::Failed(message),
                        });
                    }
                }
            }
        }
        Rc::new(next)
    }
}

#[function_component(SqlTerminal)]
pub(super) fn sql_terminal() -> Html {
    let dictionary = use_dictionary();
    let environment = use_app_environment();
    let workspace = use_reducer(WorkspaceState::default);
    let query_counter = use_mut_ref(|| 1_u64);
    let examples_ref = use_node_ref();

    {
        let examples_ref = examples_ref.clone();
        let document = environment.document.clone();
        use_effect_with((), move |_| {
            let close = {
                let examples_ref = examples_ref.clone();
                move || {
                    if let Some(element) = examples_ref.cast::<web_sys::Element>() {
                        let _ = element.remove_attribute("open");
                    }
                }
            };
            let pointer_listener = {
                let close = close.clone();
                EventListener::new(&document, "pointerdown", move |event| {
                    let within_menu = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .and_then(|element| {
                            element
                                .closest("[data-debug-sql-examples-menu]")
                                .ok()
                                .flatten()
                        })
                        .is_some();
                    if !within_menu {
                        close();
                    }
                })
            };
            let key_listener = {
                EventListener::new(&document, "keydown", move |event| {
                    if event
                        .dyn_ref::<web_sys::KeyboardEvent>()
                        .is_some_and(|key| key.key() == "Escape")
                    {
                        close();
                    }
                })
            };
            move || {
                drop(pointer_listener);
                drop(key_listener);
            }
        });
    }

    let selected = workspace
        .tabs
        .iter()
        .find(|tab| tab.id == workspace.selected);
    let on_new = {
        let workspace = workspace.clone();
        let counter = query_counter.clone();
        Callback::from(move |_: MouseEvent| {
            let id = {
                let mut value = counter.borrow_mut();
                *value += 1;
                *value
            };
            workspace.dispatch(WorkspaceAction::New(id));
        })
    };
    let on_run = {
        let dictionary = dictionary.clone();
        let workspace = workspace.clone();
        let selected = selected.map(|tab| (tab.id, tab.statement.clone()));
        Callback::from(move |_: MouseEvent| {
            let Some((id, statement)) = selected.clone() else {
                return;
            };
            workspace.dispatch(WorkspaceAction::Begin(id));
            let validated = bounded_select(&statement).map_err(str::to_owned);
            let database = dictionary.database();
            let workspace = workspace.clone();
            spawn_local(async move {
                let outcome = match (validated, database) {
                    (Err(message), _) => Err(message),
                    (_, Err(error)) => Err(error.to_string()),
                    (Ok(sql), Ok(db)) => db
                        .debug_select(&sql)
                        .await
                        .map_err(|error| error.to_string()),
                };
                workspace.dispatch(WorkspaceAction::Complete(id, statement, outcome));
            });
        })
    };

    html! {
        <section data-debug-sql-terminal="" class="flex min-h-0 flex-col gap-3 rounded-xl border border-line bg-raised p-3 md:h-[calc(100dvh-19rem)] md:min-h-[26rem] md:overflow-hidden">
            <div role="tablist" aria-label="SQL query tabs" class="flex min-h-10 shrink-0 items-center gap-1 overflow-x-auto border-b border-line pb-2">
                {for workspace.tabs.iter().map(|tab| {
                    let id = tab.id;
                    let active = workspace.selected == id;
                    let on_select = {
                        let workspace = workspace.clone();
                        Callback::from(move |_: MouseEvent| workspace.dispatch(WorkspaceAction::Select(id)))
                    };
                    let on_close = {
                        let workspace = workspace.clone();
                        Callback::from(move |_: MouseEvent| workspace.dispatch(WorkspaceAction::Close(id)))
                    };
                    html! {
                        <div key={id} class={classes!("flex", "shrink-0", "items-center", "rounded-md", if active { "bg-accent-soft" } else { "hover:bg-surface" })}>
                            <button type="button" role="tab" aria-selected={active.to_string()} onclick={on_select} class="flex min-h-9 items-center gap-2 px-3 text-sm font-medium focus-visible:outline-2 focus-visible:outline-accent">
                                <Icon data={IconData::LUCIDE_FILE_CODE} width="16" height="16" />
                                {format!("Query {id}")}
                                {if matches!(&tab.result, Some(QueryResult::Running)) { html! { <Icon data={IconData::LUCIDE_REFRESH_CW} width="13" height="13" class="animate-spin motion-reduce:animate-none" /> } } else { Html::default() }}
                            </button>
                            {if workspace.tabs.len() > 1 { html! {
                                <button type="button" aria-label={format!("Close Query {id}")} onclick={on_close} class="flex size-8 items-center justify-center rounded-md text-muted hover:text-foreground focus-visible:outline-2 focus-visible:outline-accent">
                                    <Icon data={IconData::LUCIDE_X} width="14" height="14" />
                                </button>
                            } } else { Html::default() }}
                        </div>
                    }
                })}
                <button type="button" onclick={on_new} aria-label="New SQL query tab" title="New query" class="flex size-9 shrink-0 items-center justify-center rounded-md text-muted hover:bg-surface hover:text-foreground focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_PLUS} width="17" height="17" />
                </button>
            </div>
            <div class="flex shrink-0 flex-wrap items-center justify-between gap-2">
                <details ref={examples_ref.clone()} data-debug-sql-examples-menu="" class="group relative z-10">
                    <summary data-debug-sql-examples="" class="flex min-h-9 cursor-pointer list-none items-center gap-2 rounded-lg border border-line bg-surface px-3 text-sm font-medium hover:border-accent focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_BOOK_OPEN} width="16" height="16" />
                        {"Examples"}
                        <Icon data={IconData::LUCIDE_CHEVRON_DOWN} width="14" height="14" class="transition-transform group-open:rotate-180" />
                    </summary>
                    <div class="absolute left-0 top-full mt-2 max-h-80 w-[min(36rem,82vw)] overflow-auto rounded-xl border border-line bg-raised p-3 shadow-xl">
                        {for [
                            ("Explore", 0_usize, 5_usize),
                            ("HSK and vocabulary", 5, 15),
                            ("Dictionary diagnostics", 15, EXAMPLES.len()),
                        ].into_iter().map(|(group, start, end)| html! {
                            <div class="mb-3 last:mb-0" key={group}>
                                <p class="mb-2 px-2 text-xs font-semibold uppercase tracking-wide text-muted">{group}</p>
                                <div class="grid grid-cols-1 gap-1 sm:grid-cols-2">
                                    {for EXAMPLES[start..end].iter().map(|example| {
                                        let sql = example.sql.to_owned();
                                        let workspace = workspace.clone();
                                        let examples_ref = examples_ref.clone();
                                        let id = workspace.selected;
                                        let onclick = Callback::from(move |_: MouseEvent| {
                                            workspace.dispatch(WorkspaceAction::Edit(id, sql.clone()));
                                            if let Some(element) = examples_ref.cast::<web_sys::Element>() {
                                                let _ = element.remove_attribute("open");
                                            }
                                        });
                                        html! {
                                            <button type="button" key={example.label} title={example.sql} {onclick}
                                                class="min-h-9 rounded-md px-2 py-2 text-left text-sm text-foreground hover:bg-accent-soft focus-visible:outline-2 focus-visible:outline-accent">
                                                {example.label}
                                            </button>
                                        }
                                    })}
                                </div>
                            </div>
                        })}
                    </div>
                </details>
                <button type="button" data-debug-sql-run="" onclick={on_run} aria-label="Execute SQL query" title="Execute SQL query" class="inline-flex size-9 items-center justify-center rounded-lg bg-brand-action text-white focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_PLAY} width="17" height="17" />
                </button>
            </div>
            {if let Some(tab) = selected {
                let id = tab.id;
                let on_input = {
                    let workspace = workspace.clone();
                    Callback::from(move |event: InputEvent| {
                        if let Some(input) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                            workspace.dispatch(WorkspaceAction::Edit(id, input.value()));
                        }
                    })
                };
                html! {
                    <div key={id} class="flex min-h-0 flex-1 flex-col gap-3">
                        <textarea id="debug-sql-input" aria-label="SQL editor" data-debug-sql-input="" value={tab.statement.clone()} oninput={on_input} rows="6" spellcheck="false" class="block h-36 min-h-28 w-full shrink-0 resize-y rounded-lg border border-line bg-surface p-3 font-mono text-sm leading-relaxed focus-visible:outline-2 focus-visible:outline-accent" />
                        <section aria-label="Query result" data-debug-sql-workspace="" class="flex min-h-40 flex-1 flex-col overflow-hidden rounded-lg border border-line bg-surface">
                            <div class="min-h-0 flex-1 overflow-auto p-3" role="tabpanel">
                                {if let Some(result) = &tab.result { render_result(result) } else { html! {
                                    <div class="flex items-center gap-2 text-sm text-muted"><Icon data={IconData::LUCIDE_TABLE} width="17" height="17" />{"Run the query to view results"}</div>
                                }}}
                            </div>
                        </section>
                    </div>
                }
            } else { Html::default() }}
        </section>
    }
}

fn render_result(result: &QueryResult) -> Html {
    match result {
        QueryResult::Running => {
            html! { <p role="status" class="flex items-center gap-2 text-sm text-muted"><Icon data={IconData::LUCIDE_REFRESH_CW} width="16" height="16" class="animate-spin motion-reduce:animate-none" />{"Executing query"}</p> }
        }
        QueryResult::Failed(message) => {
            html! { <p role="alert" data-debug-sql-error="" class="flex items-start gap-2 text-sm text-red-400"><Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="17" height="17" />{message}</p> }
        }
        QueryResult::Ready(rows) if rows.is_empty() => {
            html! { <p role="status" class="text-sm text-muted">{"No rows returned"}</p> }
        }
        QueryResult::Ready(rows) => {
            let columns: Vec<String> = rows[0].keys().cloned().collect();
            let csv_url = format!(
                "data:text/csv;charset=utf-8,{}",
                js_sys::encode_uri_component(&as_csv(rows))
            );
            html! {
                <div data-debug-sql-results="" class="space-y-3">
                    <div class="flex flex-wrap items-center justify-between gap-2">
                        <span role="status" class="text-xs text-muted">{format!("{} rows (limit {MAX_ROWS})", rows.len())}</span>
                        <a data-debug-sql-export="" href={csv_url} download="zipeek-dictionary-preview.csv" title="Export displayed rows as CSV" class="inline-flex min-h-9 items-center gap-2 rounded-md border border-line px-3 text-xs font-medium hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                            <Icon data={IconData::LUCIDE_DOWNLOAD} width="15" height="15" />{"CSV"}
                        </a>
                    </div>
                    <div class="overflow-x-auto rounded-md border border-line">
                        <table class="w-full border-collapse text-left text-xs">
                            <thead class="sticky top-0 bg-raised"><tr>{for columns.iter().map(|column| html! { <th class="whitespace-nowrap border-b border-line p-2">{column}</th> })}</tr></thead>
                            <tbody>{for rows.iter().map(|row| html! {
                                <tr class="hover:bg-raised">{for columns.iter().map(|column| html! {
                                    <td class="max-w-64 truncate border-b border-line p-2" title={row.get(column).map(display_cell).unwrap_or_default()}>{row.get(column).map(display_cell).unwrap_or_default()}</td>
                                })}</tr>
                            })}</tbody>
                        </table>
                    </div>
                </div>
            }
        }
    }
}

fn display_cell(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_tabs_keep_editors_and_results_isolated() {
        let mut state = Rc::new(WorkspaceState::default());
        state = state.reduce(WorkspaceAction::New(2));
        state = state.reduce(WorkspaceAction::Edit(2, "SELECT 1".into()));
        state = state.reduce(WorkspaceAction::Begin(2));
        state = state.reduce(WorkspaceAction::Complete(2, "SELECT 1".into(), Ok(vec![])));
        assert!(matches!(state.tabs[1].result, Some(QueryResult::Ready(_))));
        assert!(state.tabs[0].result.is_none());
        state = state.reduce(WorkspaceAction::Edit(2, "SELECT 2".into()));
        assert!(state.tabs[1].result.is_none());
        state = state.reduce(WorkspaceAction::Complete(2, "SELECT 1".into(), Ok(vec![])));
        assert!(state.tabs[1].result.is_none());
    }

    #[test]
    fn query_tabs_bound_count_and_preserve_one_editor() {
        let mut state = Rc::new(WorkspaceState::default());
        for id in 2..=10 {
            state = state.reduce(WorkspaceAction::New(id));
        }
        assert_eq!(state.tabs.len(), MAX_QUERY_TABS);
        assert_eq!(state.tabs.first().map(|tab| tab.id), Some(3));
        state = state.reduce(WorkspaceAction::Select(4));
        state = state.reduce(WorkspaceAction::Close(4));
        assert_eq!(state.selected, 10);
        for id in 3..=9 {
            state = state.reduce(WorkspaceAction::Close(id));
        }
        assert_eq!(state.tabs.len(), 1);
        state = state.reduce(WorkspaceAction::Close(10));
        assert_eq!(state.tabs.len(), 1);
    }
}
