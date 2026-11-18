use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
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

#[function_component(LearningDataSettings)]
pub(super) fn learning_data_settings() -> Html {
    let check = use_state(|| WorkerCheck::NotChecked);
    let on_check = {
        let check = check.clone();
        Callback::from(move |_: MouseEvent| {
            if matches!(&*check, WorkerCheck::Checking) {
                return;
            }
            check.set(WorkerCheck::Checking);
            let check = check.clone();
            spawn_local(async move {
                match inspect_worker().await {
                    Ok(message) => check.set(WorkerCheck::Available(message)),
                    Err(error) => check.set(WorkerCheck::Failed(describe_error(error))),
                }
            });
        })
    };
    let (status, failed) = match &*check {
        WorkerCheck::NotChecked => ("Not checked".to_owned(), false),
        WorkerCheck::Checking => ("Checking worker…".to_owned(), false),
        WorkerCheck::Available(text) => (text.clone(), false),
        WorkerCheck::Failed(text) => (text.clone(), true),
    };
    let worker_message = if failed {
        html! { <p role="alert" data-learning-worker-status="" class="mt-3 break-words text-sm text-red-600">{status}</p> }
    } else {
        html! { <p role="status" data-learning-worker-status="" class="mt-3 break-words text-sm text-muted">{status}</p> }
    };
    html! {
        <div data-settings-learning="" class="flex flex-col gap-4">
            <super::super::personal_storage::PersonalStoragePanel />
            <details data-learning-data-table="" class="rounded-xl border border-line bg-surface p-4">
                <summary class="cursor-pointer font-semibold">{"Data categories"}</summary>
                <div class="mt-3 overflow-x-auto">
                    <table class="w-full min-w-[480px] text-left text-sm">
                        <caption class="sr-only">{"Learning data capabilities and availability"}</caption>
                        <thead><tr class="border-b border-line text-muted"><th scope="col" class="p-2">{"Category"}</th><th scope="col" class="p-2">{"Current handling"}</th></tr></thead>
                        <tbody>
                            <tr class="border-b border-line"><th scope="row" class="p-2 font-medium">{"Practice completions"}</th><td class="p-2">{"Optional browser-local history"}</td></tr>
                            <tr class="border-b border-line"><th scope="row" class="p-2 font-medium">{"Word Sets and settings"}</th><td class="p-2">{"Browser-local; portable backup options below"}</td></tr>
                            <tr><th scope="row" class="p-2 font-medium">{"Proficiency evidence"}</th><td class="p-2">{"Not connected to Statistics"}</td></tr>
                        </tbody>
                    </table>
                </div>
            </details>
            <details data-learning-technical="" class="rounded-xl border border-line bg-surface p-4">
                <summary class="cursor-pointer text-sm font-semibold">{"Technical details"}</summary>
                <p class="mt-3 text-sm text-muted">{"The analysis worker is not connected to recorded learning evidence. Checking it does not start tracking or award mastery."}</p>
                {worker_message}
                <button type="button" data-learning-worker-check="" onclick={on_check}
                    disabled={matches!(&*check, WorkerCheck::Checking)}
                    class="mt-3 min-h-11 rounded-lg border border-line px-4 font-semibold hover:bg-raised disabled:opacity-50">
                    {"Check analysis worker"}
                </button>
            </details>
        </div>
    }
}
