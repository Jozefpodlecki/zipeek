use gloo::timers::callback::Interval;
use std::{cell::Cell, rc::Rc};
use yew::prelude::*;

use crate::{
    contexts::database::DatabaseStatus,
    hooks::{use_app_environment, use_database_status, use_dictionary},
    services::database::StorageStatistics,
};

#[function_component(DatabaseDiagnostics)]
pub(super) fn database_diagnostics() -> Html {
    let dictionary = use_dictionary();
    let environment = use_app_environment();
    let probe = use_state(|| None::<Result<zipeek_db::wasm::ProbeReport, String>>);
    let actual_tags = use_state(|| None::<Result<String, String>>);
    {
        let probe = probe.clone();
        let document = environment.document.clone();
        let is_tauri = environment.is_tauri;
        use_effect_with((), move |_| {
            let active = Rc::new(Cell::new(true));
            if !is_tauri {
                let pending = active.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = zipeek_db::wasm::PersonalDbProbe::run(&document)
                        .await
                        .map_err(|error| error.to_string());
                    if pending.get() {
                        probe.set(Some(result));
                    }
                });
            }
            move || active.set(false)
        });
    }
    let status = use_database_status();
    let statistics = use_state(|| None::<Result<StorageStatistics, String>>);

    {
        let statistics = statistics.clone();
        let actual_tags = actual_tags.clone();
        use_effect_with(status.clone(), move |status| {
            let active = Rc::new(Cell::new(true));
            let pending = active.clone();
            match status {
                DatabaseStatus::Loading => {
                    statistics.set(None);
                    actual_tags.set(None);
                }
                DatabaseStatus::Failed(message) => {
                    statistics.set(Some(Err(format!("Dictionary unavailable: {message}"))));
                    actual_tags.set(None);
                }
                DatabaseStatus::Ready(_) => {
                    statistics.set(None);
                    actual_tags.set(None);
                    wasm_bindgen_futures::spawn_local(async move {
                        let (result, tags) = match dictionary.database() {
                            Ok(database) => (
                                database
                                    .storage_statistics()
                                    .await
                                    .map_err(|error| error.to_string()),
                                database
                                    .actual_storage_tags()
                                    .await
                                    .map_err(|error| error.to_string()),
                            ),
                            Err(error) => (Err(error.to_string()), Err(error.to_string())),
                        };
                        if pending.get() {
                            statistics.set(Some(result));
                            actual_tags.set(Some(tags));
                        }
                    });
                }
            }
            move || active.set(false)
        });
    }

    let manifest_details = match &status {
        DatabaseStatus::Ready(info) => html! {
            <dl class="mt-3 grid grid-cols-2 gap-2 text-sm">
                <dt>{if environment.is_tauri { "Native DuckDB engine" } else { "Browser DuckDB engine" }}</dt>
                <dd>{&info.engine}</dd>
                <dt>{"Generator DuckDB engine"}</dt>
                <dd>{&info.native_engine}</dd>
                <dt>{"Dictionary storage version"}</dt>
                <dd>{match &*actual_tags {
                    Some(Ok(tags)) => extract_storage_version(tags).unwrap_or("Unknown"),
                    Some(Err(_)) => "Unavailable",
                    None => "Loading...",
                }}</dd>
                <dt>{if environment.is_tauri { "Native storage version" } else { "WASM supported storage version" }}</dt>
                <dd role="status">{if environment.is_tauri {
                    info.storage_compatibility.as_str()
                } else {
                    match &*probe {
                        None => "Checking...",
                        Some(Ok(report)) => extract_storage_version(&report.storage_tags).unwrap_or("Unknown"),
                        Some(Err(_)) => "Check failed",
                    }
                }}</dd>
                <dt>{"Dictionary words"}</dt>
                <dd>{info.row_count}</dd>
            </dl>
        },
        _ => Html::default(),
    };

    let storage_details = match &*statistics {
        None => html! { <p role="status">{"Reading database metadata..."}</p> },
        Some(Err(error)) => html! {
            <p role="status" class="break-all">
                {format!("DuckDB storage metadata unavailable: {error}")}
            </p>
        },
        Some(Ok(info)) => html! {
            <dl data-duckdb-statistics="" class="mt-3 grid grid-cols-2 gap-2 text-sm">
                <dt>{"Database size"}</dt><dd>{&info.database_size}</dd>
                <dt>{"Block size (bytes)"}</dt><dd>{&info.block_size}</dd>
                <dt>{"Total blocks"}</dt><dd>{&info.total_blocks}</dd>
                <dt>{"Used blocks"}</dt><dd>{&info.used_blocks}</dd>
                <dt>{"Free blocks"}</dt><dd>{&info.free_blocks}</dd>
                <dt>{"WAL size"}</dt><dd>{&info.wal_size}</dd>
            </dl>
        },
    };

    html! {
        <section class="rounded-xl border border-line bg-raised p-4" data-duckdb-diagnostics="">
            <h3 class="font-semibold">{"Loaded dictionary storage"}</h3>
            <p class="mt-1 text-xs text-muted">
                {if environment.is_tauri { "Live native DuckDB metadata from the desktop dictionary." } else { "Live storage metadata from the loaded dictionary and the browser DuckDB runtime." }}
            </p>
            {manifest_details}
            {storage_details}
            {if let Some(Err(error)) = &*actual_tags { html! {
                <p role="alert" class="mt-2 break-words text-sm text-red-300">{format!("Unable to query actual dictionary storage version: {error}")}</p>
            }} else { Html::default() }}
            {if let Some(Err(error)) = &*probe { html! {
                <p role="alert" class="mt-2 break-words text-sm text-red-300">{format!("WASM storage check failed: {error}")}</p>
            }} else { Html::default() }}
            <PersistenceSimulator document={environment.document.clone()} disabled={environment.is_tauri} />
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct PersistenceSimulatorProps {
    document: web_sys::Document,
    disabled: bool,
}

#[function_component(PersistenceSimulator)]
fn persistence_simulator(props: &PersistenceSimulatorProps) -> Html {
    let running = use_state(|| false);
    let run_id = use_state(String::new);
    let completed = use_state(|| 0_u32);
    let logs = use_state(Vec::<String>::new);
    let latest = use_state(|| None::<Result<String, String>>);
    let busy = use_state(|| false);
    let started_at = use_state(js_sys::Date::now);
    {
        let running = running.clone();
        let completed = completed.clone();
        let logs = logs.clone();
        let busy = busy.clone();
        let document = props.document.clone();
        let started_at = *started_at;
        let run_id = (*run_id).clone();
        use_effect_with(
            (*running, *completed, run_id.clone()),
            move |(active, count, _)| {
                let alive = Rc::new(Cell::new(true));
                let in_flight = Rc::new(Cell::new(false));
                let interval = if *active && *count < 10 && !run_id.is_empty() {
                    let alive_for_tick = alive.clone();
                    let in_flight_for_tick = in_flight.clone();
                    let running = running.clone();
                    let completed = completed.clone();
                    let logs = logs.clone();
                    let busy = busy.clone();
                    let document = document.clone();
                    let cycle = *count + 1;
                    Some(Interval::new(5_000, move || {
                        if !alive_for_tick.get() || in_flight_for_tick.get() {
                            return;
                        }
                        in_flight_for_tick.set(true);
                        busy.set(true);
                        let started = js_sys::Date::now();
                        let running = running.clone();
                        let completed = completed.clone();
                        let logs = logs.clone();
                        let busy = busy.clone();
                        let document = document.clone();
                        let run_id = run_id.clone();
                        let alive = alive_for_tick.clone();
                        let in_flight = in_flight_for_tick.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let result =
                                zipeek_db::wasm::PersonalPersistenceSimulator::save_and_verify(
                                    &document, &run_id, cycle,
                                )
                                .await;
                            in_flight.set(false);
                            if !alive.get() {
                                busy.set(false);
                                return;
                            }
                            let elapsed = js_sys::Date::now() - started;
                            let total_elapsed = js_sys::Date::now() - started_at;
                            let entry = match result {
                                Ok(checkpoint) => format!(
                                    "Cycle {cycle}/10: revision {} saved, reopened and verified (operation {elapsed:.0} ms; total {total_elapsed:.0} ms)",
                                    checkpoint.revision
                                ),
                                Err(error) => {
                                    running.set(false);
                                    format!("Cycle {cycle}/10 FAILED: {error}")
                                }
                            };
                            let successful = entry.contains("saved, reopened and verified");
                            let mut updated = (*logs).clone();
                            updated.push(entry);
                            logs.set(updated);
                            busy.set(false);
                            if successful {
                                completed.set(cycle);
                                if cycle == 10 {
                                    running.set(false);
                                }
                            }
                        });
                    }))
                } else {
                    None
                };
                move || {
                    alive.set(false);
                    drop(interval);
                }
            },
        );
    }
    let on_start = {
        let running = running.clone();
        let run_id = run_id.clone();
        let completed = completed.clone();
        let logs = logs.clone();
        let latest = latest.clone();
        let started_at = started_at.clone();
        Callback::from(move |_: MouseEvent| {
            started_at.set(js_sys::Date::now());
            run_id.set(uuid::Uuid::now_v7().to_string());
            completed.set(0);
            logs.set(Vec::new());
            latest.set(None);
            running.set(true);
        })
    };
    let on_stop = {
        let running = running.clone();
        Callback::from(move |_: MouseEvent| running.set(false))
    };
    let on_verify = {
        let document = props.document.clone();
        let latest = latest.clone();
        Callback::from(move |_: MouseEvent| {
            let document = document.clone();
            let latest = latest.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result =
                    zipeek_db::wasm::PersonalPersistenceSimulator::verify_latest(&document)
                        .await
                        .map(|record| match record {
                            Some(value) => format!(
                                "Persisted run {} restored at revision {}/10",
                                value.run_id, value.revision
                            ),
                            None => "No simulator checkpoint has been saved".to_owned(),
                        })
                        .map_err(|error| error.to_string());
                latest.set(Some(result));
            });
        })
    };
    html! {
        <div class="mt-5 border-t border-line pt-4" data-persistence-simulator="">
            <h4 class="font-semibold">{"Personal DuckDB / OPFS persistence simulator"}</h4>
            <p class="mt-1 text-xs text-muted">{"Development-only. Writes an isolated synthetic session every 5 seconds, up to 10 cycles, using the existing personal database and exclusive Web Lock. Each cycle closes and reopens the database to verify its revision. No learner scores are changed."}</p>
            <div class="mt-3 flex flex-wrap gap-2">
                <button type="button" class="rounded-lg bg-accent px-3 py-2 text-sm text-white disabled:opacity-50" onclick={on_start} disabled={props.disabled || *running || *busy}>{"Start simulation"}</button>
                <button type="button" class="rounded-lg border border-line px-3 py-2 text-sm disabled:opacity-50" onclick={on_stop} disabled={!*running}>{"Stop"}</button>
                <button type="button" class="rounded-lg border border-line px-3 py-2 text-sm disabled:opacity-50" disabled={props.disabled || *running || *busy} onclick={on_verify}>{"Verify saved state after reload"}</button>
            </div>
            <p class="mt-2 text-sm" role="status">{format!("{} | {} of 10 completed", if *running { "Running" } else { "Stopped" }, *completed)}</p>
            <ul class="mt-2 space-y-1 text-xs" aria-label="Persistence simulation results">
                {for (*logs).iter().map(|entry| html! { <li class="break-words">{entry}</li> })}
            </ul>
            {match &*latest {
                Some(Ok(message)) => html! { <p class="mt-2 text-sm" role="status">{message}</p> },
                Some(Err(message)) => html! { <p class="mt-2 text-sm text-red-300" role="alert">{format!("Persistence verification failed: {message}")}</p> },
                None => Html::default(),
            }}
        </div>
    }
}

fn extract_storage_version(tags: &str) -> Option<&str> {
    let index = tags.find("storage_version")?;
    let tail = &tags[index + "storage_version".len()..];
    let start = tail.find('v')?;
    let version = &tail[start..];
    let len = version
        .bytes()
        .take_while(|byte| byte.is_ascii_digit() || *byte == b'v' || *byte == b'.' || *byte == b'+')
        .count();
    (len > 1).then_some(&version[..len])
}

#[cfg(test)]
mod tests {
    use super::extract_storage_version;

    #[test]
    fn duckdb_storage_tags_display_only_the_version() {
        assert_eq!(
            extract_storage_version("{storage_version=v2.0.0+}"),
            Some("v2.0.0+")
        );
        assert_eq!(
            extract_storage_version("{'storage_version': 'v1.4.0+'}"),
            Some("v1.4.0+")
        );
        assert_eq!(extract_storage_version("{}"), None);
    }
}
