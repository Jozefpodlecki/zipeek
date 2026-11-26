use serde_json::{Map, Value};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::hooks::use_dictionary;

use super::sql::{MAX_ROWS, as_csv, bounded_select};
use super::sql_examples::EXAMPLES;

const MAX_RESULT_TABS: usize = 8;

#[derive(Clone, PartialEq)]
enum QueryResult {
    Running,
    Failed(String),
    Ready(Vec<Map<String, Value>>),
}

#[derive(Clone, PartialEq)]
struct ResultTab {
    id: u64,
    name: String,
    result: QueryResult,
}

#[derive(Clone, PartialEq, Default)]
struct ResultsState {
    selected: Option<u64>,
    tabs: Vec<ResultTab>,
}

enum ResultsAction {
    Begin(u64),
    Complete(u64, Result<Vec<Map<String, Value>>, String>),
    Select(u64),
    Close(u64),
}

impl Reducible for ResultsState {
    type Action = ResultsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            ResultsAction::Begin(id) => {
                if next.tabs.len() == MAX_RESULT_TABS {
                    next.tabs.remove(0);
                }
                next.tabs.push(ResultTab {
                    id,
                    name: format!("Result {id}"),
                    result: QueryResult::Running,
                });
                next.selected = Some(id);
            }
            ResultsAction::Complete(id, outcome) => {
                if let Some(tab) = next.tabs.iter_mut().find(|tab| tab.id == id) {
                    tab.result = match outcome {
                        Ok(rows) => QueryResult::Ready(rows),
                        Err(message) => QueryResult::Failed(message),
                    };
                }
            }
            ResultsAction::Select(id) => {
                if next.tabs.iter().any(|tab| tab.id == id) {
                    next.selected = Some(id);
                }
            }
            ResultsAction::Close(id) => {
                next.tabs.retain(|tab| tab.id != id);
                if next.selected == Some(id) {
                    next.selected = next.tabs.last().map(|tab| tab.id);
                }
            }
        }
        Rc::new(next)
    }
}

#[function_component(SqlTerminal)]
pub(super) fn sql_terminal() -> Html {
    let dictionary = use_dictionary();
    let statement = use_state(|| "SELECT id, simplified, pinyin FROM words LIMIT 10".to_owned());
    let results = use_reducer(ResultsState::default);
    let query_counter = use_mut_ref(|| 0_u64);
    let on_input = {
        let statement = statement.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                statement.set(input.value());
            }
        })
    };
    let on_run = {
        let dictionary = dictionary.clone();
        let statement = statement.clone();
        let results = results.clone();
        let query_counter = query_counter.clone();
        Callback::from(move |_: MouseEvent| {
            let id = {
                let mut counter = query_counter.borrow_mut();
                *counter += 1;
                *counter
            };
            results.dispatch(ResultsAction::Begin(id));
            let validated = bounded_select(&statement).map_err(str::to_owned);
            let database = dictionary.database();
            let results = results.clone();
            spawn_local(async move {
                let outcome = match (validated, database) {
                    (Err(message), _) => Err(message),
                    (_, Err(error)) => Err(error.to_string()),
                    (Ok(sql), Ok(db)) => db.debug_select(&sql).await.map_err(|error| error.to_string()),
                };
                results.dispatch(ResultsAction::Complete(id, outcome));
            });
        })
    };
    let running = results.tabs.iter().any(|tab| matches!(&tab.result, QueryResult::Running));
    let selected = results.tabs.iter().find(|tab| Some(tab.id) == results.selected);
    html! {
        <section data-debug-sql-terminal="" class="space-y-4 rounded-xl border border-line bg-raised p-4">
            <div class="flex flex-wrap items-center justify-between gap-3">
                <details class="group relative z-10">
                    <summary data-debug-sql-examples="" class="flex min-h-10 cursor-pointer list-none items-center gap-2 rounded-lg border border-line bg-surface px-3 text-sm font-medium hover:border-accent focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_BOOK_OPEN} width="17" height="17" />
                        {"Query examples"}
                        <Icon data={IconData::LUCIDE_CHEVRON_DOWN} width="15" height="15" class="transition-transform group-open:rotate-180" />
                    </summary>
                    <div class="absolute left-0 top-full mt-2 max-h-96 w-[min(36rem,85vw)] overflow-auto rounded-xl border border-line bg-raised p-3 shadow-xl">
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
                                        let statement = statement.clone();
                                        let onclick = Callback::from(move |_: MouseEvent| statement.set(sql.clone()));
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
                <button type="button" data-debug-sql-run="" onclick={on_run} aria-label="Execute SQL query" title="Execute SQL query" class="inline-flex size-10 items-center justify-center rounded-lg bg-brand-action text-white focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_PLAY} width="19" height="19" />
                </button>
            </div>
            <textarea id="debug-sql-input" aria-label="SQL editor" data-debug-sql-input="" value={(*statement).clone()} oninput={on_input} rows="12" spellcheck="false" class="block min-h-72 w-full resize-y rounded-lg border border-line bg-surface p-4 font-mono text-sm leading-relaxed focus-visible:outline-2 focus-visible:outline-accent" />
            <section aria-label="Query results" data-debug-sql-workspace="" class="min-h-72 overflow-hidden rounded-lg border border-line bg-surface">
                <div role="tablist" aria-label="Query result tabs" class="flex min-h-11 items-center gap-1 overflow-x-auto border-b border-line px-2">
                    {if results.tabs.is_empty() { html! { <span class="px-2 text-xs text-muted">{"Results"}</span> } } else { Html::default() }}
                    {for results.tabs.iter().map(|tab| {
                        let id = tab.id;
                        let active = results.selected == Some(id);
                        let on_select = {
                            let results = results.clone();
                            Callback::from(move |_: MouseEvent| results.dispatch(ResultsAction::Select(id)))
                        };
                        let on_close = {
                            let results = results.clone();
                            Callback::from(move |_: MouseEvent| results.dispatch(ResultsAction::Close(id)))
                        };
                        html! {
                            <div key={id} class={classes!("flex", "shrink-0", "items-center", "rounded-t-md", if active { "bg-accent-soft" } else { "hover:bg-raised" })}>
                                <button type="button" role="tab" aria-selected={active.to_string()} onclick={on_select} class="flex min-h-10 items-center gap-2 px-3 text-xs font-medium focus-visible:outline-2 focus-visible:outline-accent">
                                    <Icon data={if matches!(&tab.result, QueryResult::Running) { IconData::LUCIDE_REFRESH_CW } else { IconData::LUCIDE_TABLE }} width="15" height="15" class={if matches!(&tab.result, QueryResult::Running) { "animate-spin motion-reduce:animate-none" } else { "" }} />
                                    {&tab.name}
                                </button>
                                <button type="button" aria-label={format!("Close {}", tab.name)} title="Close result" onclick={on_close} class="flex size-8 items-center justify-center rounded-md text-muted hover:text-foreground focus-visible:outline-2 focus-visible:outline-accent">
                                    <Icon data={IconData::LUCIDE_X} width="14" height="14" />
                                </button>
                            </div>
                        }
                    })}
                </div>
                <div role="tabpanel" class="min-h-64 p-3">
                    {if let Some(tab) = selected { render_result(&tab.result) } else { html! {
                        <div class="flex min-h-56 items-center justify-center gap-2 text-sm text-muted"><Icon data={IconData::LUCIDE_TABLE} width="18" height="18" />{"Run a query to view results"}</div>
                    } }}
                </div>
            </section>
            {if running { html! { <span role="status" class="sr-only">{"A query is running"}</span> } } else { Html::default() }}
        </section>
    }
}

