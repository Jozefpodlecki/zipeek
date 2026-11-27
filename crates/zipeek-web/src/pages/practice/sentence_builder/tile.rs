use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::sentence_builder::{
    BuilderAction, DragSource, DropTarget, MoveDirection, TileId,
};

fn drag_start(source: DragSource, on_action: Callback<BuilderAction>) -> Callback<DragEvent> {
    Callback::from(move |event: DragEvent| {
        if let Some(data) = event.data_transfer() {
            let tile = match source {
                DragSource::Bank(tile) | DragSource::Selection(tile) => tile,
            };
            let _ = data.set_data("text/plain", &tile.index().to_string());
        }
        on_action.emit(BuilderAction::BeginDrag(source));
    })
}

fn drag_end(source: DragSource, on_action: Callback<BuilderAction>) -> Callback<DragEvent> {
    Callback::from(move |_: DragEvent| on_action.emit(BuilderAction::CancelDrag(source)))
}

#[derive(Properties, PartialEq)]
pub(super) struct BankProps {
    pub tile: TileId,
    pub label: &'static str,
    pub disabled: bool,
    pub on_action: Callback<BuilderAction>,
}

#[function_component(BankTile)]
pub(super) fn bank_tile(props: &BankProps) -> Html {
    let tile = props.tile;
    let on_click = {
        let on_action = props.on_action.clone();
        Callback::from(move |_: MouseEvent| on_action.emit(BuilderAction::Choose(tile)))
    };
    let on_dragstart = drag_start(DragSource::Bank(tile), props.on_action.clone());
    let on_dragend = drag_end(DragSource::Bank(tile), props.on_action.clone());
    html! {
        <button type="button" onclick={on_click} disabled={props.disabled} draggable={(!props.disabled).to_string()}
            ondragstart={on_dragstart} ondragend={on_dragend}
            class="min-h-11 cursor-grab rounded-xl border border-line bg-raised px-4 py-2 text-lg text-ink transition-colors hover:border-accent hover:bg-accent-soft disabled:opacity-25 motion-reduce:transition-none"
            aria-label={format!("Add {}", props.label)}>
            {props.label}
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct SelectedProps {
    pub tile: TileId,
    pub label: &'static str,
    pub locked: bool,
    pub can_move_left: bool,
    pub can_move_right: bool,
    pub on_action: Callback<BuilderAction>,
}

#[function_component(SelectedTile)]
pub(super) fn selected_tile(props: &SelectedProps) -> Html {
    let tile = props.tile;
    let on_click = {
        let on_action = props.on_action.clone();
        Callback::from(move |_: MouseEvent| on_action.emit(BuilderAction::Remove(tile)))
    };
    let on_left = {
        let on_action = props.on_action.clone();
        Callback::from(move |_: MouseEvent| {
            on_action.emit(BuilderAction::Move(tile, MoveDirection::Left))
        })
    };
    let on_right = {
        let on_action = props.on_action.clone();
        Callback::from(move |_: MouseEvent| {
            on_action.emit(BuilderAction::Move(tile, MoveDirection::Right))
        })
    };
    let on_dragstart = drag_start(DragSource::Selection(tile), props.on_action.clone());
    let on_dragend = drag_end(DragSource::Selection(tile), props.on_action.clone());
    let on_dragover = Callback::from(|event: DragEvent| event.prevent_default());
    let on_drop = {
        let on_action = props.on_action.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            event.stop_propagation();
            on_action.emit(BuilderAction::Drop(DropTarget::Tile(tile)));
        })
    };
    html! {
        <div draggable={(!props.locked).to_string()} ondragstart={on_dragstart} ondragend={on_dragend} ondragover={on_dragover} ondrop={on_drop}
            class="group flex items-center gap-0.5 rounded-lg border border-accent/30 bg-accent-soft px-1 py-1 text-ink transition-colors hover:border-accent motion-reduce:transition-none">
            <button type="button" onclick={on_left} disabled={props.locked || !props.can_move_left} class="rounded p-1 text-muted hover:text-ink disabled:opacity-25" aria-label={format!("Move {} left", props.label)} title="Move left">
                <Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="15" height="15" />
            </button>
            <button type="button" onclick={on_click} disabled={props.locked} class="px-1 py-1 text-lg font-medium disabled:cursor-default" aria-label={format!("Remove {}", props.label)} title="Remove tile">
                {props.label}
            </button>
            <button type="button" onclick={on_right} disabled={props.locked || !props.can_move_right} class="rounded p-1 text-muted hover:text-ink disabled:opacity-25" aria-label={format!("Move {} right", props.label)} title="Move right">
                <Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="15" height="15" />
            </button>
        </div>
    }
}
