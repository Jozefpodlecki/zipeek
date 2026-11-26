use super::WORD_SET_SIZE;
use super::loading::{LoadingPhase, RegionLoading};
use super::state::RegionLoad;
use crate::hooks::use_word_sets;
use crate::reducers::word_sets::WordSetsFeedback;
use crate::services::word_sets::{CourseWordSetSource, acquired_source};
use yew::prelude::*;
use zipeek_pedagogy::curriculum::VOCABULARY_REGIONS as COURSES;

#[derive(Properties, PartialEq)]
pub struct RegionDetailsProps {
    pub selected: usize,
    pub load: RegionLoad,
    pub on_retry: Callback<MouseEvent>,
    pub on_acquire: Callback<usize>,
}

#[function_component(RegionDetails)]
pub fn region_details(props: &RegionDetailsProps) -> Html {
    let sets = use_word_sets();
    html! {
                <aside data-vocabulary-details="" aria-live="polite" class="min-w-0 overflow-hidden rounded-2xl border border-line bg-surface shadow-lg lg:h-full">
                    if let Some(course) = COURSES.get(props.selected) {
                        <div class="space-y-3 p-4 sm:p-5">
                            <header class="space-y-2">
                                <p class="text-xs font-bold uppercase tracking-widest text-accent">{course.domain}</p>
                                <h2 class="text-xl font-bold leading-tight text-ink">{course.title}</h2>
                                <p class="text-sm leading-relaxed text-muted">{course.description.split(". Pinned").next().unwrap_or(course.description)}</p>
                            </header>
                            <div class="grid grid-cols-2 gap-2" aria-label="Collection size">
                                <div class="rounded-xl border border-line bg-raised p-3">
                                    <p class="text-2xl font-bold tabular-nums text-ink">{course.readings.len()}</p>
                                    <p class="text-xs text-muted">{"HSK readings"}</p>
                                </div>
                                <div class="rounded-xl border border-line bg-raised p-3">
                                    <p class="text-2xl font-bold tabular-nums text-ink">{course.readings.len().div_ceil(WORD_SET_SIZE)}</p>
                                    <p class="text-xs text-muted">{"Word Sets"}</p>
                                </div>
                            </div>
                            {match &props.load {
                                RegionLoad::WaitingForDictionary => html! { <RegionLoading phase={LoadingPhase::Dictionary} /> },
                                RegionLoad::Loading => html! { <RegionLoading phase={LoadingPhase::Vocabulary} /> },
                                RegionLoad::DictionaryUnavailable(error) => html! {
                                    <div class="space-y-3 rounded-xl border border-line p-4">
                                        <p role="alert" class="text-sm text-red-500">{format!("Dictionary initialization failed: {error}")}</p>
                                        <button type="button" class="rounded-lg border border-line px-4 py-2 text-sm font-semibold text-ink" onclick={props.on_retry.clone()}> {"Retry dictionary"} </button>
                                    </div>
                                },
                                RegionLoad::Failed(error) => html! {
                                    <div class="space-y-3 rounded-xl border border-line p-4">
                                        <p role="alert" class="text-sm text-red-500">{error}</p>
                                        <button type="button" class="rounded-lg border border-line px-4 py-2 text-sm font-semibold text-ink" onclick={props.on_retry.clone()}>{"Retry loading"}</button>
                                    </div>
                                },
                                RegionLoad::Ready(words) => {
                                    let total = words.len().div_ceil(WORD_SET_SIZE);
                                    let acquired = match &sets.state.sets {
                                        Ok(items) => (0..total)
                                            .filter(|&index| acquired_source(items, &CourseWordSetSource { region_id: (props.selected + 1) as u16, set_index: (index + 1) as u16 }))
                                            .count(),
                                        Err(_) => 0,
                                    };
                                    let next = (0..total).find(|&index| match &sets.state.sets {
                                        Ok(items) => !acquired_source(items, &CourseWordSetSource { region_id: (props.selected + 1) as u16, set_index: (index + 1) as u16 }),
                                        Err(_) => false,
                                    });
                                    let next_action = next.map(|index| {
                                        let on_acquire = props.on_acquire.clone();
                                        Callback::from(move |_: MouseEvent| on_acquire.emit(index))
                                    });
                                    html! {
                                        <div class="space-y-3">
                                            <div class="space-y-2">
                                                <div class="flex items-center justify-between gap-3 text-sm">
                                                    <span class="font-semibold text-ink">{"Your Word Sets"}</span>
                                                    <span class="tabular-nums text-muted">{format!("{acquired} of {total} acquired")}</span>
                                                </div>
                                                <div class="h-2 overflow-hidden rounded-full bg-raised" role="progressbar" aria-label="Word Sets acquired" aria-valuemin="0" aria-valuemax={total.to_string()} aria-valuenow={acquired.to_string()}>
                                                    <div class="h-full rounded-full bg-brand-action transition-[width]" style={format!("width: {}%", acquired * 100 / total)}></div>
                                                </div>
                                            </div>
                                            {match sets.state.course_feedback.as_ref()
                                                .filter(|(source, _)| source.region_id == course.id)
                                                .map(|(_, feedback)| feedback)
                                            {
                                                Some(WordSetsFeedback::Success(message)) => html! { <p role="status" class="text-sm text-ink">{message}</p> },
                                                Some(WordSetsFeedback::Error(message)) => html! { <p role="alert" class="text-sm text-red-500">{message}</p> },
                                                None => Html::default(),
                                            }}
                                            if sets.state.sets.is_err() {
                                                <p role="alert" class="rounded-xl border border-line p-3 text-sm text-red-500">{"Word Sets storage is unavailable. Reload to retry."}</p>
                                            } else if let Some(onclick) = next_action {
                                                <button data-course-create-word-set="" type="button" class="w-full rounded-xl bg-brand-action px-4 py-3 text-sm font-bold text-white shadow-sm transition-colors hover:bg-brand-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action" {onclick}>
                                                    {format!("Acquire Word Set {} of {}", next.unwrap_or(0) + 1, total)}
                                                </button>
                                            } else {
                                                <p data-course-word-set-existing="" role="status" class="rounded-xl border border-line bg-raised p-3 text-sm font-semibold text-ink">{"All Word Sets acquired"}</p>
                                            }
                                            <div class="space-y-2 border-t border-line pt-3" aria-label="Word Set acquisition roadmap">
                                                <p class="text-xs font-semibold uppercase tracking-wide text-muted">{"Collection roadmap"}</p>
                                                <ol class="grid grid-cols-2 gap-2">
                                                    {for (0..total).map(|index| {
                                                        let source = CourseWordSetSource {
                                                            region_id: course.id,
                                                            set_index: (index + 1) as u16,
                                                        };
                                                        let is_acquired = match &sets.state.sets {
                                                            Ok(items) => acquired_source(items, &source),
                                                            Err(_) => false,
                                                        };
                                                        let from = index * WORD_SET_SIZE + 1;
                                                        let to = ((index + 1) * WORD_SET_SIZE).min(words.len());
                                                        html! {
                                                            <li key={index} class="flex min-w-0 items-center justify-between gap-1 rounded-lg border border-line bg-raised px-2 py-2 text-xs">
                                                                <span class="min-w-0 truncate font-semibold text-ink" title={format!("Words {from}-{to}")}>{format!("Set {}", index + 1)}</span>
                                                                <span class={if is_acquired {"font-semibold text-accent"} else {"text-muted"}}>
                                                                    {if is_acquired {"Acquired"} else {"Available"}}
                                                                </span>
                                                            </li>
                                                        }
                                                    })}
                                                </ol>
                                            </div>
                                        </div>
                                    }
                                },
                            }}

                        </div>
                    } else {
                        <p class="p-5 text-sm text-muted">{"Vocabulary collection unavailable."}</p>
                    }
                </aside>
    }
}
