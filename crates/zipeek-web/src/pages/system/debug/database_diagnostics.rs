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

#[derive(Clone, Debug, PartialEq)]
enum SimulationPhase {
    Idle,
    Running,
    Stopped,
    Completed,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
struct SimulationState {
    phase: SimulationPhase,
    run_id: String,
    completed: u32,
    logs: Vec<String>,
    verification: Option<Result<String, String>>,
    busy: bool,
    verifying: bool,
    started_at: f64,
    verification_id: u64,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            phase: SimulationPhase::Idle,
            run_id: String::new(),
            completed: 0,
            logs: Vec::new(),
            verification: None,
            busy: false,
            verifying: false,
            started_at: 0.0,
            verification_id: 0,
        }
    }
}

enum SimulationAction {
    Start {
        id: String,
        at: f64,
    },
    Stop,
    CycleStarted {
        id: String,
    },
    CycleFinished {
        id: String,
        cycle: u32,
        result: Result<u32, String>,
        elapsed: f64,
        total_elapsed: f64,
    },
    VerifyStarted,
    VerifyFinished {
        verification_id: u64,
        result: Result<String, String>,
    },
}

impl Reducible for SimulationState {
    type Action = SimulationAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            SimulationAction::Start { id, at } => {
                if next.busy || next.verifying || next.phase == SimulationPhase::Running {
                    return self;
                }
                next = Self {
                    phase: SimulationPhase::Running,
                    run_id: id,
                    started_at: at,
                    verification_id: next.verification_id.wrapping_add(1),
                    ..Self::default()
                };
            }
            SimulationAction::Stop => {
                if next.phase == SimulationPhase::Running {
                    next.phase = SimulationPhase::Stopped;
                }
            }
            SimulationAction::CycleStarted { id } => {
                if next.phase == SimulationPhase::Running && next.run_id == id && !next.busy {
                    next.busy = true;
                }
            }
            SimulationAction::CycleFinished {
                id,
                cycle,
                result,
                elapsed,
                total_elapsed,
            } => {
                if next.run_id != id || !next.busy || cycle != next.completed + 1 {
                    return self;
                }
                next.busy = false;
                match result {
                    Ok(revision) => {
                        next.logs.push(format!(
                            "Cycle {cycle}/10: revision {revision} saved, reopened and verified (operation {elapsed:.0} ms; total {total_elapsed:.0} ms)"
                        ));
                        next.completed = cycle;
                        if cycle == 10 && next.phase == SimulationPhase::Running {
                            next.phase = SimulationPhase::Completed;
                        }
                    }
                    Err(error) => {
                        next.logs.push(format!("Cycle {cycle}/10 FAILED: {error}"));
                        next.phase = SimulationPhase::Failed;
                    }
                }
            }
            SimulationAction::VerifyStarted => {
                if next.phase != SimulationPhase::Running && !next.busy && !next.verifying {
                    next.verification_id = next.verification_id.wrapping_add(1);
                    next.verification = None;
                    next.verifying = true;
                }
            }
            SimulationAction::VerifyFinished {
                verification_id,
                result,
            } => {
                if next.verifying && next.verification_id == verification_id {
                    next.verification = Some(result);
                    next.verifying = false;
                }
            }
        }
        Rc::new(next)
    }
}

