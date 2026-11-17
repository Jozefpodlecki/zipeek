use std::collections::{BTreeMap, BTreeSet};
use yew::prelude::*;
use zipeek_exercises::crossword::{Crossword, Direction};

#[derive(Properties, PartialEq)]
pub(crate) struct CrosswordBoardProps {
    pub game: Crossword,
    pub highlighted_clue: Option<u8>,
    pub on_place: Callback<(usize, usize, usize)>,
    pub selected_tile: Option<usize>,
    pub last_crossing: Option<(usize, usize)>,
}

#[function_component(CrosswordBoard)]
pub(crate) fn crossword_board(props: &CrosswordBoardProps) -> Html {
    let intersections = props.game.intersection_positions();
    let next_in_clue = props
        .highlighted_clue
        .and_then(|number| props.game.next_empty_in_clue(number));
    let highlighted: BTreeSet<_> = props
        .game
        .clues()
        .iter()
        .filter(|clue| Some(clue.number) == props.highlighted_clue)
        .flat_map(|clue| {
            clue.answer
                .chars()
                .enumerate()
                .map(move |(offset, _)| match clue.direction {
                    Direction::Across => (clue.row, clue.col + offset),
                    Direction::Down => (clue.row + offset, clue.col),
                })
        })
        .collect();
    let cells: BTreeMap<_, _> = props
        .game
        .cells()
        .map(|(position, _, value)| (position, value))
        .collect();

    let min_row = cells.keys().map(|(row, _)| *row).min().unwrap_or(0);
    let max_row = cells.keys().map(|(row, _)| *row).max().unwrap_or(0);
    let min_col = cells.keys().map(|(_, col)| *col).min().unwrap_or(0);
    let max_col = cells.keys().map(|(_, col)| *col).max().unwrap_or(0);
    let rows = max_row - min_row + 1;
    let cols = max_col - min_col + 1;

    let board_width_dvh = 72.0 * cols as f64 / rows as f64;
    let board_style = format!(
        "grid-template-columns: repeat({cols}, minmax(0, 1fr));          grid-template-rows: repeat({rows}, minmax(0, 1fr));          aspect-ratio: {cols} / {rows}; width: min(100%, {board_width_dvh:.2}dvh)"
    );

    let on_drag_over = Callback::from(|event: DragEvent| event.prevent_default());

    html! {
        <section
            class="flex w-full min-w-0 flex-1 items-center justify-center"
            aria-label="Crossword board"
            data-crossword-board=""
        >
            <div
                class="mx-auto grid max-h-full flex-none content-stretch justify-self-center gap-1.5"
                style={board_style}
                role="group"
                aria-label="Hanzi crossword squares"
            >
                {for (min_row..=max_row).flat_map(|row| (min_col..=max_col).map(move |col| (row, col))).map(|(row, col)| {
                    if let Some(filled) = cells.get(&(row, col)) {
                        html! {
                            <CrosswordCell
                                key={format!("{row}-{col}")}
                                row={row}
                                col={col}
                                filled={*filled}
                                selected_tile={props.selected_tile}
                                on_place={props.on_place.clone()}
                                on_drag_over={on_drag_over.clone()}
                                highlighted={filled.is_none() && highlighted.contains(&(row, col))}
                                next_in_clue={next_in_clue == Some((row, col))}
                                intersection={intersections.contains(&(row, col))}
                                last_crossing={props.last_crossing == Some((row, col))}
                            />
                        }
                    } else {
                        html! {
                            <span
                                key={format!("{row}-{col}")}
                                class="aspect-square"
                                aria-hidden="true"
                            />
                        }
                    }
                })}
            </div>
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct CrosswordCellProps {
    row: usize,
    col: usize,
    filled: Option<char>,
    selected_tile: Option<usize>,
    on_place: Callback<(usize, usize, usize)>,
    on_drag_over: Callback<DragEvent>,
    highlighted: bool,
    next_in_clue: bool,
    intersection: bool,
    last_crossing: bool,
}

#[function_component(CrosswordCell)]
fn crossword_cell(props: &CrosswordCellProps) -> Html {
    let row = props.row;
    let col = props.col;
    let selected_tile = props.selected_tile;
    let on_click = {
        let on_place = props.on_place.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(index) = selected_tile {
                on_place.emit((row, col, index));
            }
        })
    };
    let on_drop = {
        let on_place = props.on_place.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            if let Some(index) = event
                .data_transfer()
                .and_then(|data| data.get_data("text/plain").ok())
                .and_then(|raw| raw.parse::<usize>().ok())
            {
                on_place.emit((row, col, index));
            }
        })
    };
    let cell_label = format!(
        "Row {}, column {}, {}",
        row + 1,
        col + 1,
        props
            .filled
            .map(|ch| ch.to_string())
            .unwrap_or_else(|| "empty".to_owned()),
    );
    let cell_character = props.filled.map(|ch| ch.to_string()).unwrap_or_default();
    let cell_classes = classes!(
        "flex",
        "min-h-0",
        "min-w-0",
        "items-center",
        "justify-center",
        "overflow-hidden",
        "p-0",
        "leading-none",
        "bg-surface",
        "text-ink",
        "border-line",
        "relative",
        "aspect-square",
        "rounded-lg",
        "border",
        "font-bold",
        "focus-visible:outline-2",
        "focus-visible:outline-offset-2",
        if props.filled.is_some() {
            classes!("bg-[color-mix(in_srgb,var(--accent)_25%,var(--surface))]")
        } else {
            classes!(
                "bg-[color-mix(in_srgb,var(--accent)_13%,var(--surface))]",
                "hover:border-accent",
                "focus-visible:border-accent",
                "focus-visible:shadow-[inset_0_0_0_1px_var(--accent)]"
            )
        },
        props.filled.is_some().then(|| classes!(
            "bg-[color-mix(in_srgb,var(--surface)_76%,var(--raised))]",
            "border-[color-mix(in_srgb,var(--line)_65%,transparent)]",
            "text-[color-mix(in_srgb,var(--ink)_72%,var(--muted))]",
            "hover:border-[color-mix(in_srgb,var(--line)_65%,transparent)]"
        )),
        props.highlighted.then(|| classes!(
            "!border-accent",
            "!bg-[color-mix(in_srgb,var(--accent)_42%,var(--surface))]",
            "shadow-[0_0_0_2px_color-mix(in_srgb,var(--accent)_35%,transparent)]"
        )),
        props.next_in_clue.then(|| classes!(
            "outline-[3px]",
            "outline-offset-[-4px]",
            "outline-brand-action",
            "shadow-[inset_0_0_0_2px_var(--ink)]"
        )),
        props
            .intersection
            .then_some("border-[color-mix(in_srgb,var(--brand-action)_55%,var(--line))]"),
        props
            .last_crossing
            .then_some("motion-safe:animate-[crossword-crossing-pulse_480ms_ease-out]"),
    );

    html! {
        <button
            type="button"
            class={cell_classes}
            aria-label={cell_label}
            data-crossword-cell=""
            onclick={on_click}
            ondrop={on_drop}
            ondragover={props.on_drag_over.clone()}
        >
            <span
                class="block max-w-full whitespace-nowrap text-[clamp(1rem,2.5vw,2.5rem)] leading-none"
            >
                {cell_character}
            </span>
        </button>
    }
}
