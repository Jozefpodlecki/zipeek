use std::rc::Rc;
use yew::Reducible;
use zipeek_exercises::crossword::{Crossword, CrosswordWord, Placement};
use zipeek_exercises::practice_draft::HskBand;

use super::{rotation::CrosswordRotation, storage};
use crate::AppEnvironment;

#[derive(Clone, PartialEq)]
pub(super) struct CrosswordState {
    pub game: Option<Crossword>,
    pub bands: Vec<HskBand>,
    pub session_id: String,
    pub selected_tile: Option<usize>,
    pub active_clue: Option<u8>,
    pub rejected_tile: Option<usize>,
    pub rejection_sequence: u32,
    pub notice: Option<&'static str>,
    pub last_crossing: Option<(usize, usize)>,
}

pub(super) enum CrosswordAction {
    Start {
        vocabulary: Vec<CrosswordWord>,
        bands: Vec<HskBand>,
        history: CrosswordRotation,
    },
    SelectTile(usize),
    ActivateClue(u8),
    Place {
        row: usize,
        col: usize,
        tile: usize,
    },
    Reset {
        history: CrosswordRotation,
    },
}

impl CrosswordState {
    pub(super) fn from_environment(
        environment: &AppEnvironment,
        url_session_id: Option<&str>,
    ) -> Self {
        let session_id =
            storage::session_id_from_query(url_session_id).unwrap_or_else(storage::new_session_id);
        let restored = storage::load(environment, &session_id);
        let (game, bands) = match restored {
            Some((game, bands)) => (Some(game), bands),
            None => (None, Vec::new()),
        };
        Self {
            game,
            bands,
            session_id,
            selected_tile: None,
            active_clue: None,
            rejected_tile: None,
            rejection_sequence: 0,
            notice: None,
            last_crossing: None,
        }
    }

    fn place_tile(&mut self, row: usize, col: usize, tile: usize) {
        let Some(game) = self.game.as_mut() else {
            return;
        };
        match game.place(row, col, tile) {
            Placement::Correct | Placement::Finished => {
                self.last_crossing = game
                    .intersection_positions()
                    .contains(&(row, col))
                    .then_some((row, col));
                self.selected_tile = None;
                self.rejected_tile = None;
                self.notice = None;

                if self.active_clue.is_some_and(|number| {
                    game.clues()
                        .iter()
                        .find(|clue| clue.number == number)
                        .is_some_and(|clue| game.clue_completed(clue))
                }) {
                    self.active_clue = None;
                }
            }
            Placement::Wrong => {
                self.rejected_tile = Some(tile);
                self.rejection_sequence = self.rejection_sequence.wrapping_add(1);
                self.notice = Some("Incorrect tile");
            }
            Placement::Occupied | Placement::Invalid => {}
        }
    }

    fn clear_interaction(&mut self) {
        self.selected_tile = None;
        self.active_clue = None;
        self.rejected_tile = None;
        self.rejection_sequence = 0;
        self.notice = None;
        self.last_crossing = None;
    }
}

impl Reducible for CrosswordState {
    type Action = CrosswordAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            CrosswordAction::Start {
                vocabulary,
                bands,
                history,
            } => {
                let session_id = storage::new_session_id();
                let seed = storage::seed_from_session_id(&session_id);
                let game = history.generate(seed, &vocabulary);
                let Some(game) = game else {
                    next.notice = Some(
                        "Unable to generate a connected crossword from this sample. Please retry or adjust the selected levels.",
                    );
                    return Rc::new(next);
                };
                next.game = Some(game);
                next.bands = bands;
                next.session_id = session_id;
                next.clear_interaction();
            }
            CrosswordAction::SelectTile(index) => {
                let target = next.active_clue.and_then(|number| {
                    next.game
                        .as_ref()
                        .and_then(|game| game.next_empty_in_clue(number))
                });
                if let Some((row, col)) = target {
                    next.place_tile(row, col, index);
                } else if next.active_clue.is_some() {
                    next.notice = Some("This clue is complete. Choose another clue.");
                } else {
                    next.selected_tile = Some(index);
                    next.rejected_tile = None;
                }
            }
            CrosswordAction::ActivateClue(number) => {
                let actionable = next.game.as_ref().is_some_and(|game| {
                    game.clues()
                        .iter()
                        .any(|clue| clue.number == number && !game.clue_completed(clue))
                });
                if !actionable {
                    return Rc::new(next);
                }
                next.active_clue = Some(number);
                if let Some(tile) = next.selected_tile {
                    if let Some((row, col)) = next
                        .game
                        .as_ref()
                        .and_then(|game| game.next_empty_in_clue(number))
                    {
                        next.place_tile(row, col, tile);
                    }
                }
            }
            CrosswordAction::Place { row, col, tile } => next.place_tile(row, col, tile),
            CrosswordAction::Reset { history } => {
                if let Some(previous) = next.game.as_ref() {
                    let vocabulary = previous.vocabulary().to_vec();
                    let session_id = storage::new_session_id();
                    let mut seed = storage::seed_from_session_id(&session_id);
                    if seed == previous.seed() {
                        seed = seed.wrapping_add(1);
                    }
                    if let Some(candidate) = history.generate(seed, &vocabulary) {
                        next.game = Some(candidate);
                        next.session_id = session_id;
                        next.clear_interaction();
                    } else {
                        next.notice = Some(
                            "Could not generate another connected crossword. Try another puzzle.",
                        );
                    }
                }
            }
        }
        Rc::new(next)
    }
}
