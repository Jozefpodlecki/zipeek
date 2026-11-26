use super::super::tiles::TileId;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum TileAction {
    Toggle(TileId),
    Move(TileId, i8),
}

#[derive(Properties, PartialEq)]
pub(super) struct MetricTileProps {
    pub tile: TileId,
    pub enabled: bool,
    pub can_up: bool,
    pub can_down: bool,
    pub on_action: Callback<TileAction>,
}

#[function_component(MetricTile)]
pub(super) fn metric_tile(props: &MetricTileProps) -> Html {
    let tile = props.tile;
    let on_toggle = props.on_action.clone();
    let toggle = Callback::from(move |_: MouseEvent| on_toggle.emit(TileAction::Toggle(tile)));
    let on_up = props.on_action.clone();
    let move_up = Callback::from(move |_: MouseEvent| on_up.emit(TileAction::Move(tile, -1)));
    let on_down = props.on_action.clone();
    let move_down = Callback::from(move |_: MouseEvent| on_down.emit(TileAction::Move(tile, 1)));
    html! {
        <article class="flex min-w-0 items-center gap-2 rounded-lg border border-line bg-raised px-2 py-1.5">
            <button type="button" role="switch" aria-checked={props.enabled.to_string()} aria-label={format!("Show {}", tile.label())}
                onclick={toggle} class="flex min-h-10 min-w-0 flex-1 items-center gap-2 rounded-lg px-1 text-left hover:bg-surface focus-visible:outline-2 focus-visible:outline-accent">
                <span aria-hidden="true" class={classes!("flex", "h-5", "w-9", "rounded-full", "p-0.5", if props.enabled {"bg-brand-action"} else {"bg-line"})}>
                    <span class={classes!("h-4", "w-4", "rounded-full", "bg-white", "transition-transform", if props.enabled {"translate-x-4"} else {"translate-x-0"})} />
                </span>
                <span class="min-w-0 flex-1">
                    <span class="block truncate text-sm font-semibold">{tile.label()}</span>
                    <span class="block truncate text-xs text-muted">{tile.group()}</span>
                </span>
            </button>
            <div class="flex shrink-0 items-center gap-1">
                <button type="button" disabled={!props.can_up} onclick={move_up} aria-label={format!("Move {} up", tile.label())}
                    class="inline-flex min-h-10 min-w-9 items-center justify-center rounded-md border border-line hover:bg-surface disabled:opacity-30 focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_ARROW_UP} width="15px" />
                </button>
                <button type="button" disabled={!props.can_down} onclick={move_down} aria-label={format!("Move {} down", tile.label())}
                    class="inline-flex min-h-10 min-w-9 items-center justify-center rounded-md border border-line hover:bg-surface disabled:opacity-30 focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_ARROW_DOWN} width="15px" />
                </button>
            </div>
        </article>
    }
}
