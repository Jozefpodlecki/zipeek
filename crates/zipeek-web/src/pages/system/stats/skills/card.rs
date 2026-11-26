use super::super::state::SkillGap;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct SkillCardProps {
    pub skill: SkillGap,
    pub recorded: usize,
    pub review: usize,
    pub total: usize,
    pub on_explore: Callback<SkillGap>,
}

#[function_component(SkillCard)]
pub(super) fn skill_card(props: &SkillCardProps) -> Html {
    let skill = props.skill;
    let missing = props.total.saturating_sub(props.recorded + props.review);
    let on_explore = props.on_explore.clone();
    let view = Callback::from(move |_: MouseEvent| on_explore.emit(skill));
    let icon = match skill {
        SkillGap::Recognition => IconData::LUCIDE_BOOK_OPEN,
        SkillGap::Listening => IconData::LUCIDE_CLOCK,
        SkillGap::Speaking => IconData::LUCIDE_MESSAGE_CIRCLE,
        SkillGap::Sentence => IconData::LUCIDE_LAYERS,
        SkillGap::Writing => IconData::LUCIDE_PENCIL,
    };
    html! {
        <article class="flex min-h-44 flex-col rounded-xl border border-line bg-surface p-4">
            <div class="flex items-center gap-2"><Icon data={icon} width="18px" /><h3 class="font-semibold">{skill.label()}</h3></div>
            <p class="mt-4 text-2xl font-semibold tabular-nums">{props.recorded + props.review}<span class="text-base font-normal text-muted">{format!(" / {}", props.total)}</span></p>
            <div class="mt-3 flex h-2 overflow-hidden rounded-full bg-raised" role="img" aria-label={format!("{} activity records, {} review markers, {} without records", props.recorded, props.review, missing)}>
                <div class="bg-accent" style={format!("width: {}%", props.recorded * 100 / props.total.max(1))} />
                <div class="bg-amber-500" style={format!("width: {}%", props.review * 100 / props.total.max(1))} />
            </div>
            <p class="mt-2 text-xs text-muted">{format!("{} recorded · {} marked for review · {} without evidence", props.recorded, props.review, missing)}</p>
            <button type="button" onclick={view} class="mt-auto inline-flex min-h-10 items-center gap-2 self-start rounded-lg border border-line px-3 text-sm hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                {"Explore"}<Icon data={IconData::LUCIDE_ARROW_RIGHT} width="15px" />
            </button>
        </article>
    }
}
