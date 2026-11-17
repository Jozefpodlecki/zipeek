use yew::prelude::*;
use zipeek_exercises::character_hunt::Difficulty;

use super::state::{HuntAction, HuntSessionContext};

#[derive(Properties, PartialEq)]
pub(super) struct HuntTileProps {
    pub index: usize,
}

#[function_component(HuntTile)]
pub(super) fn hunt_tile(props: &HuntTileProps) -> Html {
    let context =
        use_context::<HuntSessionContext>().expect("HuntTile must be rendered within HuntSession");
    let session = &context.0;
    let Some(board) = session.board.as_ref() else {
        return Html::default();
    };
    let Some(tile) = board.tiles.get(props.index) else {
        return Html::default();
    };
    let selected = board.selected.contains(&props.index);
    let hinted = session.highlighted.contains(&props.index);
    let font_class = match session.difficulty {
        Difficulty::Easy => "text-5xl sm:text-7xl",
        Difficulty::Medium => "text-4xl sm:text-6xl",
        Difficulty::Difficult => "text-3xl sm:text-5xl",
        Difficulty::Hardcore => "text-xl sm:text-3xl",
    };
    let on_tap = {
        let session = session.clone();
        let index = props.index;
        Callback::from(move |_| session.dispatch(HuntAction::Tap(index)))
    };
    let state_label = if tile.completed {
        "found"
    } else if tile.filler {
        "not selectable"
    } else if selected {
        "selected"
    } else {
        "available"
    };
    let label = match &tile.pinyin {
        Some(reading) => format!("Hanzi {}, {reading}, {state_label}", tile.character),
        None => format!("Hanzi {}, {state_label}", tile.character),
    };

    let tile_pronunciation = {
        if let Some(reading) = &tile.pinyin {
            html! {
                <span
                    class="max-w-full truncate px-1 text-xs leading-tight text-current sm:text-sm"
                    data-hunt-pinyin=""
                >
                    {reading}
                </span>
            }
        } else {
            Html::default()
        }
    };

    let element_classes_1 = classes!(
        "aspect-square",
        "flex",
        "min-w-0",
        "flex-col",
        "items-center",
        "justify-center",
        "gap-1",
        "rounded-xl",
        "border",
        "font-medium",
        "transition-colors",
        "focus-visible:outline-2",
        "focus-visible:outline-accent",
        if tile.completed {
            "border-line bg-raised text-muted opacity-40"
        } else if tile.filler {
            "border-line bg-raised text-muted opacity-25"
        } else if selected {
            "border-accent bg-brand-action text-white ring-2 ring-accent"
        } else if hinted {
            "border-accent bg-accent-soft text-ink ring-2 ring-accent"
        } else {
            "border-line bg-raised hover:border-accent"
        },
    );
    html! {
        <button
            type="button"
            data-hunt-tile={props.index.to_string()}
            data-completed={tile.completed.to_string()}
            data-hunt-hinted={hinted.to_string()}
            aria-pressed={selected.to_string()}
            aria-label={label}
            disabled={tile.completed || tile.filler || board.complete()}
            onclick={on_tap}
            class={element_classes_1}
        >
            <span
                class={font_class}
                aria-hidden="true"
            >
                {tile.character}
            </span>
            {tile_pronunciation}
        </button>
    }
}
