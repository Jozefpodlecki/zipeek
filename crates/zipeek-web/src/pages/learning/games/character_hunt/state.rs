use std::rc::Rc;

use yew::prelude::*;
use zipeek_exercises::character_hunt::{
    Candidate, CharacterHunt, Difficulty, HuntSessionStats, TapResult,
};

#[derive(Clone, PartialEq)]
pub(super) struct HuntState {
    pub candidates: Rc<Vec<Candidate>>,
    pub difficulty: Difficulty,
    pub total_rounds: Option<u16>,
    pub round: u16,
    pub board: Option<CharacterHunt>,
    pub feedback: String,
    pub hint_visible: bool,
    pub consecutive_misses: u32,
    pub highlighted: Vec<usize>,
    pub revealed_words: u32,
    pub stats: HuntSessionStats,
}

pub(super) enum HuntAction {
    Tap(usize),
    ShowHint,
    ClearHint,
    RevealWord,
    NextRound(u64),
}

impl HuntState {
    pub fn new(
        candidates: Vec<Candidate>,
        difficulty: Difficulty,
        total_rounds: Option<u16>,
        seed: u64,
    ) -> Self {
        let mut state = Self {
            candidates: Rc::new(candidates),
            difficulty,
            total_rounds,
            round: 1,
            board: None,
            feedback: String::new(),
            hint_visible: false,
            consecutive_misses: 0,
            highlighted: Vec::new(),
            revealed_words: 0,
            stats: HuntSessionStats::default(),
        };
        state.generate_board(seed);
        state
    }

    pub fn is_last_round(&self) -> bool {
        self.total_rounds.is_some_and(|limit| self.round >= limit)
    }

    fn generate_board(&mut self, seed: u64) {
        match CharacterHunt::generate(&self.candidates, self.difficulty, seed) {
            Ok(board) => {
                self.board = Some(board);
                self.feedback.clear();
            }
            Err(error) => {
                self.board = None;
                let size = self.difficulty.size();
                self.feedback = format!(
                    "Cannot build a solvable {size} × {size} board from this vocabulary: \
                     {error:?}. Choose an easier difficulty or a larger vocabulary source."
                );
            }
        }
        self.hint_visible = false;
        self.consecutive_misses = 0;
        self.highlighted.clear();
    }
}

