use super::super::query::ActivityBucket;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct ActivityDayProps {
    pub day: ActivityBucket,
    pub selected: bool,
    pub on_select: Callback<String>,
}

#[function_component(ActivityDay)]
pub(super) fn activity_day(props: &ActivityDayProps) -> Html {
    let day = &props.day;
    let practice_width = day.practice * 100 / day.count.max(1);
    let games_width = day.games * 100 / day.count.max(1);
    let onclick = {
        let date = day.date.clone();
        let on_select = props.on_select.clone();
        Callback::from(move |_: MouseEvent| on_select.emit(date.clone()))
    };
    html! {
        <button type="button" onclick={onclick} aria-pressed={props.selected.to_string()}
            aria-label={format!("{}: {} Practice and {} Games sessions. Filter by date", day.date, day.practice, day.games)}
            class={classes!("flex", "h-10", "min-w-0", "flex-col", "items-center", "justify-center", "gap-0.5", "rounded-md", "border", "px-1", "focus-visible:outline-2", "focus-visible:outline-accent", if props.selected { "border-accent bg-raised" } else { "border-line bg-surface hover:bg-raised" })}>
            <span class="text-[11px] tabular-nums text-muted">{day.date.get(8..).unwrap_or(day.date.as_str()).to_owned()}</span>
            <div class="flex h-1.5 w-full overflow-hidden rounded-full bg-raised" aria-hidden="true">
                <div class="flex h-full w-full overflow-hidden rounded-full">
                    <span class="h-full bg-accent" style={format!("width: {practice_width}%")} />
                    <span class="h-full bg-amber-400/80" style={format!("width: {games_width}%")} />
                </div>
            </div>
        </button>
    }
}
