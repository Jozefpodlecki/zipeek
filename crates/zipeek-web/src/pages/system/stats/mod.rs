mod bulk_selection;
mod context;
mod controls;
mod data;
mod dialog_focus;
mod explorer;
mod fixture;
mod grammar;
mod history;
mod insights;
mod overview;
mod pagination;
mod query;
mod query_hook;
mod sections;
mod selection_review;
mod settings;
mod simulator;
mod skills;
mod state;
mod tiles;

use gloo::timers::callback::Timeout;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use context::{
    LoadedSectionContext, StatsDataContext, StatsSourceContext, StatsUiContext, use_stats_ui,
};
use data::{DataAction, DemoStatisticsSource, SectionLoad, StatisticsDataState, StatisticsSource};
use explorer::VocabularyExplorer;
use grammar::GrammarExplorer;
use history::HistoryExplorer;
use insights::InsightExplorer;
use overview::Overview;
use sections::DashboardTab;
use selection_review::SelectionReview;
use settings::DashboardSettings;
use skills::SkillsExplorer;
use state::{DashboardAction, DashboardState, PreviewPhase};

#[function_component(StatsContent)]
fn stats_content() -> Html {
    let context = use_stats_ui();
    let state = context.0.clone();
    let timeout = use_mut_ref(|| None::<Timeout>);
    let data = use_context::<StatsDataContext>().expect("Statistics data provider is required");
    {
        let timeout = timeout.clone();
        use_effect_with((), move |_| {
            move || {
                timeout.borrow_mut().take();
            }
        });
    }
    let on_preview = {
        let state = state.clone();
        let timeout = timeout.clone();
        let data = data.clone();
        Callback::from(move |_: MouseEvent| match state.phase {
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
        })
    };
    let tabs = DashboardTab::ALL.into_iter().map(|target| {
        let selected = state.tab == target;
        let state = state.clone();
        let onclick = Callback::from(move |_: MouseEvent| state.dispatch(DashboardAction::SelectTab(target)));
        html! {
            <button key={target.key()} type="button" role="tab" data-stats-tab={target.key()}
                aria-selected={selected.to_string()} onclick={onclick}
                class={classes!("inline-flex", "shrink-0", "min-h-10", "items-center", "gap-2", "rounded-lg", "px-3", "text-sm", "font-semibold", "focus-visible:outline-2", "focus-visible:outline-accent", if selected {"bg-brand-action text-white"} else {"border border-line bg-surface text-ink hover:bg-raised"})}>
                <Icon data={target.icon()} width="17px" height="17px" />
                {target.label()}
            </button>
        }
    });
    html! {
        <main class="mx-auto flex w-full max-w-[1520px] flex-col gap-3 px-4 py-4 sm:px-6 sm:py-6 xl:px-8" data-stats-page="">
            <header class="flex flex-wrap items-center justify-between gap-3">
                <div class="flex items-center gap-2">
                    <h1 class="sr-only">{"Statistics"}</h1>
                </div>
                <button type="button" data-stats-preview-toggle="" aria-pressed={(state.phase == PreviewPhase::Open).to_string()}
                    onclick={on_preview}
                    aria-label={match state.phase { PreviewPhase::Open => "Close preview", PreviewPhase::Loading { .. } => "Cancel preview", PreviewPhase::Closed => "Preview dashboard" }}
                    title={match state.phase { PreviewPhase::Open => "Close preview", PreviewPhase::Loading { .. } => "Cancel preview", PreviewPhase::Closed => "Preview dashboard" }}
                    class="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line bg-surface px-3 text-sm font-semibold hover:bg-raised disabled:opacity-60 focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={if state.phase == PreviewPhase::Open { IconData::LUCIDE_X } else { IconData::LUCIDE_EYE }} width="17px" height="17px" />
                </button>
            </header>
            {match state.phase {
                PreviewPhase::Closed => html! {
                    <section data-stats-unavailable="" class="rounded-xl border border-line bg-surface p-5" aria-labelledby="stats-data-status-heading">
                        <h2 id="stats-data-status-heading" class="font-semibold">{"Learning evidence not connected"}</h2>
                        <p class="mt-2 text-sm text-muted">{"Verified learning observations are needed before proficiency can be assessed."}</p>
                        <a href="#/settings?section=learning" class="mt-3 inline-flex min-h-10 items-center rounded-lg border border-line px-3 text-sm font-semibold hover:bg-raised">{"Learning settings"}</a>
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
                        <div role="tablist" aria-label="Statistics sections" class="flex flex-nowrap gap-2 overflow-x-auto pb-1">{for tabs}</div>
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
    let data = use_context::<StatsDataContext>()
        .expect("Statistics data provider is required")
        .0;
    let source = use_context::<StatsSourceContext>()
        .expect("Statistics source provider is required")
        .0;
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
            if !matches!(
                tab,
                DashboardTab::Hanzi | DashboardTab::Grammar | DashboardTab::History
            ) && !matches!(data.get(tab), Some(SectionLoad::Ready(_)))
            {
                data.dispatch(DataAction::Start {
                    tab,
                    generation: active_generation,
                });
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
                            tab,
                            generation: active_generation,
                            payload,
                        }),
                        Err(error) => data_for_task.dispatch(DataAction::Failed {
                            tab,
                            generation: active_generation,
                            error,
                        }),
                    }
                });
            }
            move || {
                alive.set(false);
                data.dispatch(DataAction::Invalidate {
                    tab,
                    generation: active_generation,
                });
            }
        });
    }
    let on_retry = {
        let retry_epoch = retry_epoch.clone();
        Callback::from(move |_: MouseEvent| retry_epoch.set((*retry_epoch).wrapping_add(1)))
    };
    match props.tab {
        DashboardTab::Hanzi => return html! { <VocabularyExplorer /> },
        DashboardTab::Grammar => return html! { <GrammarExplorer /> },
        DashboardTab::History => return html! { <HistoryExplorer /> },
        _ => {}
    }
    match data.get(props.tab) {
        Some(SectionLoad::Ready(payload)) => html! {
            <ContextProvider<LoadedSectionContext> context={LoadedSectionContext(payload.clone())}>
                {match props.tab {
                                DashboardTab::Overview => html! { <Overview metrics={ui.metrics} tiles={ui.tiles.clone()} on_vocabulary={on_vocabulary} /> },
                                DashboardTab::Hanzi => html! { <VocabularyExplorer /> },
                                DashboardTab::Grammar => html! { <GrammarExplorer /> },
                                DashboardTab::Skills => html! { <SkillsExplorer /> },
                                DashboardTab::History => html! { <HistoryExplorer /> },
                                DashboardTab::Insights => html! { <InsightExplorer /> },
                                DashboardTab::Settings => html! { <DashboardSettings /> },
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