impl Reducible for HuntState {
    type Action = HuntAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            HuntAction::Tap(index) => {
                next.highlighted.clear();
                next.hint_visible = false;
                if let Some(board) = &mut next.board {
                    match board.tap(index) {
                        Ok(TapResult::Found) => {
                            next.feedback = board
                                .completed_words()
                                .last()
                                .map(|word| match &word.meaning {
                                    Some(meaning) => format!("{} — {meaning}", word.hanzi),
                                    None => format!("Found {}", word.hanzi),
                                })
                                .unwrap_or_else(|| "Word found".to_owned());
                            next.consecutive_misses = 0;
                        }
                        Ok(TapResult::Incorrect) => {
                            next.consecutive_misses = next.consecutive_misses.saturating_add(1);
                            next.feedback = "Not a word on this board. Try again.".to_owned();
                            if matches!(next.consecutive_misses, 3 | 5) {
                                if let Some(indices) = board.hint_indices() {
                                    next.highlighted = if next.consecutive_misses == 3 {
                                        indices.into_iter().take(1).collect()
                                    } else {
                                        indices
                                    };
                                }
                            }
                        }
                        Ok(TapResult::Selected) => next.feedback.clear(),
                        Ok(TapResult::Ignored) | Err(_) => return self,
                    }
                    if board.complete() {
                        next.highlighted.clear();
                    }
                }
            }
            HuntAction::ShowHint => {
                if next
                    .board
                    .as_ref()
                    .is_some_and(|board| !board.complete() && board.selected.is_empty())
                {
                    next.hint_visible = true;
                    next.highlighted = next
                        .board
                        .as_ref()
                        .and_then(CharacterHunt::hint_indices)
                        .unwrap_or_default();
                }
            }
            HuntAction::ClearHint => {
                next.highlighted.clear();
            }
            HuntAction::RevealWord => {
                if next.consecutive_misses < 7 {
                    return self;
                }
                if let Some(board) = &mut next.board {
                    if let Some(word) = board.reveal_word() {
                        next.revealed_words = next.revealed_words.saturating_add(1);
                        next.consecutive_misses = 0;
                        next.highlighted.clear();
                        next.hint_visible = false;
                        next.feedback = format!("Revealed: {word}");
                    }
                }
            }
            HuntAction::NextRound(seed) => {
                if next.is_last_round() {
                    return self;
                }
                let Some(board) = &next.board else {
                    return self;
                };
                let Some(stats) = next.stats.with_completed_round(board) else {
                    return self;
                };

                match CharacterHunt::generate(&next.candidates, next.difficulty, seed) {
                    Ok(new_board) => {
                        next.stats = stats;
                        next.round = next.round.saturating_add(1);
                        next.board = Some(new_board);
                        next.feedback.clear();
                        next.hint_visible = false;
                        next.consecutive_misses = 0;
                        next.highlighted.clear();
                    }
                    Err(error) => {
                        next.feedback = format!(
                            "Could not prepare the next board ({error:?}). Try Next round again."
                        );
                    }
                }
            }
        }
        Rc::new(next)
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct HuntSessionContext(pub UseReducerHandle<HuntState>, pub Option<Callback<()>>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_reports_unsolvable_vocabulary() {
        let state = HuntState::new(Vec::new(), Difficulty::Easy, Some(5), 1);
        assert!(state.board.is_none());
        assert!(state.feedback.contains("Cannot build a solvable"));
    }

    #[test]
    fn premature_next_does_not_count_or_advance() {
        let state = Rc::new(HuntState::new(Vec::new(), Difficulty::Easy, Some(5), 1));
        let next = state.reduce(HuntAction::NextRound(2));
        assert_eq!(next.round, 1);
        assert_eq!(next.stats.completed_rounds, 0);
    }

    #[test]
    fn failed_next_board_preserves_completed_round_and_totals() {
        let candidates = ["中国", "天气"]
            .into_iter()
            .map(|hanzi| Candidate {
                hanzi: hanzi.to_owned(),
                readings: None,
                meaning: None,
            })
            .collect();
        let mut state = HuntState::new(candidates, Difficulty::Easy, Some(5), 1);
        let mut board = state.board.take().expect("two words fill a 2x2 board");
        for word in ["中国", "天气"] {
            for character in word.chars() {
                let index = board
                    .tiles
                    .iter()
                    .position(|tile| tile.character == character)
                    .expect("fixture character appears exactly once");
                board.tap(index).expect("valid fixture selection");
            }
        }
        assert!(board.complete());
        state.board = Some(board);

        state.candidates = Rc::new(Vec::new());

        let next = Rc::new(state).reduce(HuntAction::NextRound(2));
        assert_eq!(next.round, 1);
        assert_eq!(next.stats.completed_rounds, 0);
        assert!(next.board.as_ref().is_some_and(CharacterHunt::complete));
        assert!(next.feedback.contains("Could not prepare the next board"));
    }

    #[test]
    fn repeated_invalid_attempts_show_progressive_spatial_hints() {
        let words = ["中国", "天气"]
            .into_iter()
            .map(|hanzi| Candidate {
                hanzi: hanzi.to_owned(),
                readings: None,
                meaning: None,
            })
            .collect();
        let mut state = Rc::new(HuntState::new(words, Difficulty::Easy, Some(5), 7));
        let board = state.board.as_ref().unwrap();
        let answer = board.hint_indices().unwrap();
        let wrong_index = (0..board.tiles.len())
            .find(|index| !answer.contains(index))
            .unwrap();
        let first = answer[0];
        for miss in 1..=5 {
            state = state.reduce(HuntAction::Tap(first));
            state = state.reduce(HuntAction::Tap(wrong_index));
            assert_eq!(state.consecutive_misses, miss);
            if miss == 3 {
                assert_eq!(state.highlighted.len(), 1);
            }
        }
        assert!(state.highlighted.len() >= 2);
        state = state.reduce(HuntAction::ClearHint);
        assert!(state.highlighted.is_empty());
        for miss in 6..=7 {
            state = state.reduce(HuntAction::Tap(first));
            state = state.reduce(HuntAction::Tap(wrong_index));
            assert_eq!(state.consecutive_misses, miss);
        }
        let before = state.board.as_ref().unwrap().found.len();
        state = state.reduce(HuntAction::RevealWord);
        assert_eq!(state.board.as_ref().unwrap().found.len(), before + 1);
        assert_eq!(state.revealed_words, 1);
        assert_eq!(state.consecutive_misses, 0);
    }

    #[test]
    fn interaction_dismisses_spatial_hint_immediately() {
        let words = ["中国", "天气"]
            .into_iter()
            .map(|hanzi| Candidate {
                hanzi: hanzi.to_owned(),
                readings: None,
                meaning: None,
            })
            .collect();
        let mut state = Rc::new(HuntState::new(words, Difficulty::Easy, Some(5), 5));
        state = state.reduce(HuntAction::ShowHint);
        assert!(!state.highlighted.is_empty());
        let index = state.board.as_ref().unwrap().hint_indices().unwrap()[0];
        state = state.reduce(HuntAction::Tap(index));
        assert!(state.highlighted.is_empty());
        assert!(!state.hint_visible);
        assert_eq!(state.board.as_ref().unwrap().selected, vec![index]);
    }

    #[test]
    fn hint_does_not_interrupt_partial_selection() {
        let words = ["中国", "天气"]
            .into_iter()
            .map(|hanzi| Candidate {
                hanzi: hanzi.to_owned(),
                readings: None,
                meaning: None,
            })
            .collect();
        let mut state = Rc::new(HuntState::new(words, Difficulty::Easy, Some(5), 5));
        let index = state.board.as_ref().unwrap().hint_indices().unwrap()[0];
        state = state.reduce(HuntAction::Tap(index));
        let attempted = state.reduce(HuntAction::ShowHint);
        assert_eq!(attempted.board.as_ref().unwrap().selected, vec![index]);
        assert!(attempted.highlighted.is_empty());
    }
}
