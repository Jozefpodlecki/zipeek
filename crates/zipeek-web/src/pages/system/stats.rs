mod context;
mod data;
mod controls;
mod fixture;
mod explorer;
mod grammar;
mod history;
mod insights;
mod skills;
mod overview;
mod sections;
mod state;

use gloo::timers::callback::Timeout;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use context::{LoadedSectionContext, StatsDataContext, StatsSourceContext, StatsUiContext, use_stats_ui};
use data::{DataAction, DemoStatisticsSource, SectionLoad, StatisticsDataState, StatisticsSource};
use explorer::VocabularyExplorer;
use grammar::GrammarExplorer;
use history::HistoryExplorer;
use insights::InsightExplorer;
use skills::SkillsExplorer;
use overview::Overview;
use sections::DashboardTab;
use state::{DashboardAction, DashboardPreset, DashboardState, MetricId, PreviewPhase};

#[derive(Properties, PartialEq)]
struct MetricSwitchProps {
    name: &'static str,
    metric: MetricId,
}

#[function_component(MetricSwitch)]
fn metric_switch(props: &MetricSwitchProps) -> Html {
    let dashboard = use_stats_ui();
    let checked = dashboard.0.metrics.is_enabled(props.metric);
    let onclick = {
        let state = dashboard.0.clone();
        let metric = props.metric;
        Callback::from(move |_: MouseEvent| state.dispatch(DashboardAction::ToggleMetric(metric)))
    };
    html! {
        <button type="button" role="switch" aria-checked={checked.to_string()} onclick={onclick}
            class="flex min-h-12 w-full items-center justify-between gap-4 rounded-xl border border-line bg-raised px-4 py-3 text-left text-sm font-medium transition-colors hover:bg-surface focus-visible:outline-2 focus-visible:outline-accent">
            <span>{props.name}</span>
            <span aria-hidden="true" class={classes!("relative", "inline-flex", "h-6", "w-11", "shrink-0", "rounded-full", "p-1", "transition-colors", if checked { "bg-brand-action" } else { "bg-line" })}>
                <span class={classes!("h-4", "w-4", "rounded-full", "bg-white", "shadow-sm", "transition-transform", if checked { "translate-x-5" } else { "translate-x-0" })} />
            </span>
        </button>
    }
}

