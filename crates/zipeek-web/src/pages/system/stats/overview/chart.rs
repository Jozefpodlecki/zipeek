use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::super::{context::use_loaded_section, data::SectionPayload};
use super::{
    activity::{activity_buckets, activity_categories},
    breakdown::ActivityBreakdown,
    fallback::ActivityFallback,
    lifecycle::{ChartStatus, use_activity_chart},
};

#[function_component(ActivityChart)]
pub(super) fn activity_chart() -> Html {
    let loaded = use_loaded_section();
    let SectionPayload::Overview { sessions, .. } = loaded.as_ref() else {
        unreachable!("Overview projection expected")
    };
    let buckets = activity_buckets(sessions);
    let categories = activity_categories(sessions);
    let canvas = use_node_ref();
    let status = use_activity_chart(canvas.clone(), buckets);
    let show_breakdown = use_state(|| false);
    let open_breakdown = {
        let show_breakdown = show_breakdown.clone();
        Callback::from(move |_: MouseEvent| show_breakdown.set(true))
    };
    let close_breakdown = {
        let show_breakdown = show_breakdown.clone();
        Callback::from(move |_: ()| show_breakdown.set(false))
    };

    html! {
        <article class="min-w-0 rounded-xl border border-line bg-surface p-4" data-stats-activity-chart="">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h2 class="font-semibold">{"Activity"}</h2>
                <span class="text-xs text-muted">{"Completed sessions · weekly"}</span>
            </div>
            <div class="relative mt-3 h-40" aria-hidden="true">
                <canvas ref={canvas} class={if *status == ChartStatus::Ready {"h-full w-full"} else {"invisible h-full w-full"}} />
                if *status != ChartStatus::Ready {
                    <ActivityFallback buckets={buckets} />
                }
            </div>
            if *status == ChartStatus::Unavailable {
                <p class="text-xs text-muted">{"Chart unavailable. Weekly totals remain available in the breakdown."}</p>
            }
            <button type="button" onclick={open_breakdown} class="mt-2 inline-flex min-h-9 items-center gap-2 rounded-lg border border-line px-3 text-xs text-muted hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent" aria-label="View weekly activity breakdown">
                <Icon data={IconData::LUCIDE_LIST} width="15px" />{"Breakdown"}
            </button>
            if *show_breakdown { <ActivityBreakdown categories={categories} on_close={close_breakdown} /> }
        </article>
    }
}