#[function_component(PersistenceSimulator)]
fn persistence_simulator(props: &PersistenceSimulatorProps) -> Html {
    let state = use_reducer_eq(SimulationState::default);
    let in_flight = use_mut_ref(|| false);
    {
        let state = state.clone();
        let in_flight = in_flight.clone();
        let document = props.document.clone();
        let run_id = state.run_id.clone();
        let completed = state.completed;
        let started_at = state.started_at;
        let running = state.phase == SimulationPhase::Running;
        use_effect_with((running, completed, run_id.clone()), move |_| {
            let alive = Rc::new(Cell::new(true));
            let interval = if running && completed < 10 && !run_id.is_empty() {
                let state = state.clone();
                let in_flight = in_flight.clone();
                let document = document.clone();
                let alive = alive.clone();
                Some(Interval::new(5_000, move || {
                    if !alive.get() || *in_flight.borrow() {
                        return;
                    }
                    *in_flight.borrow_mut() = true;
                    state.dispatch(SimulationAction::CycleStarted { id: run_id.clone() });
                    let started = js_sys::Date::now();
                    let state = state.clone();
                    let in_flight = in_flight.clone();
                    let document = document.clone();
                    let run_id = run_id.clone();
                    let cycle = completed + 1;
                    wasm_bindgen_futures::spawn_local(async move {
                        let result =
                            zipeek_db::wasm::PersonalPersistenceSimulator::save_and_verify(
                                &document, &run_id, cycle,
                            )
                            .await
                            .map(|checkpoint| checkpoint.revision)
                            .map_err(|error| error.to_string());
                        *in_flight.borrow_mut() = false;
                        state.dispatch(SimulationAction::CycleFinished {
                            id: run_id,
                            cycle,
                            result,
                            elapsed: js_sys::Date::now() - started,
                            total_elapsed: js_sys::Date::now() - started_at,
                        });
                    });
                }))
            } else {
                None
            };
            move || {
                alive.set(false);
                drop(interval);
            }
        });
    }
    let on_start = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(SimulationAction::Start {
                id: uuid::Uuid::now_v7().to_string(),
                at: js_sys::Date::now(),
            });
        })
    };
    let on_stop = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(SimulationAction::Stop))
    };
    let on_verify = {
        let document = props.document.clone();
        let state = state.clone();
        let verification_id = state.verification_id.wrapping_add(1);
        Callback::from(move |_: MouseEvent| {
            state.dispatch(SimulationAction::VerifyStarted);
            let document = document.clone();
            let state = state.clone();
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
                state.dispatch(SimulationAction::VerifyFinished {
                    verification_id,
                    result,
                });
            });
        })
    };
    let running = state.phase == SimulationPhase::Running;
    html! {
        <div class="mt-5 border-t border-line pt-4" data-persistence-simulator="">
            <h4 class="font-semibold">{"Personal DuckDB / OPFS persistence simulator"}</h4>
            <p class="mt-1 text-xs text-muted">{"Development-only. Writes an isolated synthetic session every 5 seconds, up to 10 cycles, using the existing personal database and exclusive Web Lock. Each cycle closes and reopens the database to verify its revision. No learner scores are changed."}</p>
            <div class="mt-3 flex flex-wrap gap-2">
                <button type="button" class="rounded-lg bg-accent px-3 py-2 text-sm text-white disabled:opacity-50" onclick={on_start} disabled={props.disabled || running || state.busy || state.verifying}>{"Start simulation"}</button>
                <button type="button" class="rounded-lg border border-line px-3 py-2 text-sm disabled:opacity-50" onclick={on_stop} disabled={!running}>{"Stop"}</button>
                <button type="button" class="rounded-lg border border-line px-3 py-2 text-sm disabled:opacity-50" disabled={props.disabled || running || state.busy || state.verifying} onclick={on_verify}>{"Verify saved state after reload"}</button>
            </div>
            <p class="mt-2 text-sm" role="status">{format!("{} | {} of 10 completed", match &state.phase {
                SimulationPhase::Running => "Running",
                SimulationPhase::Idle => "Idle",
                SimulationPhase::Stopped => "Stopped",
                SimulationPhase::Completed => "Completed",
                SimulationPhase::Failed => "Failed",
            }, state.completed)}</p>
            <ul class="mt-2 space-y-1 text-xs" aria-label="Persistence simulation results">
                {for state.logs.iter().map(|entry| html! { <li class="break-words">{entry}</li> })}
            </ul>
            {match &state.verification {
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
    use super::{SimulationAction, SimulationPhase, SimulationState, extract_storage_version};
    use std::rc::Rc;
    use yew::Reducible;

    #[test]
    fn simulation_rejects_outdated_cycle_results() {
        let state = Rc::new(SimulationState::default()).reduce(SimulationAction::Start {
            id: "first".to_owned(),
            at: 10.0,
        });
        let state = state.reduce(SimulationAction::CycleStarted {
            id: "first".to_owned(),
        });
        let state = state.reduce(SimulationAction::CycleFinished {
            id: "old".to_owned(),
            cycle: 1,
            result: Ok(1),
            elapsed: 2.0,
            total_elapsed: 2.0,
        });
        assert_eq!(state.completed, 0);
        assert!(state.busy);
        let state = state.reduce(SimulationAction::CycleFinished {
            id: "first".to_owned(),
            cycle: 1,
            result: Ok(1),
            elapsed: 2.0,
            total_elapsed: 2.0,
        });
        assert_eq!(state.completed, 1);
        assert!(!state.busy);
        assert_eq!(state.logs.len(), 1);
    }

    #[test]
    fn stopped_simulation_keeps_in_flight_result_without_resuming() {
        let state = Rc::new(SimulationState::default()).reduce(SimulationAction::Start {
            id: "test".to_owned(),
            at: 1.0,
        });
        let state = state.reduce(SimulationAction::CycleStarted {
            id: "test".to_owned(),
        });
        let state = state.reduce(SimulationAction::Stop);
        let state = state.reduce(SimulationAction::CycleFinished {
            id: "test".to_owned(),
            cycle: 1,
            result: Ok(1),
            elapsed: 2.0,
            total_elapsed: 2.0,
        });
        assert_eq!(state.phase, SimulationPhase::Stopped);
        assert_eq!(state.completed, 1);
    }

    #[test]
    fn old_verification_cannot_overwrite_newer_result() {
        let state = Rc::new(SimulationState::default()).reduce(SimulationAction::VerifyStarted);
        let state = state.reduce(SimulationAction::VerifyFinished {
            verification_id: 0,
            result: Ok("stale".to_owned()),
        });
        assert!(state.verification.is_none());
        let state = state.reduce(SimulationAction::VerifyFinished {
            verification_id: 1,
            result: Ok("fresh".to_owned()),
        });
        assert_eq!(state.verification, Some(Ok("fresh".to_owned())));
    }

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
