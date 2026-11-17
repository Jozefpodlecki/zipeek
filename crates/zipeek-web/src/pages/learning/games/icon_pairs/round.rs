use super::catalogue::IconWord;

pub(super) const PAIRS_PER_ROUND: usize = 6;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Tile {
    pub word_index: usize,
    pub icon: bool,
}

#[derive(Clone)]
pub(super) struct Round {
    pub words: Vec<IconWord>,
    pub tiles: Vec<Tile>,
    pub selected: Option<Tile>,
    pub matched: Vec<usize>,
    pub attempts: usize,
    pub feedback: String,
    pub round_number: usize,
}

pub(super) fn shuffle<T>(items: &mut [T], mut seed: u64) {
    for index in (1..items.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        items.swap(index, (seed as usize) % (index + 1));
    }
}

impl Round {
    pub fn new(mut pool: Vec<IconWord>, seed: u64, round_number: usize) -> Self {
        shuffle(&mut pool, seed | 1);
        let mut icon_keys = std::collections::BTreeSet::new();
        let mut hanzi_keys = std::collections::BTreeSet::new();
        let mut words = Vec::new();
        for word in pool {
            if !icon_keys.contains(word.icon_key) && !hanzi_keys.contains(word.hanzi) {
                icon_keys.insert(word.icon_key);
                hanzi_keys.insert(word.hanzi);
                words.push(word);
            }
            if words.len() == PAIRS_PER_ROUND {
                break;
            }
        }
        let mut tiles = (0..words.len())
            .flat_map(|word_index| {
                [
                    Tile {
                        word_index,
                        icon: true,
                    },
                    Tile {
                        word_index,
                        icon: false,
                    },
                ]
            })
            .collect::<Vec<_>>();
        shuffle(&mut tiles, seed.wrapping_add(0x9e37_79b9_7f4a_7c15));
        Self {
            words,
            tiles,
            selected: None,
            matched: vec![],
            attempts: 0,
            feedback: "Choose any icon or Chinese word, then its match.".to_owned(),
            round_number,
        }
    }

    pub fn choose(&mut self, tile: Tile) {
        if self.matched.contains(&tile.word_index) {
            return;
        }
        if let Some(first) = self.selected.take() {
            if first == tile {
                return;
            }
            if first.icon == tile.icon {
                self.selected = Some(tile);
                return;
            }
            self.attempts += 1;
            if first.word_index == tile.word_index {
                self.matched.push(tile.word_index);
                self.feedback = format!("Matched: {}", self.words[tile.word_index].hanzi);
            } else {
                self.feedback = "Not a match. Choose another pair.".to_owned();
            }
        } else {
            self.selected = Some(tile);
        }
    }

    pub fn complete(&self) -> bool {
        !self.words.is_empty() && self.matched.len() == self.words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::learning::games::icon_pairs::catalogue::all_words;
    #[test]
    fn matches_only_opposite_tile_types() {
        let mut round = Round::new(all_words(), 17, 1);
        assert_eq!(round.words.len(), 6);
        round.choose(Tile {
            word_index: 0,
            icon: true,
        });
        round.choose(Tile {
            word_index: 1,
            icon: true,
        });
        assert_eq!(round.attempts, 0);
        round.choose(Tile {
            word_index: 1,
            icon: false,
        });
        assert_eq!(round.matched, vec![1]);
    }
    #[test]
    fn each_round_has_unique_icons_and_twelve_mixed_tiles() {
        let round = Round::new(all_words(), 19, 1);
        assert_eq!(round.tiles.len(), 12);
        assert_eq!(
            round
                .words
                .iter()
                .map(|word| word.hanzi)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            6
        );
        assert_eq!(round.tiles.iter().filter(|tile| tile.icon).count(), 6);
    }
}
