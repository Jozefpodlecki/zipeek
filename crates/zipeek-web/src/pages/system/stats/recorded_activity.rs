use std::{cell::Cell, rc::Rc};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::{hooks::use_app_environment, services::learning_history};

#[derive(Deserialize)]
struct SavedSession {
    completed_at_ms: i64,
    steps: Vec<SavedStep>,
}

#[derive(Deserialize)]
struct SavedStep {
    exercise_id: String,
    outcome: SavedOutcome,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SavedOutcome {
    Completed,
    Skipped,
    Scored { earned: u16, possible: u16 },
}

struct ActivityRow {
    id: String,
    date: String,
    exercises: usize,
    completed: usize,
    skipped: usize,
    scored: usize,
    focus: String,
}

struct ActivitySnapshot {
    sessions: usize,
    attempts: usize,
    completed: usize,
    skipped: usize,
    rows: Vec<ActivityRow>,
}

fn decode_rows(rows: Vec<zipeek_db::contracts::HistoryRow>) -> Result<ActivitySnapshot, String> {
    let mut result = ActivitySnapshot {
        sessions: rows.len(),
        attempts: 0,
        completed: 0,
        skipped: 0,
        rows: Vec::with_capacity(rows.len()),
    };
    for row in rows {
        let saved: SavedSession = serde_json::from_str(&row.evidence)
            .map_err(|_| "Invalid saved Practice evidence".to_owned())?;
        let completed_at = DateTime::<Utc>::from_timestamp_millis(saved.completed_at_ms)
            .ok_or_else(|| "Invalid Practice completion timestamp".to_owned())?;
        let mut completed = 0;
        let mut skipped = 0;
        let mut scored = 0;
        let mut exercises = std::collections::BTreeSet::new();
        for step in &saved.steps {
            exercises.insert(step.exercise_id.as_str());
            match step.outcome {
                SavedOutcome::Completed => completed += 1,
                SavedOutcome::Skipped => skipped += 1,
                SavedOutcome::Scored { earned, possible } => {
                    if earned > possible {
                        return Err("Invalid scored Practice evidence".to_owned());
                    }
                    completed += 1;
                    scored += 1;
                }
            }
        }
        result.attempts += saved.steps.len();
        result.completed += completed;
        result.skipped += skipped;
        result.rows.push(ActivityRow {
            id: row.session_id,
            date: completed_at.format("%Y-%m-%d %H:%M UTC").to_string(),
            exercises: saved.steps.len(),
            completed,
            skipped,
            scored,
            focus: exercises.into_iter().collect::<Vec<_>>().join(", "),
        });
    }
    result
        .rows
        .sort_by(|a, b| b.date.cmp(&a.date).then(a.id.cmp(&b.id)));
    Ok(result)
}

#[function_component(RecordedActivity)]
pub(super) fn recorded_activity() -> Html {
    let environment = use_app_environment();
    let state = use_state(|| None::<Result<ActivitySnapshot, String>>);
    let retry = use_state(|| 0u32);
    {
        let document = environment.document.clone();
        let state = state.clone();
        use_effect_with(*retry, move |_| {
            let alive = Rc::new(Cell::new(true));
            let active = alive.clone();
            state.set(None);
            spawn_local(async move {
                let result = learning_history::recorded_rows(&document)
                    .await
                    .and_then(decode_rows);
                if active.get() {
                    state.set(Some(result));
                }
            });
            move || alive.set(false)
        });
    }
    let on_refresh = {
        let retry = retry.clone();
        Callback::from(move |_: MouseEvent| retry.set(retry.wrapping_add(1)))
    };
    html! {
        <section class="space-y-4 rounded-2xl border border-line bg-surface p-4 sm:p-6" data-stats-recorded-activity="">
            <div class="flex items-center justify-between gap-3">
                <h2 class="text-lg font-semibold text-ink">{"Recorded practice activity"}</h2>
                <button type="button" onclick={on_refresh} aria-label="Refresh saved activity" title="Refresh saved activity"
                    class="inline-flex size-10 items-center justify-center rounded-lg border border-line text-ink hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_REFRESH_CW} width="18" height="18" />
                </button>
            </div>
            {match &*state {
                None => html! { <p role="status" class="text-sm text-muted">{"Loading saved practice activity…"}</p> },
                Some(Err(error)) => html! { <p role="alert" class="text-sm text-rose-500">{error.to_string()}</p> },
                Some(Ok(snapshot)) if snapshot.sessions == 0 => html! {
                    <p class="text-sm text-muted">{"No saved practice activity yet. Enable Save Practice outcomes in Learning settings to record future sessions."}</p>
                },
                Some(Ok(snapshot)) => html! {
                    <>
                        <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
                            {for [
                                ("Sessions", snapshot.sessions),
                                ("Exercises", snapshot.attempts),
                                ("Completed", snapshot.completed),
                                ("Skipped", snapshot.skipped),
                            ].into_iter().map(|(label, value)| html! {
                                <div class="rounded-xl bg-raised p-3" key={label}>
                                    <p class="text-xs text-muted">{label}</p>
                                    <p class="mt-1 text-xl font-semibold tabular-nums text-ink">{value}</p>
                                </div>
                            })}
                        </div>
                        <div class="max-h-72 overflow-y-auto rounded-xl border border-line" data-stats-recorded-history="">
                            {for snapshot.rows.iter().take(50).map(|row| html! {
                                <div key={row.id.clone()} class="flex flex-col gap-1 border-b border-line p-3 last:border-b-0 sm:flex-row sm:items-center sm:justify-between">
                                    <div class="min-w-0">
                                        <p class="text-sm font-medium text-ink">{&row.date}</p>
                                        <p class="truncate text-xs text-muted" title={row.focus.clone()}>{&row.focus}</p>
                                    </div>
                                    <span class="text-xs tabular-nums text-muted">
                                        {format!("{} exercises · {} completed · {} skipped · {} scored", row.exercises, row.completed, row.skipped, row.scored)}
                                    </span>
                                </div>
                            })}
                        </div>
                        <p class="text-xs text-muted">{"Activity reflects saved sessions, not verified mastery."}</p>
                    </>
                },
            }}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_sessions_are_activity_not_mastery() {
        let id = "123e4567-e89b-72d3-a456-426614174001";
        let json = format!(
            r#"{{"schema_version":2,"session_id":"{id}","completed_at_ms":1795700000000,"steps":[{{"step_id":"{id}:0","exercise_id":"character-builder","outcome":"completed"}}]}}"#
        );
        let snapshot = decode_rows(vec![zipeek_db::contracts::HistoryRow {
            session_id: id.into(),
            evidence: json,
        }])
        .unwrap();
        assert_eq!(
            (snapshot.sessions, snapshot.attempts, snapshot.completed),
            (1, 1, 1)
        );
        assert!(snapshot.rows[0].focus.contains("character-builder"));
    }
}
