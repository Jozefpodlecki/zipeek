mod tile;
use self::tile::CrosswordTile;
use std::collections::BTreeMap;
use yew::prelude::*;
use zipeek_exercises::crossword::Crossword;

#[derive(Properties, PartialEq)]
pub(crate) struct CrosswordTilesProps {
    pub game: Crossword,
    pub selected_tile: Option<usize>,
    pub active_clue: Option<u8>,
    pub rejected_tile: Option<usize>,
    pub rejection_sequence: u32,
    pub on_select: Callback<usize>,
}

#[function_component(CrosswordTiles)]
pub(crate) fn crossword_tiles(props: &CrosswordTilesProps) -> Html {
    let pronunciations: BTreeMap<char, &str> = props
        .game
        .clues()
        .iter()
        .flat_map(|clue| clue.answer.chars().zip(clue.pinyin.split_whitespace()))
        .collect();

    html! {
        <section
            class="min-w-0"
            aria-label="Character tiles"
        >
            <div
                class="flex min-w-0 gap-2 overflow-x-auto overscroll-x-contain pb-2"
                role="group"
                aria-label={if props.active_clue.is_some() { "Choose a character for the highlighted clue" } else { "Select or drag a character, then choose a board square" }}
            >
                {for props.game.bank().iter().enumerate().map(|(index, character)| {
                    let selected = props.selected_tile == Some(index);
                    let rejected = props.rejected_tile == Some(index);
                    let pronunciation = pronunciations.get(character).copied().unwrap_or("");
                    html! {
                        <CrosswordTile
                            key={format!("tile-{index}-{character}-{}", if rejected { props.rejection_sequence } else { 0 })}
                            {index}
                            character={*character}
                            pronunciation={pronunciation.to_owned()}
                            {selected}
                            {rejected}
                            on_select={props.on_select.clone()}
                        />
                    }
                })}
            </div>
        </section>
    }
}
