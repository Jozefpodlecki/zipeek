use std::{cell::Cell, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::AppEnvironment;

use super::{context::use_loaded_section, data::{SectionPayload, SessionRecord}, fixture::EvidenceStatus, sections::OverviewCards, state::MetricSelection};
use crate::services::charts;

const WEEKS: usize = 5;
const ACTIVITY_WEEKS: [&str; WEEKS + 1] = [
    "2026-10-15", "2026-10-22", "2026-10-29", "2026-11-05", "2026-11-12", "2026-11-19",
];

fn activity_buckets(sessions: &[SessionRecord]) -> [u32; WEEKS] {
    let mut counts = [0; WEEKS];
    for session in sessions {
        for index in 0..WEEKS {
            if session.date.as_str() >= ACTIVITY_WEEKS[index] && session.date.as_str() < ACTIVITY_WEEKS[index + 1] {
                counts[index] += 1;
                break;
            }
        }
    }
    counts
}

#[function_component(ActivityChart)]
fn activity_chart() -> Html {
    let loaded = use_loaded_section();
    let SectionPayload::Overview { sessions, .. } = loaded.as_ref() else { unreachable!("Overview projection expected") };
    let buckets = activity_buckets(sessions);
    let canvas = use_node_ref();
    let environment = use_context::<AppEnvironment>().expect("Statistics requires AppEnvironment");
    let chart = use_mut_ref(|| None::<zipeek_browser::charts::ChartInstance>);
    let error = use_state(|| false);
    let ready = use_state(|| false);
    {
        let canvas = canvas.clone();
        let window = environment.window.clone();
        let bucket_values = buckets;
        let chart = chart.clone();
        let error = error.clone();
        let ready = ready.clone();
        use_effect_with(bucket_values, move |_| {
            let alive = Rc::new(Cell::new(true));
            if let Some(element) = canvas.cast::<HtmlCanvasElement>() {
                let accent = window.get_computed_style(&element)
                    .ok()
                    .flatten()
                    .and_then(|style| style.get_property_value("--accent").ok())
                    .filter(|color| !color.trim().is_empty())
                    .unwrap_or_else(|| "#8062d0".to_owned());
                let chart_for_task = chart.clone();
                let alive_for_task = alive.clone();
                spawn_local(async move {
                    match charts::stats_activity(&element, &bucket_values, &accent).await {
                        Ok(instance) if alive_for_task.get() => { *chart_for_task.borrow_mut() = Some(instance); ready.set(true); }
                        Ok(_) => {}
                        Err(reason) if alive_for_task.get() => {
                            log::warn!("Statistics activity chart failed: {reason:?}");
                            error.set(true);
                        },
                        Err(_) => {}
                    }
                });
            }
            move || {
                alive.set(false);
                chart.borrow_mut().take();
            }
        });
    }
    html! {
        <article class="min-w-0 rounded-xl border border-line bg-surface p-4" data-stats-activity-chart="">
            <h2 class="font-semibold">{"Practice activity"}</h2>
            <div class="relative mt-3 h-40" aria-hidden="true">
                <canvas ref={canvas} class={if *ready {"h-full w-full"} else {"invisible h-full w-full"}} />
                if !*ready {
                    <div class="absolute inset-0 flex items-end justify-around gap-3 px-5 pb-6 pt-2">
                        {for buckets.iter().enumerate().map(|(index, count)| {
                            let maximum = f64::from(*buckets.iter().max().unwrap_or(&1)).max(1.0);
                            let height = (f64::from(*count) / maximum * 100.0).clamp(8.0, 100.0);
                            html! {
                                <div key={index} class="flex h-full min-w-0 flex-1 flex-col items-center justify-end gap-1">
                                    <div class="w-full max-w-12 rounded-t-md bg-accent/75" style={format!("height: {height:.1}%")} />
                                    <span class="text-xs text-muted">{ACTIVITY_WEEKS[index].trim_start_matches("2026-")}</span>
                                </div>
                            }
                        })}
                    </div>
                }
            </div>
            if *error {
                <p class="text-xs text-muted">{"Chart unavailable; activity is listed below."}</p>
            }
            <details class="mt-2 text-xs text-muted">
                <summary class="cursor-pointer rounded-md py-1 focus-visible:outline-2 focus-visible:outline-accent">{"Activity data"}</summary>
                <table class="mt-2 w-full text-left"><caption class="sr-only">{"Fictional completed sessions grouped into five seven-day periods"}</caption>
                    <thead><tr><th scope="col">{"Period"}</th><th scope="col">{"Sessions"}</th></tr></thead>
                    <tbody>{for buckets.iter().enumerate().map(|(index, value)| html! {
                        <tr key={index}><th scope="row">{ACTIVITY_WEEKS[index]}</th><td>{value}</td></tr>
                    })}</tbody>
                </table>
            </details>
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct OverviewProps {
    pub metrics: MetricSelection,
    pub on_vocabulary: Callback<()>,
}

#[function_component(Overview)]
pub(super) fn overview(props: &OverviewProps) -> Html {
    let on_vocabulary = props.on_vocabulary.clone();
    let open_vocabulary = Callback::from(move |_: MouseEvent| on_vocabulary.emit(()));
    let loaded = use_loaded_section();
    let SectionPayload::Overview { words, .. } = loaded.as_ref() else { unreachable!("Overview projection expected") };
    let review = words.iter().filter(|record| {
        [record.recognition, record.listening, record.speaking, record.writing, record.sentence]
            .contains(&EvidenceStatus::NeedsReview)
    }).take(4).collect::<Vec<_>>();
    html! {
        <div class="space-y-3" data-stats-overview="">
            <OverviewCards words={props.metrics.words} encounters={props.metrics.encounters} sessions={props.metrics.sessions} assessment={props.metrics.coverage} />
            <div class="grid items-start gap-3 lg:grid-cols-[minmax(0,3fr)_minmax(16rem,2fr)]">
                <ActivityChart />
                <section class="min-w-0 rounded-xl border border-line bg-surface p-4" aria-label="Items for review">
                    <div class="flex items-center justify-between gap-2">
                        <h2 class="font-semibold">{"Review candidates"}</h2>
                        <button type="button" class="rounded-md p-2 text-accent hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent" aria-label="Browse vocabulary" onclick={open_vocabulary}>
                            <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="18px" height="18px" />
                        </button>
                    </div>
                    <div class="mt-2 divide-y divide-line">
                        {for review.iter().map(|record| html! {
                            <div key={record.id.clone()} class="flex items-center justify-between gap-3 py-2">
                                <div><span lang="zh-Hans" class="font-semibold">{record.hanzi.clone()}</span>
                                    <span class="ml-2 text-xs text-muted">{record.pinyin.clone()}</span>
                                </div>
                                <span class="text-xs text-muted">{"Review"}</span>
                            </div>
                        })}
                    </div>
                </section>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn activity_buckets_cover_all_fixture_sessions() {
        let sessions = super::super::data::demo_sessions_for_tests();
        let buckets = activity_buckets(&sessions);
        assert!(buckets.iter().sum::<u32>() as usize <= sessions.len());
        assert!(buckets.iter().any(|count| *count > 0));
        assert!(buckets.windows(2).any(|pair| pair[0] != pair[1]));
    }
}
