use super::super::state::{MetricId, MetricSelection};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct CoreMetricsProps {
    pub metrics: MetricSelection,
    pub on_toggle: Callback<MetricId>,
}

#[function_component(CoreMetrics)]
pub(super) fn core_metrics(props: &CoreMetricsProps) -> Html {
    html! {
        <section class="rounded-xl border border-line bg-surface p-3" aria-label="Core overview metrics">
            <h3 class="font-semibold">{"Core metrics"}</h3>
            <div class="mt-3 grid gap-2 md:grid-cols-2">
                {for [(MetricId::Words, "Words encountered"), (MetricId::Encounters, "Encounters"),
                    (MetricId::Sessions, "Vocabulary session appearances"), (MetricId::Coverage, "Skill categories tracked")]
                    .into_iter().map(|(metric, label)| {
                        let on_toggle = props.on_toggle.clone();
                        let enabled = props.metrics.is_enabled(metric);
                        let onclick = Callback::from(move |_: MouseEvent| on_toggle.emit(metric));
                        html! {
                            <button key={label} type="button" role="switch" aria-checked={enabled.to_string()} onclick={onclick}
                                class="flex min-h-11 items-center justify-between gap-2 rounded-lg border border-line bg-raised px-3 text-left text-sm hover:bg-surface focus-visible:outline-2 focus-visible:outline-accent">
                                <span>{label}</span>
                                <span aria-hidden="true" class={classes!("inline-flex", "h-5", "w-9", "rounded-full", "p-0.5", "transition-colors", if enabled {"bg-brand-action"} else {"bg-line"})}>
                                    <span class={classes!("h-4", "w-4", "rounded-full", "bg-white", "transition-transform", if enabled {"translate-x-4"} else {"translate-x-0"})} />
                                </span>
                            </button>
                        }
                    })}
            </div>
        </section>
    }
}
