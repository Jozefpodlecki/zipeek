use super::super::{
    state::{DashboardPreset, MetricSelection},
    tiles::TileSelection,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct LayoutPresetsProps {
    pub metrics: MetricSelection,
    pub tiles: TileSelection,
    pub on_select: Callback<DashboardPreset>,
}

#[function_component(LayoutPresets)]
pub(super) fn layout_presets(props: &LayoutPresetsProps) -> Html {
    html! {
        <section class="rounded-xl border border-line bg-surface p-3" aria-label="Layout presets">
            <h3 class="font-semibold">{"Layouts"}</h3>
            <div class="mt-3 flex flex-wrap gap-2" role="group" aria-label="Dashboard card presets">
                {for [(DashboardPreset::Focused, "Focused"), (DashboardPreset::Activity, "Activity"), (DashboardPreset::Detailed, "Detailed")]
                    .into_iter().map(|(preset, label)| {
                        let on_select = props.on_select.clone();
                        let onclick = Callback::from(move |_: MouseEvent| on_select.emit(preset));
                        let active = props.metrics == preset.metrics() && props.tiles == TileSelection::preset(preset);
                        html! {
                            <button key={label} type="button" aria-pressed={active.to_string()} onclick={onclick}
                                class={classes!("min-h-10", "rounded-lg", "border", "px-4", "text-sm", "font-semibold", "focus-visible:outline-2", "focus-visible:outline-accent", if active {"border-accent bg-brand-action text-white"} else {"border-line bg-raised hover:bg-surface"})}>
                                {label}
                            </button>
                        }
                    })}
            </div>
        </section>
    }
}
