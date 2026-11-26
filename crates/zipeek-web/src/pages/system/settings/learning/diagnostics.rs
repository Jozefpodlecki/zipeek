use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_browser::pedagogy_worker::inspect_worker;

#[derive(Clone, PartialEq)]
enum WorkerCheck {
    NotChecked,
    Checking,
    Available(String),
    Failed(String),
}

fn describe_error(error: JsValue) -> String {
    js_sys::Reflect::get(&error, &JsValue::from_str("message"))
        .ok()
        .and_then(|value| value.as_string())
        .or_else(|| error.as_string())
        .unwrap_or_else(|| "Worker check failed".to_owned())
}

#[function_component(LearningDiagnostics)]
pub(super) fn learning_diagnostics() -> Html {
    let status = use_state(|| WorkerCheck::NotChecked);
    let on_check = {
        let status = status.clone();
        Callback::from(move |_: MouseEvent| {
            if matches!(&*status, WorkerCheck::Checking) {
                return;
            }
            status.set(WorkerCheck::Checking);
            let status = status.clone();
            spawn_local(async move {
                match inspect_worker().await {
                    Ok(message) => status.set(WorkerCheck::Available(message)),
                    Err(error) => status.set(WorkerCheck::Failed(describe_error(error))),
                }
            });
        })
    };
    let (message, failed) = match &*status {
        WorkerCheck::NotChecked => ("Not checked".to_owned(), false),
        WorkerCheck::Checking => ("Checking…".to_owned(), false),
        WorkerCheck::Available(value) => (format!("Worker reachable: {value}"), false),
        WorkerCheck::Failed(value) => (format!("Worker unavailable: {value}"), true),
    };
    html! {
        <details data-learning-technical="" class="rounded-xl border border-line bg-surface p-4">
            <summary class="cursor-pointer font-semibold focus-visible:outline-2 focus-visible:outline-accent">{"Technical diagnostics"}</summary>
            <p class="mt-3 text-sm text-muted">{"Worker availability is diagnostic only. It does not start assessment or enable recording."}</p>
            <div class="mt-3 flex flex-wrap items-center gap-3">
                <button type="button" data-learning-worker-check="" onclick={on_check} disabled={matches!(&*status, WorkerCheck::Checking)} class="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line px-3 text-sm font-semibold hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50">
                    <Icon data={IconData::LUCIDE_ACTIVITY} width="16px" />{"Check worker"}
                </button>
                <p role={if failed { "alert" } else { "status" }} data-learning-worker-status="" class={classes!("break-words", "text-sm", if failed { "text-red-500" } else { "text-muted" })}>{message}</p>
            </div>
        </details>
    }
}
