use super::super::{
    context::use_stats_ui,
    state::{DashboardAction, DashboardPreset, MetricId},
};
use super::{
    catalogue::MetricCatalogue, core_metrics::CoreMetrics, presets::LayoutPresets, tile::TileAction,
};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(DashboardSettings)]
pub(crate) fn dashboard_settings() -> Html {
    let dashboard = use_stats_ui().0;
    let configured = dashboard.tiles.visible.len()
        + [
            MetricId::Words,
            MetricId::Encounters,
            MetricId::Sessions,
            MetricId::Coverage,
        ]
        .iter()
        .filter(|metric| dashboard.metrics.is_enabled(**metric))
        .count();
    let reset = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::ResetTiles))
    };
    let on_preset = {
        let dashboard = dashboard.clone();
        Callback::from(move |preset: DashboardPreset| {
            dashboard.dispatch(DashboardAction::SetMetricPreset(preset))
        })
    };
    let on_metric = {
        let dashboard = dashboard.clone();
        Callback::from(move |metric: MetricId| {
            dashboard.dispatch(DashboardAction::ToggleMetric(metric))
        })
    };
    let on_tile_action = {
        let dashboard = dashboard.clone();
        Callback::from(move |action: TileAction| match action {
            TileAction::Toggle(tile) => dashboard.dispatch(DashboardAction::ToggleTile(tile)),
            TileAction::Move(tile, direction) => {
                dashboard.dispatch(DashboardAction::MoveTile(tile, direction))
            }
        })
    };
    html! {
        <section class="space-y-2" data-stats-settings="">
            <header class="flex flex-wrap items-start justify-between gap-4 rounded-xl border border-line bg-surface p-3">
                <div class="space-y-1">
                    <h2 class="text-lg font-semibold">{"Customize Overview"}</h2>
                    <p class="text-xs tabular-nums text-muted" data-stats-tile-count="">{format!("{} of 24 available metrics enabled", configured)}</p>
                </div>
                <button type="button" onclick={reset} class="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line bg-raised px-3 text-sm font-semibold hover:bg-surface focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_ROTATE_CCW} width="16px" />{"Reset"}
                </button>
            </header>
            <LayoutPresets metrics={dashboard.metrics} tiles={dashboard.tiles.clone()} on_select={on_preset} />
            <CoreMetrics metrics={dashboard.metrics} on_toggle={on_metric} />
            <MetricCatalogue tiles={dashboard.tiles.clone()} on_action={on_tile_action} />
        </section>
    }
}
