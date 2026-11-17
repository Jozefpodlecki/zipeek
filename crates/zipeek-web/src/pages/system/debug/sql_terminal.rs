use serde_json::{Map, Value};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::hooks::use_dictionary;

use super::sql::{MAX_ROWS, as_csv, bounded_select};
use super::sql_examples::EXAMPLES;

#[derive(Clone, PartialEq)]
enum QueryState {
    Idle,
    Running,
    Failed(String),
    Ready(Vec<Map<String, Value>>),
}

#[function_component(SqlTerminal)]
pub(super) fn sql_terminal() -> Html {
    let dictionary = use_dictionary();
    let statement = use_state(|| "SELECT id, simplified, pinyin FROM words LIMIT 10".to_owned());
    let state = use_state(|| QueryState::Idle);
    let selected_example = use_state(|| String::new());
    let on_example = {
        let statement = statement.clone();
        let state = state.clone();
        let selected_example = selected_example.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<web_sys::HtmlSelectElement>() {
                let value = input.value();
                if let Ok(index) = value.parse::<usize>() {
                    if let Some(example) = EXAMPLES.get(index) {
                        statement.set(example.sql.to_owned());
                        state.set(QueryState::Idle);
                    }
                }
                selected_example.set(value);
            }
        })
    };
    let on_input = {
        let statement = statement.clone();
        let selected_example = selected_example.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                statement.set(input.value());
                selected_example.set(String::new());
            }
        })
    };
    let on_run = {
        let dictionary = dictionary.clone();
        let statement = statement.clone();
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            let sql = match bounded_select(&statement) {
                Ok(sql) => sql,
                Err(message) => {
                    state.set(QueryState::Failed(message.into()));
                    return;
                }
            };
            state.set(QueryState::Running);
            let database = dictionary.database();
            let state = state.clone();
            spawn_local(async move {
                let result = match database {
                    Ok(db) => db.debug_select(&sql).await.map_err(|e| e.to_string()),
                    Err(error) => Err(error.to_string()),
                };
                state.set(match result {
                    Ok(rows) => QueryState::Ready(rows),
                    Err(message) => QueryState::Failed(message),
                });
            });
        })
    };
    let result = match &*state {
        QueryState::Idle => {
            html! { <p class="text-sm text-muted">{"Enter a read-only query and select Run."}</p> }
        }
        QueryState::Running => html! { <p role="status">{"Running query…"}</p> },
        QueryState::Failed(message) => html! {
            <div role="alert" data-debug-sql-error="" class="flex items-start gap-3 rounded-lg border border-red-500/40 bg-red-500/10 p-4 text-sm">
                <span aria-hidden="true" class="shrink-0 text-red-400">{"!"}</span>
                <div class="min-w-0 space-y-1">
                    <p class="font-semibold text-red-300">{"Query failed"}</p>
                    <p class="break-words text-foreground">{"Check your SQL, selected tables, and column names."}</p>
                    <pre class="max-h-40 overflow-auto whitespace-pre-wrap break-words rounded bg-surface p-2 font-mono text-xs text-muted">{message}</pre>
                    <p class="text-xs text-muted">{"Load a known-good example from the dropdown or edit the query and try again."}</p>
                </div>
            </div>
        },
        QueryState::Ready(rows) if rows.is_empty() => {
            html! { <p role="status">{"Query returned no rows."}</p> }
        }
        QueryState::Ready(rows) => {
            let columns: Vec<String> = rows[0].keys().cloned().collect();
            let csv = as_csv(rows);
            let csv_url = format!(
                "data:text/csv;charset=utf-8,{}",
                js_sys::encode_uri_component(&csv)
            );
            html! {
                <div data-debug-sql-results="" class="space-y-3">
                    <p role="status" class="text-sm">{format!("{} row(s) (max {MAX_ROWS})", rows.len())}</p>
                    <div class="max-h-72 overflow-auto rounded-lg border border-line">
                        <table class="w-full border-collapse text-left text-xs">
                            <thead><tr>{for columns.iter().map(|column| html! { <th class="border-b border-line p-2">{column}</th> })}</tr></thead>
                            <tbody>{for rows.iter().map(|row| html! {
                                <tr>{for columns.iter().map(|column| html! {
                                    <td class="max-w-56 truncate border-b border-line p-2" title={row.get(column).map(display_cell).unwrap_or_default()}>{row.get(column).map(display_cell).unwrap_or_default()}</td>
                                })}</tr>
                            })}</tbody>
                        </table>
                    </div>
                    <a data-debug-sql-export="" href={csv_url} download="zipeek-dictionary-preview.csv" class="inline-flex min-h-10 items-center rounded-lg border border-line px-4 text-sm font-semibold">{"Export displayed rows as CSV"}</a>
                </div>
            }
        }
    };
    html! {
        <section data-debug-sql-terminal="" class="space-y-3 rounded-xl border border-line bg-raised p-4">
            <h3 class="font-semibold">{"Dictionary SQL terminal"}</h3>
            <p class="text-xs text-muted">{"Debug builds only. Queries run against the packaged read-only dictionary, not personal DuckDB/OPFS data. Single SELECT, max 100 rows; exports contain only the displayed rows. Not a general-purpose SQL sandbox."}</p>
            <div class="flex flex-wrap items-end justify-between gap-3">
                <label for="debug-sql-input" class="text-sm font-semibold">{"SQL query"}</label>
                <div class="flex min-w-0 items-center gap-2">
                    <label for="debug-sql-example" class="shrink-0 text-xs text-muted">{"Examples"}</label>
                    <select id="debug-sql-example" data-debug-sql-examples="" value={(*selected_example).clone()} onchange={on_example} class="min-h-10 min-w-0 max-w-64 rounded-lg border border-line bg-surface px-3 text-sm">
                        <option value="">{"Choose one of 20 queries"}</option>
                        {for EXAMPLES.iter().enumerate().map(|(index, example)| html! {
                            <option key={index} value={index.to_string()}>{example.label}</option>
                        })}
                    </select>
                </div>
            </div>
            <textarea id="debug-sql-input" data-debug-sql-input="" value={(*statement).clone()} oninput={on_input} rows="3" class="w-full rounded-lg border border-line bg-surface p-3 font-mono text-sm" />
            <button type="button" data-debug-sql-run="" onclick={on_run} disabled={matches!(&*state, QueryState::Running)} class="min-h-10 rounded-lg bg-brand-action px-5 font-semibold text-white disabled:opacity-50">{"Run SELECT"}</button>
            {result}
        </section>
    }
}

fn display_cell(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
