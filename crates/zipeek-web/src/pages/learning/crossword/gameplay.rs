use crate::components::games::crossword::{
    board::CrosswordBoard, clues::CrosswordClues, tiles::CrosswordTiles,
};
use yew::prelude::*;
use zipeek_exercises::crossword::Crossword;

#[derive(Properties, PartialEq)]
pub(super) struct CrosswordGameplayProps {
    pub game: Crossword,
    pub active_clue: Option<u8>,
    pub selected_tile: Option<usize>,
    pub last_crossing: Option<(usize, usize)>,
    pub rejected_tile: Option<usize>,
    pub rejection_sequence: u32,
    pub loading: bool,
    pub on_place: Callback<(usize, usize, usize)>,
    pub on_activate: Callback<u8>,
    pub on_select: Callback<usize>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(CrosswordGameplay)]
pub(super) fn crossword_gameplay(props: &CrosswordGameplayProps) -> Html {
    let crossword_loading_overlay = if props.loading {
        html! { <div
            role="status"
            aria-live="polite"
            class="absolute inset-0 z-10 flex items-center justify-center rounded-2xl bg-surface/80 text-sm font-semibold"
        >
            {"Generating crossword…"}
        </div> }
    } else {
        Html::default()
    };
    html! {
        <>
            <div
                class="grid min-h-0 min-w-0 flex-1 grid-cols-[minmax(0,1fr)] gap-3 pt-3 lg:grid-cols-[minmax(0,1fr)_minmax(370px,0.85fr)]"
            >
                <section
                    class="relative flex min-h-0 min-w-0 flex-col items-center justify-center rounded-2xl border border-line bg-raised p-3"
                >
                    {crossword_loading_overlay}
                    <CrosswordBoard
                        game={props.game.clone()}
                        highlighted_clue={props.active_clue}
                        selected_tile={props.selected_tile}
                        last_crossing={props.last_crossing}
                        on_place={props.on_place.clone()}
                    />
                </section>
                <CrosswordClues
                    game={props.game.clone()}
                    active_clue={props.active_clue}
                    on_activate={props.on_activate.clone()}
                />
            </div>
            <section
                class="crossword-bank mt-3 shrink-0 rounded-2xl border border-line bg-raised p-2 sm:p-3"
            >
                <CrosswordTiles
                    game={props.game.clone()}
                    selected_tile={props.selected_tile}
                    active_clue={props.active_clue}
                    rejected_tile={props.rejected_tile}
                    rejection_sequence={props.rejection_sequence}
                    on_select={props.on_select.clone()}
                />
            </section>
            {for props.children.iter()}
        </>
    }
}
