use super::super::state::SkillGap;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct InsightRowProps {
    pub rank: usize,
    pub title: String,
    pub count: usize,
    pub explanation: String,
    pub skill: Option<SkillGap>,
    pub on_explore: Callback<SkillGap>,
}

#[function_component(InsightRow)]
pub(super) fn insight_row(props: &InsightRowProps) -> Html {
    let skill = props.skill;
    let on_explore = props.on_explore.clone();
    let action = Callback::from(move |_: MouseEvent| {
        if let Some(skill) = skill {
            on_explore.emit(skill);
        }
    });
    html! {
        <article class="flex flex-wrap items-center gap-3 px-4 py-3 hover:bg-raised/50">
            <span class="inline-flex size-8 shrink-0 items-center justify-center rounded-full bg-raised text-xs font-semibold text-muted">{props.rank}</span>
            <div class="min-w-0 flex-1">
                <h3 class="font-semibold">{props.title.clone()}</h3>
                <p class="text-xs text-muted">{props.explanation.clone()}</p>
            </div>
            <div class="text-right"><span class="text-lg font-bold tabular-nums">{props.count}</span><span class="ml-1 text-xs text-muted">{"words"}</span></div>
            <button type="button" onclick={action} aria-label={format!("Explore {}", props.title)} title="Explore words"
                class="inline-flex size-10 items-center justify-center rounded-lg border border-line hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="17px" />
            </button>
        </article>
    }
}
