use crate::services::practice_session_store::{
    AssessmentCapability, AssessmentEvidence, ExerciseVerdict,
};
use yew::prelude::*;
use zipeek_core::Word;

use super::super::{
    feedback::{ExerciseReview, FeedbackVerdict},
    instruction::InstructionIcon,
    surface::ExerciseSurface,
};

use super::{
    mosaic::{mosaic_complete, mosaic_tiles},
    progress::VocabularyRoundResult,
};

mod state;
use state::{MosaicAction, MosaicState};

#[derive(Properties, PartialEq)]
pub(super) struct WordMosaicsProps {
    pub words: Vec<Word>,
    pub seed: u64,
    pub on_completed: Callback<VocabularyRoundResult>,
}

#[function_component(WordMosaics)]
pub(super) fn word_mosaics(props: &WordMosaicsProps) -> Html {
    let game = use_reducer_eq(MosaicState::default);
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
        let words = props.words.clone();
        Callback::from(move |_| {
            if game.matched.len() != words.len() || words.is_empty() {
                return;
            }
            let item_evidence = words
                .iter()
                .enumerate()
                .map(|(pair, word)| {
                    let mistakes = game
                        .pair_mistakes
                        .get(&pair)
                        .copied()
                        .unwrap_or_default()
                        .min(u16::MAX as u32) as u16;
                    AssessmentEvidence {
                        capability: AssessmentCapability::VocabularyMeaningMatching,
                        target: word.simplified.clone(),
                        verdict: if mistakes == 0 {
                            ExerciseVerdict::Correct
                        } else {
                            ExerciseVerdict::PartiallyCorrect
                        },
                        attempts: mistakes.saturating_add(1),
                        mistakes,
                        hints: 0,
                        assisted: false,
                    }
                })
                .collect();
            on_completed.emit(VocabularyRoundResult {
                correct: true,
                attempts: (game.matched.len() as u32).saturating_add(game.mistakes),
                mistakes: game.mistakes,
                item_evidence,
            });
        })
    };

    html! {
        <div class="w-full" data-exercise="word-mosaics">
            <ExerciseSurface icon={InstructionIcon::Match} instruction="Match each Chinese word with its meaning.">
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
            <div data-mosaic-feedback="">
                {if game.feedback.is_empty() {
                    Html::default()
                } else if complete {
                    html! { <ExerciseReview verdict={FeedbackVerdict::Correct} on_continue={on_continue.clone()} word_mosaics=true><p>{game.feedback.clone()}</p></ExerciseReview> }
                } else {
                    html! { <p role="status" aria-live="polite">{game.feedback.clone()}</p> }
                }}
            </div>
            </ExerciseSurface>
        </div>
    }
}
