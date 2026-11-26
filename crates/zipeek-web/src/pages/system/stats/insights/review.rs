use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct ReviewRecommendationProps {
    pub count: usize,
    pub on_open: Callback<MouseEvent>,
}

#[function_component(ReviewRecommendation)]
pub(super) fn review_recommendation(props: &ReviewRecommendationProps) -> Html {
    html! {
        <article class="flex flex-wrap items-center gap-3 px-4 py-3 hover:bg-raised/50">
            <span class="inline-flex size-8 shrink-0 items-center justify-center rounded-full bg-raised text-xs text-muted">
                <Icon data={IconData::LUCIDE_BOOKMARK} width="16px" />
            </span>
            <div class="min-w-0 flex-1"><h3 class="font-semibold">{"To review"}</h3><p class="text-xs text-muted">{"Words marked for review"}</p></div>
            <strong class="text-lg tabular-nums">{props.count}</strong>
            <button type="button" aria-label="Explore words for review" title="Review words" onclick={props.on_open.clone()}
                class="inline-flex size-10 items-center justify-center rounded-lg border border-line hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="17px" />
            </button>
        </article>
    }
}
