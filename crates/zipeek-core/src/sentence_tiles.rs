#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileMove {
    Add(usize),
    Remove(usize),
    Before { tile: usize, target: usize },
}

pub fn apply_tile_move(selected: &[usize], bank_size: usize, action: TileMove) -> Vec<usize> {
    let mut next = selected.to_vec();
    match action {
        TileMove::Add(tile) if tile < bank_size && !next.contains(&tile) => {
            next.push(tile);
        }
        TileMove::Remove(tile) => {
            next.retain(|selected_tile| *selected_tile != tile);
        }
        TileMove::Before { tile, target } if tile != target => {
            if let (Some(source), Some(destination)) = (
                next.iter().position(|selected_tile| *selected_tile == tile),
                next.iter()
                    .position(|selected_tile| *selected_tile == target),
            ) {
                let moved = next.remove(source);
                let destination = if source < destination {
                    destination - 1
                } else {
                    destination
                };
                next.insert(destination, moved);
            }
        }
        _ => {}
    }
    next
}

pub fn is_complete(selected: &[usize], bank_size: usize) -> bool {
    bank_size > 0 && selected.len() == bank_size && {
        let mut seen = vec![false; bank_size];
        selected.iter().all(|&tile| {
            if let Some(used) = seen.get_mut(tile) {
                if *used {
                    return false;
                }
                *used = true;
                true
            } else {
                false
            }
        })
    }
}

pub fn assembled_answer<'a>(bank: &'a [String], selected: &[usize]) -> Option<String> {
    if !is_complete(selected, bank.len()) {
        return None;
    }
    Some(
        selected
            .iter()
            .map(|&index| bank[index].as_str())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

pub fn assembled_answer_with_separator(
    bank: &[String],
    selected: &[usize],
    separator: &str,
) -> Option<String> {
    if !is_complete(selected, bank.len()) {
        return None;
    }
    Some(
        selected
            .iter()
            .map(|&index| bank[index].as_str())
            .collect::<Vec<_>>()
            .join(separator),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_word_text_uses_distinct_tile_ids() {
        let selected = apply_tile_move(&[], 3, TileMove::Add(0));
        let selected = apply_tile_move(&selected, 3, TileMove::Add(2));
        assert_eq!(selected, vec![0, 2]);
        assert_eq!(apply_tile_move(&selected, 3, TileMove::Add(0)), selected);
        assert_eq!(apply_tile_move(&selected, 3, TileMove::Add(3)), selected);
        assert_eq!(
            assembled_answer(&["I".into(), "see".into(), "I".into()], &[0, 1, 2]),
            Some("I see I".into())
        );
    }

    #[test]
    fn reordering_is_stable_for_left_and_right_moves() {
        let selected = vec![0, 1, 2, 3];
        assert_eq!(
            apply_tile_move(&selected, 4, TileMove::Before { tile: 0, target: 3 }),
            vec![1, 2, 0, 3]
        );
        assert_eq!(
            apply_tile_move(&selected, 4, TileMove::Before { tile: 3, target: 1 }),
            vec![0, 3, 1, 2]
        );
        assert_eq!(
            apply_tile_move(&selected, 4, TileMove::Before { tile: 1, target: 1 }),
            selected
        );
        assert_eq!(
            apply_tile_move(&selected, 4, TileMove::Before { tile: 6, target: 1 }),
            selected
        );
    }

    #[test]
    fn completeness_rejects_duplicates_and_invalid_indices() {
        assert!(is_complete(&[2, 0, 1], 3));
        assert!(!is_complete(&[0, 0, 2], 3));
        assert!(!is_complete(&[0, 1, 3], 3));
        assert!(!is_complete(&[0, 1], 3));
        assert!(!is_complete(&[], 0));
        let bank = ["one".into(), "two".into(), "three".into()];
        assert_eq!(
            assembled_answer(&bank, &[2, 0, 1]),
            Some("three one two".into())
        );
        assert_eq!(assembled_answer(&bank, &[0, 0, 2]), None);
    }

    #[test]
    fn chinese_tiles_join_without_spaces_and_preserve_repeats() {
        let bank = ["我".into(), "喜欢".into(), "我".into(), "。".into()];
        assert_eq!(
            assembled_answer_with_separator(&bank, &[0, 1, 2, 3], ""),
            Some("我喜欢我。".into())
        );
        assert_eq!(
            assembled_answer_with_separator(&bank, &[0, 1, 1, 3], ""),
            None
        );
    }

    #[test]
    fn removal_does_not_affect_other_tile_ids() {
        assert_eq!(
            apply_tile_move(&[0, 1, 2], 3, TileMove::Remove(1)),
            vec![0, 2]
        );
    }
}