#[function_component(StatsContent)]
fn stats_content() -> Html {
    let context = use_stats_ui();
    let state = context.0.clone();
    let timeout = use_mut_ref(|| None::<Timeout>);
    let data = use_context::<StatsDataContext>().expect("Statistics data provider is required");
    {
        let timeout = timeout.clone();
        use_effect_with((), move |_| move || { timeout.borrow_mut().take(); });
    }
    let on_preview = {
        let state = state.clone();
        let timeout = timeout.clone();
        let data = data.clone();
        Callback::from(move |_: MouseEvent| {
            match state.phase {
                PreviewPhase::Closed => {
                    timeout.borrow_mut().take();
                    let request_id = state.request_id.wrapping_add(1);
                    state.dispatch(DashboardAction::RequestPreview);
                    let state = state.clone();
                    timeout.borrow_mut().replace(Timeout::new(2_000, move || {
                        state.dispatch(DashboardAction::PreviewReady(request_id));
                    }));
                }
                PreviewPhase::Loading { .. } => {
                    timeout.borrow_mut().take();
                    data.0.dispatch(DataAction::Reset);
                    state.dispatch(DashboardAction::ClosePreview);
                }
                PreviewPhase::Open => {
                    timeout.borrow_mut().take();
                    data.0.dispatch(DataAction::Reset);
                    state.dispatch(DashboardAction::ClosePreview);
                }
            }
        })
    };
    let tabs = DashboardTab::ALL.into_iter().map(|target| {
        let selected = state.tab == target;
        let state = state.clone();
        let onclick = Callback::from(move |_: MouseEvent| state.dispatch(DashboardAction::SelectTab(target)));
        html! {
            <button key={target.key()} type="button" role="tab" data-stats-tab={target.key()}
                aria-selected={selected.to_string()} onclick={onclick}
                class={classes!("inline-flex", "min-h-10", "items-center", "gap-2", "rounded-lg", "px-3", "text-sm", "font-semibold", "focus-visible:outline-2", "focus-visible:outline-accent", if selected {"bg-brand-action text-white"} else {"border border-line bg-surface text-ink hover:bg-raised"})}>
                <Icon data={target.icon()} width="17px" height="17px" />
                {target.label()}
            </button>
        }
    });
    html! {
        <main class="mx-auto flex w-full max-w-6xl flex-col gap-3 px-4 py-4 sm:py-6" data-stats-page="">
            <header class="flex flex-wrap items-center justify-between gap-3">
                <div class="flex items-center gap-2">
                    <h1 class="sr-only">{"Statistics"}</h1>
                    if state.phase == PreviewPhase::Open {
                        <span data-stats-demo="" title="Demonstration figures, not your learning activity or proficiency assessment"
                            class="inline-flex items-center gap-1 rounded-full border border-line bg-raised px-3 py-1 text-xs font-semibold">
                            <Icon data={IconData::LUCIDE_EYE} width="14px" height="14px" />
                            {"Demo"}
                        </span>
                    }
                </div>
                <button type="button" data-stats-preview-toggle="" aria-pressed={(state.phase == PreviewPhase::Open).to_string()}
                    onclick={on_preview}
                    class="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line bg-surface px-3 text-sm font-semibold hover:bg-raised disabled:opacity-60 focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_EYE} width="17px" height="17px" />
                    {match state.phase { PreviewPhase::Open => "Close preview", PreviewPhase::Loading { .. } => "Cancel preview", PreviewPhase::Closed => "Preview dashboard" }}
                </button>
            </header>
            {match state.phase {
                PreviewPhase::Closed => html! {
                    <section data-stats-unavailable="" class="rounded-xl border border-line bg-surface p-5" aria-labelledby="stats-data-status-heading">
                        <h2 id="stats-data-status-heading" class="font-semibold">{"Learning evidence not connected"}</h2>
                        <p class="mt-2 text-sm text-muted">{"Verified learning observations are needed before proficiency can be assessed."}</p>
                        <a href="#/settings?section=learning" class="mt-3 inline-flex min-h-10 items-center rounded-lg border border-line px-3 text-sm font-semibold hover:bg-raised">{"Learning Data settings"}</a>
                    </section>
                },
                PreviewPhase::Loading { .. } => html! {
                    <section data-stats-loading="" role="status" aria-live="polite" aria-busy="true" class="space-y-3 motion-reduce:animate-none">
                        <span class="sr-only">{"Preparing dashboard preview"}</span>
                        <div class="grid grid-cols-3 gap-3">
                            {for (0..3).map(|index| html! { <div key={index} class="h-24 animate-pulse rounded-xl border border-line bg-raised motion-reduce:animate-none" aria-hidden="true" /> })}
                        </div>
                        <div class="grid gap-3 lg:grid-cols-[3fr_2fr]">
                            <div class="h-48 animate-pulse rounded-xl border border-line bg-raised motion-reduce:animate-none" aria-hidden="true" />
                            <div class="h-48 animate-pulse rounded-xl border border-line bg-raised motion-reduce:animate-none" aria-hidden="true" />
                        </div>
                    </section>
                },
                PreviewPhase::Open => html! {
                    <>
                        <div role="tablist" aria-label="Statistics sections" class="flex flex-wrap gap-2">{for tabs}</div>
                        <section data-stats-panel={state.tab.key()} role="tabpanel" aria-label={state.tab.label()} class="min-w-0 space-y-3">
                            <StatsSection tab={state.tab} />
                        </section>
                        if state.selection_review {
                            <SelectionReview />
                        }
                    </>
                },
            }}
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct StatsSectionProps {
    tab: DashboardTab,
}

#[function_component(StatsSection)]
fn stats_section(props: &StatsSectionProps) -> Html {
    let ui = use_stats_ui().0;
    let data = use_context::<StatsDataContext>().expect("Statistics data provider is required").0;
    let source = use_context::<StatsSourceContext>().expect("Statistics source provider is required").0;
    let on_vocabulary = {
        let ui = ui.clone();
        Callback::from(move |_: ()| ui.dispatch(DashboardAction::SelectTab(DashboardTab::Hanzi)))
    };
    let retry_epoch = use_state(|| 0_u64);
    let generation = use_mut_ref(|| 0_u64);
    {
        let data = data.clone();
        let source = source.clone();
        let generation = generation.clone();
        use_effect_with((props.tab, *retry_epoch), move |(tab, _)| {
            let tab = *tab;
            let alive = Rc::new(Cell::new(true));
            let active_generation = {
                let mut generation = generation.borrow_mut();
                *generation = generation.wrapping_add(1);
                *generation
            };
            if !matches!(data.get(tab), Some(SectionLoad::Ready(_))) {
                data.dispatch(DataAction::Start { tab, generation: active_generation });
                let data_for_task = data.clone();
                let alive_for_task = alive.clone();
                let request = source.load(tab);
                spawn_local(async move {
                    let result = request.await;
                    if !alive_for_task.get() {
                        return;
                    }
                    match result {
                        Ok(payload) => data_for_task.dispatch(DataAction::Loaded {
                            tab, generation: active_generation, payload,
                        }),
                        Err(error) => data_for_task.dispatch(DataAction::Failed {
                            tab, generation: active_generation, error,
                        }),
                    }
                });
            }
            move || {
                alive.set(false);
                data.dispatch(DataAction::Invalidate { tab, generation: active_generation });
            }
        });
    }
    let on_retry = {
        let retry_epoch = retry_epoch.clone();
        Callback::from(move |_: MouseEvent| retry_epoch.set((*retry_epoch).wrapping_add(1)))
    };
    match data.get(props.tab) {
        Some(SectionLoad::Ready(payload)) => html! {
            <ContextProvider<LoadedSectionContext> context={LoadedSectionContext(payload.clone())}>
                {match props.tab {
                                DashboardTab::Overview => html! { <Overview metrics={ui.metrics} on_vocabulary={on_vocabulary} /> },
                                DashboardTab::Hanzi => html! { <VocabularyExplorer /> },
                                DashboardTab::Grammar => html! { <GrammarExplorer /> },
                                DashboardTab::Skills => html! { <SkillsExplorer /> },
                                DashboardTab::History => html! { <HistoryExplorer /> },
                                DashboardTab::Insights => html! { <InsightExplorer /> },
                                DashboardTab::Settings => html! {
                                    <section class="rounded-xl border border-line bg-surface p-4" data-stats-settings="">
                                        <h2 class="mb-3 font-semibold">{"Overview cards"}</h2>
                                        <div class="mb-4 flex flex-wrap gap-2" role="group" aria-label="Dashboard card presets">
                                            {for [(DashboardPreset::Focused, "Focused"), (DashboardPreset::Activity, "Activity"), (DashboardPreset::Detailed, "Detailed")]
                                                .into_iter().map(|(preset, label)| {
                                                    let dashboard = ui.clone();
                                                    let onclick = Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::SetMetricPreset(preset)));
                                                    let active = ui.metrics == preset.metrics();
                                                    html! {
                                                        <button key={label} type="button" aria-pressed={active.to_string()} onclick={onclick}
                                                            class={classes!("min-h-10", "rounded-lg", "border", "px-4", "text-sm", "font-semibold", "focus-visible:outline-2", "focus-visible:outline-accent", if active {"border-accent bg-brand-action text-white"} else {"border-line bg-raised hover:bg-surface"})}>
                                                            {label}
                                                        </button>
                                                    }
                                                })}
                                        </div>
                                        <div class="grid gap-3 sm:grid-cols-2">
                                            <MetricSwitch name="Words observed" metric={MetricId::Words}/>
                                            <MetricSwitch name="Encounters" metric={MetricId::Encounters}/>
                                            <MetricSwitch name="Vocabulary session appearances" metric={MetricId::Sessions}/>
                                            <MetricSwitch name="Skill coverage" metric={MetricId::Coverage}/>
                                        </div>
                                    </section>
                                },
                }}
            </ContextProvider<LoadedSectionContext>>
        },
        Some(SectionLoad::Failed { error, .. }) => html! {
            <div role="alert" class="rounded-xl border border-line bg-surface p-5" data-stats-section-error="">
                <p class="font-semibold">{"This section could not be loaded"}</p>
                <p class="mt-1 text-sm text-muted">{error.to_string()}</p>
                <button type="button" onclick={on_retry} class="mt-3 min-h-10 rounded-lg bg-brand-action px-4 text-sm font-semibold text-white">{"Retry"}</button>
            </div>
        },
        _ => html! {
            <div role="status" aria-live="polite" aria-busy="true" data-stats-section-loading={props.tab.key()} class="space-y-3">
                <span class="sr-only">{format!("Loading {}", props.tab.label())}</span>
                <div class="grid gap-3 sm:grid-cols-3">
                    {for (0..3).map(|index| html! { <div key={index} class="h-24 animate-pulse rounded-xl border border-line bg-raised motion-reduce:animate-none" aria-hidden="true" /> })}
                </div>
                <div class="h-56 animate-pulse rounded-xl border border-line bg-raised motion-reduce:animate-none" aria-hidden="true" />
            </div>
        },
    }
}


