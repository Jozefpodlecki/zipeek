use crate::hooks::use_app_environment;
use crate::services::practice_history::{
    RecordingPreference, recording_preference, set_recording_preference,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[function_component(PersonalLearningHistoryPanel)]
pub(super) fn personal_learning_history_panel() -> Html {
    let environment = use_app_environment();
    let history_enabled = use_state(|| recording_preference(&environment.local_storage));
    let on_history_toggle = {
        let history_enabled = history_enabled.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |event: Event| {
            let enabled = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .checked();
            match set_recording_preference(&storage, enabled) {
                Ok(()) => {
                    history_enabled.set(Ok(if enabled {
                        RecordingPreference::Enabled
                    } else {
                        RecordingPreference::Disabled
                    }));
                }
                Err(error) => history_enabled.set(Err(error)),
            }
        })
    };
    let history_delete_confirmation = use_state(String::new);
    let history_delete_running = use_state(|| false);
    let history_feedback = use_state(String::new);
    let history_count = use_state(|| None::<Result<u64, String>>);
    let history_count_running = use_state(|| false);
    let history_summary =
        use_state(|| None::<Result<zipeek_db::wasm::PracticeOutcomeSummary, String>>);
    let history_summary_running = use_state(|| false);
    let refresh_history_summary = {
        let document = environment.document.clone();
        let summary = history_summary.clone();
        let running = history_summary_running.clone();
        Callback::from(move |_: MouseEvent| {
            if *running {
                return;
            }
            running.set(true);
            let document = document.clone();
            let summary = summary.clone();
            let running = running.clone();
            spawn_local(async move {
                let result = crate::services::learning_history::summary(&document)
                    .await
                    .map_err(|error| error.to_string());
                summary.set(Some(result));
                running.set(false);
            });
        })
    };

    let refresh_history_count = {
        let document = environment.document.clone();
        let count = history_count.clone();
        let running = history_count_running.clone();
        Callback::from(move |_: MouseEvent| {
            if *running {
                return;
            }
            running.set(true);
            let document = document.clone();
            let count = count.clone();
            let running = running.clone();
            spawn_local(async move {
                let result = crate::services::learning_history::count(&document)
                    .await
                    .map_err(|error| error.to_string());
                count.set(Some(result));
                running.set(false);
            });
        })
    };
    let on_history_delete_confirmation = {
        let confirmation = history_delete_confirmation.clone();
        Callback::from(move |event: InputEvent| {
            confirmation.set(event.target_unchecked_into::<HtmlTextAreaElement>().value());
        })
    };
    let erase_history = {
        let document = environment.document.clone();
        let confirmation = history_delete_confirmation.clone();
        let running = history_delete_running.clone();
        let feedback = history_feedback.clone();
        let count = history_count.clone();
        let summary = history_summary.clone();
        Callback::from(move |_: MouseEvent| {
            if *running || *confirmation != "DELETE HISTORY" {
                return;
            }
            running.set(true);
            let document = document.clone();
            let confirmation = confirmation.clone();
            let running = running.clone();
            let feedback = feedback.clone();
            let count = count.clone();
            let summary = summary.clone();
            spawn_local(async move {
                match crate::services::learning_history::erase(&document).await {
                    Ok(()) => {
                        confirmation.set(String::new());
                        count.set(Some(Ok(0)));
                        summary.set(Some(Ok(zipeek_db::wasm::PracticeOutcomeSummary::default())));
                        feedback.set("All stored Practice learning-history records have been erased. This does not erase active temporary sessions.".into());
                    }
                    Err(error) => {
                        feedback.set(format!("Learning history deletion failed: {error}"))
                    }
                }
                running.set(false);
            });
        })
    };
    html! {
            <div
                class="space-y-3 rounded-xl border border-line p-4"
                data-personal-learning-history=""
            >
                <h3 class="font-semibold">{"Practice learning history"}</h3>
                <label class="flex items-center gap-3">
                    <input
                        type="checkbox"
                        checked={matches!(&*history_enabled, Ok(RecordingPreference::Enabled))}
                        onchange={on_history_toggle}
                        disabled={history_enabled.is_err()}
                        data-personal-learning-history-toggle=""
                    />
                    <span>{"Save completed Practice outcomes to my personal database"}</span>
                </label>
                <p class="text-sm text-muted">
                    {if environment.is_tauri {
                        "Off by default. Practice outcomes are saved in an application-local native DuckDB file. The JSON backup does not include this database."
                    } else {
                        "Off by default. Practice outcomes are saved in a dedicated browser DuckDB/OPFS file. Browser storage can be cleared or evicted; the JSON backup does not include this database."
                    }}
                </p>
                {
                    if let Err(error) = &*history_enabled {
                        html! { <p role="alert">{error.clone()}</p> }
                    } else {
                        Html::default()
                    }
                }
                <div
                    class="flex flex-wrap items-center gap-3"
                >
                    <button
                        type="button"
                        class="rounded-lg border border-line px-4 py-2 text-sm font-semibold disabled:opacity-50"
                        disabled={*history_count_running}
                        onclick={refresh_history_count}
                        data-personal-history-count-refresh=""
                    >
                        {if *history_count_running { "Reading history…" } else { "Count saved Practice sessions" }}
                    </button>
                    <p
                        class="text-sm text-muted"
                        role="status"
                        data-personal-history-count=""
                    >
                        {
                            match &*history_count {
                                None => "No history count requested".to_owned(),
                                Some(Ok(count)) => format!("{count} completed Practice sessions stored"),
                                Some(Err(error)) => format!("Could not count saved Practice sessions: {error}"),
                            }
                        }
                    </p>
                </div>
                <p
                    class="text-xs text-muted"
                >
                    {"This count includes durably committed sessions only. It does not measure vocabulary proficiency or grammar course completion."}
                </p>
                <div
                    class="flex flex-col items-start gap-2"
                >
                    <button
                        type="button"
                        class="rounded-lg border border-line px-4 py-2 text-sm font-semibold disabled:opacity-50"
                        disabled={*history_summary_running}
                        onclick={refresh_history_summary}
                        data-personal-history-summary-refresh=""
                    >
                        {if *history_summary_running { "Reading outcomes…" } else { "Summarize saved Practice outcomes" }}
                    </button>
                    <p
                        class="text-sm text-muted"
                        role="status"
                        data-personal-history-summary=""
                    >
                        {
                            match &*history_summary {
                                None => "No outcome summary requested".to_owned(),
                                Some(Ok(result)) => format!(
                                    "{} saved sessions on {} UTC completion dates: {} scored steps ({} / {} points), {} completed without a score, {} skipped",
                                    result.sessions,
                                    result.utc_completion_dates,
                                    result.scored_steps,
                                    result.earned_points,
                                    result.possible_points,
                                    result.completed_steps,
                                    result.skipped_steps,
                                ),
                                Some(Err(error)) => format!("Could not summarize saved Practice outcomes: {error}"),
                            }
                        }
                    </p>
                </div>
                <p
                    class="text-xs text-muted"
                >
                    {"UTC completion dates are not learner-local active days or proficiency evidence. These records count session outcomes, not distinct words, skill-specific mastery or grammar course completion. Unscored completion does not imply a correct answer."}
                </p>

                <label class="text-sm font-semibold" for="delete-learning-history-confirmation">
                    {"To permanently erase saved Practice history, enter DELETE HISTORY"}
                </label>
                <textarea
                    id="delete-learning-history-confirmation"
                    class="w-full rounded-lg border border-line bg-surface p-2"
                    rows="1"
                    value={(*history_delete_confirmation).clone()}
                    oninput={on_history_delete_confirmation}
                    data-personal-history-delete-confirmation=""
                />
                <button
                    type="button"
                    class="self-start rounded-lg border border-line px-4 py-2 font-semibold disabled:opacity-50"
                    disabled={*history_delete_running || *history_delete_confirmation != "DELETE HISTORY"}
                    onclick={erase_history}
                    data-personal-history-delete=""
                >
                    {"Erase saved Practice history"}
                </button>
                {
                    if history_feedback.is_empty() {
                        Html::default()
                    } else {
                        html! { <p role="status">{(*history_feedback).clone()}</p> }
                    }
                }
            </div>
    }
}
