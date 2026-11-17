use super::super::mosaic::{MosaicChoice, MosaicTile, choose_tile};
use std::{collections::BTreeSet, rc::Rc};
use yew::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct MosaicState {
    pub(super) selected: Option<usize>,
    pub(super) matched: BTreeSet<usize>,
    pub(super) feedback: String,
    pub(super) mistakes: u32,
}

pub(super) enum MosaicAction {
    Reset,
    Select {
        index: usize,
        tiles: Vec<MosaicTile>,
    },
}

impl Reducible for MosaicState {
    type Action = MosaicAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            MosaicAction::Reset => Rc::new(Self::default()),
            MosaicAction::Select { index, tiles } => {
                let mut next = (*self).clone();
                match choose_tile(&tiles, &self.matched, self.selected, index) {
                    MosaicChoice::Selected(index) => next.selected = Some(index),
                    MosaicChoice::Deselected => next.selected = None,
                    MosaicChoice::Matched(pair) => {
                        next.matched.insert(pair);
                        next.selected = None;
                        next.feedback = "Pair matched".into();
                    }
                    MosaicChoice::Incorrect => {
                        next.mistakes += 1;
                        next.selected = None;
                        next.feedback = "Not a match. Try again.".into();
                    }
                    MosaicChoice::Unavailable => {}
                }
                Rc::new(next)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_clears_selection_feedback_and_mistakes() {
        let changed = Rc::new(MosaicState {
            selected: Some(2),
            matched: [1usize].into_iter().collect(),
            feedback: "Incorrect".to_owned(),
            mistakes: 3,
        });
        assert_eq!(
            changed.reduce(MosaicAction::Reset).as_ref(),
            &MosaicState::default()
        );
    }

    #[test]
    fn matching_reducer_tracks_feedback_and_mistakes_atomically() {
        let tiles = vec![
            MosaicTile {
                pair: 0,
                is_english: false,
                label: "中".into(),
            },
            MosaicTile {
                pair: 1,
                is_english: false,
                label: "国".into(),
            },
            MosaicTile {
                pair: 0,
                is_english: true,
                label: "middle".into(),
            },
            MosaicTile {
                pair: 1,
                is_english: true,
                label: "country".into(),
            },
        ];
        let state = Rc::new(MosaicState::default()).reduce(MosaicAction::Select {
            index: 0,
            tiles: tiles.clone(),
        });
        let state = state.reduce(MosaicAction::Select {
            index: 3,
            tiles: tiles.clone(),
        });
        assert_eq!(state.mistakes, 1);
        assert_eq!(state.selected, None);
        let state = state.reduce(MosaicAction::Select {
            index: 0,
            tiles: tiles.clone(),
        });
        let state = state.reduce(MosaicAction::Select { index: 2, tiles });
        assert_eq!(state.mistakes, 1);
        assert!(state.matched.contains(&0));
    }
}