fn render_result(result: &QueryResult) -> Html {
    match result {
        QueryResult::Running => html! { <p role="status" class="flex items-center gap-2 text-sm text-muted"><Icon data={IconData::LUCIDE_REFRESH_CW} width="16" height="16" class="animate-spin motion-reduce:animate-none" />{"Executing query"}</p> },
        QueryResult::Failed(message) => html! { <p role="alert" data-debug-sql-error="" class="flex items-start gap-2 text-sm text-red-400"><Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="17" height="17" />{message}</p> },
        QueryResult::Ready(rows) if rows.is_empty() => html! { <p role="status" class="text-sm text-muted">{"No rows returned"}</p> },
        QueryResult::Ready(rows) => {
            let columns: Vec<String> = rows[0].keys().cloned().collect();
            let csv_url = format!("data:text/csv;charset=utf-8,{}", js_sys::encode_uri_component(&as_csv(rows)));
            html! {
                <div data-debug-sql-results="" class="space-y-3">
                    <div class="flex flex-wrap items-center justify-between gap-2">
                        <span role="status" class="text-xs text-muted">{format!("{} rows (limit {MAX_ROWS})", rows.len())}</span>
                        <a data-debug-sql-export="" href={csv_url} download="zipeek-dictionary-preview.csv" title="Export displayed rows as CSV" class="inline-flex min-h-9 items-center gap-2 rounded-md border border-line px-3 text-xs font-medium hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                            <Icon data={IconData::LUCIDE_DOWNLOAD} width="15" height="15" />{"CSV"}
                        </a>
                    </div>
                    <div class="max-h-80 overflow-auto rounded-md border border-line">
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
    fn result_tabs_keep_selection_and_evict_oldest() {
        let mut state = Rc::new(ResultsState::default());
        for id in 1..=9 {
            state = state.reduce(ResultsAction::Begin(id));
        }
        assert_eq!(state.tabs.len(), MAX_RESULT_TABS);
        assert_eq!(state.tabs.first().map(|tab| tab.id), Some(2));
        assert_eq!(state.selected, Some(9));
        state = state.reduce(ResultsAction::Select(3));
        state = state.reduce(ResultsAction::Close(3));
        assert_eq!(state.selected, Some(9));
        state = state.reduce(ResultsAction::Complete(1, Ok(vec![])));
        assert_eq!(state.tabs.len(), MAX_RESULT_TABS - 1);
    }
}
