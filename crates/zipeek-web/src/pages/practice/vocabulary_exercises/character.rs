use std::collections::BTreeSet;

use zipeek_core::Word;

pub(super) fn builder_bank(word: &Word, words: &[Word], seed: u64) -> Vec<char> {
    let target: Vec<char> = word.simplified.chars().collect();
    let target_characters = target.iter().copied().collect::<BTreeSet<_>>();
    let mut bank = target.clone();
    let mut distractors = BTreeSet::new();
    for candidate in words {
        for character in candidate.simplified.chars() {
            if bank.len() >= target.len() + 3 {
                break;
            }
            if !target_characters.contains(&character) && distractors.insert(character) {
                bank.push(character);
            }
        }
        if bank.len() >= target.len() + 3 {
            break;
        }
    }
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    for i in (1..bank.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let position = (state.wrapping_mul(0x2545_f491_4f6c_dd1d) % (i as u64 + 1)) as usize;
        bank.swap(i, position);
    }
    bank
}

pub(super) fn reading_for_character(
    character: char,
    preferred: &Word,
    vocabulary: &[Word],
) -> String {
    std::iter::once(preferred)
        .chain(vocabulary.iter())
        .find_map(|word| {
            let characters: Vec<char> = word.simplified.chars().collect();
            let syllables: Vec<&str> = word.pinyin.split_whitespace().collect();
            if characters.len() != syllables.len() {
                return None;
            }
            characters
                .iter()
                .position(|candidate| *candidate == character)
                .map(|index| syllables[index].to_owned())
        })
        .unwrap_or_default()
}

pub(super) fn assembled_answer(bank: &[char], selected: &[usize]) -> Option<String> {
    let mut used = BTreeSet::new();
    let mut answer = String::new();
    for index in selected {
        if !used.insert(*index) {
            return None;
        }
        answer.push(*bank.get(*index)?);
    }
    Some(answer)
}

pub(super) fn add_tile(
    selected: &[usize],
    tile: usize,
    available: usize,
    capacity: usize,
) -> Vec<usize> {
    if tile >= available || selected.len() >= capacity || selected.contains(&tile) {
        return selected.to_vec();
    }
    let mut next = selected.to_vec();
    next.push(tile);
    next
}

pub(super) fn place_tile(
    selected: &[usize],
    tile: usize,
    available: usize,
    capacity: usize,
    slot: usize,
) -> Vec<usize> {
    if tile >= available || slot >= capacity {
        return selected.to_vec();
    }
    let mut next = selected.to_vec();
    if let Some(previous) = next.iter().position(|index| *index == tile) {
        next.remove(previous);
    } else if next.len() >= capacity {
        return selected.to_vec();
    }
    next.insert(slot.min(next.len()), tile);
    next
}

pub(super) fn remove_tile(selected: &[usize], slot: usize) -> Vec<usize> {
    let mut next = selected.to_vec();
    if slot < next.len() {
        next.remove(slot);
    }
    next
}

#[cfg(test)]
mod tests;