#[function_component(SelectionReview)]
fn selection_review() -> Html {
    let dashboard = use_stats_ui().0;
    let close = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::ReviewSelection(false)))
    };
    html! {
        <section aria-labelledby="stats-selection-heading" class="mt-3 w-full rounded-xl border border-line bg-surface p-4 shadow-lg" data-stats-selection-review="">
            <div class="flex items-center justify-between gap-3">
                <h2 id="stats-selection-heading" class="font-semibold">{"Review selection"}</h2>
                <button type="button" aria-label="Close selection review" onclick={close.clone()} class="rounded-lg border border-line p-2"><Icon data={IconData::LUCIDE_X} width="18px" /></button>
            </div>
            <div class="mt-3 max-h-32 overflow-y-auto text-sm">
                if !dashboard.selected_words.is_empty() {
                    <p class="font-semibold">{format!("{} words", dashboard.selected_words.len())}</p>
                    <p lang="zh-Hans" class="break-words">{dashboard.selected_words.iter().take(12).cloned().collect::<Vec<_>>().join(" · ")}</p>
                    if dashboard.selected_words.len() > 12 {
                        <p class="text-xs text-muted">{format!("And {} more selected words", dashboard.selected_words.len() - 12)}</p>
                    }
                }
                if !dashboard.selected_patterns.is_empty() {
                    <p class="mt-2 text-muted">{format!("{} grammar patterns selected", dashboard.selected_patterns.len())}</p>
                    <p class="text-xs text-muted">{dashboard.selected_patterns.iter().take(12).cloned().collect::<Vec<_>>().join(" · ")}</p>
                    if dashboard.selected_patterns.len() > 12 {
                        <p class="text-xs text-muted">{format!("And {} more selected patterns", dashboard.selected_patterns.len() - 12)}</p>
                    }
                }
            </div>
            <p class="mt-3 text-xs text-muted">{"Demo selection only. Opening Practice does not transfer these items."}</p>
            <div class="mt-3 flex justify-end gap-2">
                <button type="button" onclick={close} class="min-h-10 rounded-lg border border-line px-4 text-sm">{"Continue exploring"}</button>
                <a href="#/practice" class="inline-flex min-h-10 items-center rounded-lg bg-brand-action px-4 text-sm font-semibold text-white">{"Open Practice setup"}</a>
            </div>
        </section>
    }
}

#[function_component(StatsPage)]
pub fn stats_page() -> Html {
    let state = use_reducer(DashboardState::default);
    let data = use_reducer(StatisticsDataState::default);
    let source = use_state(|| {
        let source: Rc<dyn StatisticsSource> = Rc::new(DemoStatisticsSource);
        StatsSourceContext(source)
    });
    html! {
        <ContextProvider<StatsSourceContext> context={(*source).clone()}>
            <ContextProvider<StatsDataContext> context={StatsDataContext(data)}>
                <ContextProvider<StatsUiContext> context={StatsUiContext(state)}>
                    <StatsContent />
                </ContextProvider<StatsUiContext>>
            </ContextProvider<StatsDataContext>>
        </ContextProvider<StatsSourceContext>>
    }
}
