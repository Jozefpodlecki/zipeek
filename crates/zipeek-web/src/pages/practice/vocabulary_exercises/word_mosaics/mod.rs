use yew::prelude::*;
use zipeek_core::Word;

use super::mosaic::{mosaic_complete, mosaic_tiles};

mod state;
use state::{MosaicAction, MosaicState};

#[derive(Properties, PartialEq)]
pub(super) struct WordMosaicsProps {
    pub words: Vec<Word>,
    pub seed: u64,
    pub on_completed: Callback<u32>,
}

#[function_component(WordMosaics)]
pub(super) fn word_mosaics(props: &WordMosaicsProps) -> Html {
    let game = use_reducer(MosaicState::default);
    {
        let game = game.clone();
        use_effect_with((props.seed, props.words.clone()), move |_| {
            game.dispatch(MosaicAction::Reset);
        });
    }
    let tiles = mosaic_tiles(&props.words, props.seed);
    let complete = mosaic_complete(&game.matched, &tiles);
    let on_continue = {
        let on_completed = props.on_completed.clone();
        let game = game.clone();
        Callback::from(move |_| on_completed.emit(game.mistakes))
    };

    html! {
        <div class="space-y-5" data-exercise="word-mosaics">
            <p class="text-muted">{"Match each Chinese word with its meaning to clear the mosaic."}</p>
            <p role="status" aria-live="polite" data-mosaic-progress="">{format!("{} of {} pairs matched", game.matched.len(), tiles.len() / 2)}</p>
            <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
                {for tiles.iter().enumerate().map(|(tile, entry)| {
                    let pair = entry.pair;
                    let english = entry.is_english;
                    let label = entry.label.clone();
                    let is_selected = game.selected == Some(tile);
                    let is_matched = game.matched.contains(&pair);
                    let game = game.clone();
                    let tiles = tiles.clone();
                    let on_select = Callback::from(move |_| game.dispatch(MosaicAction::Select { index: tile, tiles: tiles.clone() }));
                    let element_classes_1 = classes!(
                                "min-h-28", "rounded-xl", "border", "p-3", "text-center",
                                "text-xl", "break-words", "bg-raised",
                                if is_matched { "opacity-20" } else { "opacity-100" },
                                if is_selected { "border-brand-action" } else { "border-line" },
                                if !is_matched && !is_selected { "hover:border-brand-action" } else { "" },
                            );
                    html! {
                        <button
                            type="button"
                            data-mosaic-tile=""
                            data-pair={pair.to_string()}
                            data-meaning={english.to_string()}
                            disabled={is_matched}
                            aria-pressed={is_selected.to_string()}
                            onclick={on_select}
                            class={element_classes_1}
                            lang={if english { "en" } else { "zh-Hans" }}
                        >
                            {label}
                        </button>
                    }
                })}
            </div>
            <p role="status" aria-live="polite" data-mosaic-feedback="">{game.feedback.clone()}</p>
            {if complete {
                html! { <button type="button" data-mosaic-continue="" onclick={on_continue} class="rounded-xl bg-brand-action px-6 py-4 font-bold text-white">{"Continue"}</button> }
            } else { Html::default() }}
        </div>
    }
}
