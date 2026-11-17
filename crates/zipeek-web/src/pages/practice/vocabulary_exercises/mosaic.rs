use std::collections::BTreeSet;

use zipeek_core::Word;

use super::vocabulary::meaning;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MosaicTile {
    pub pair: usize,
    pub is_english: bool,
    pub label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MosaicChoice {
    Selected(usize),
    Deselected,
    Matched(usize),
    Incorrect,
    Unavailable,
}

pub(super) fn mosaic_tiles(words: &[Word], seed: u64) -> Vec<MosaicTile> {
    let mut tiles = Vec::with_capacity(words.len() * 2);
    for (index, word) in words.iter().enumerate() {
        tiles.push(MosaicTile {
            pair: index,
            is_english: false,
            label: word.simplified.clone(),
        });
    }
    for (index, word) in words.iter().enumerate().rev() {
        tiles.push(MosaicTile {
            pair: index,
            is_english: true,
            label: meaning(word).unwrap_or_default().to_owned(),
        });
    }
    let mut state = seed ^ 0xd1b5_4a32_d192_ed03;
    for i in (1..tiles.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let position = (state.wrapping_mul(0x2545_f491_4f6c_dd1d) % (i as u64 + 1)) as usize;
        tiles.swap(i, position);
    }
    tiles
}

pub(super) fn choose_tile(
    tiles: &[MosaicTile],
    matched: &BTreeSet<usize>,
    previous: Option<usize>,
    clicked: usize,
) -> MosaicChoice {
    let Some(current) = tiles.get(clicked) else {
        return MosaicChoice::Unavailable;
    };
    if matched.contains(&current.pair) {
        return MosaicChoice::Unavailable;
    }
    let Some(previous) = previous else {
        return MosaicChoice::Selected(clicked);
    };
    if previous == clicked {
        return MosaicChoice::Deselected;
    }
    let Some(other) = tiles.get(previous) else {
        return MosaicChoice::Selected(clicked);
    };
    if matched.contains(&other.pair) {
        return MosaicChoice::Selected(clicked);
    }
    if other.pair == current.pair && other.is_english != current.is_english {
        MosaicChoice::Matched(current.pair)
    } else {
        MosaicChoice::Incorrect
    }
}

pub(super) fn mosaic_complete(matched: &BTreeSet<usize>, tiles: &[MosaicTile]) -> bool {
    !tiles.is_empty()
        && matched.len() * 2 == tiles.len()
        && matched.iter().all(|pair| {
            tiles
                .iter()
                .filter(|tile| tile.pair == *pair && tile.is_english)
                .count()
                == 1
                && tiles
                    .iter()
                    .filter(|tile| tile.pair == *pair && !tile.is_english)
                    .count()
                    == 1
        })
}

#[cfg(test)]
mod tests;
