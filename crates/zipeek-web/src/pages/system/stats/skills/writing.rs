use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct WritingActivityProps {
    pub recorded: usize,
    pub review: usize,
    pub total: usize,
    pub on_explore: Callback<MouseEvent>,
}

#[function_component(WritingActivity)]
pub(super) fn writing_activity(props: &WritingActivityProps) -> Html {
    html! {
        <article class="flex flex-wrap items-center justify-between gap-4 rounded-xl border border-line bg-surface p-4">
            <div class="flex items-start gap-3">
                <Icon data={IconData::LUCIDE_PENCIL} width="19px" />
                <div class="space-y-1">
                    <h3 class="font-semibold">{"Writing activity"}<span class="ml-2 text-xs font-normal text-muted">{"Additional category"}</span></h3>
                    <p class="text-sm text-muted">{format!("{} of {} words have a writing activity or review marker ({} review markers).", props.recorded + props.review, props.total, props.review)}</p>
                </div>
            </div>
            <button type="button" onclick={props.on_explore.clone()} class="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line px-3 text-sm hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                {"Explore"}<Icon data={IconData::LUCIDE_ARROW_RIGHT} width="15px" />
            </button>
        </article>
    }
}
