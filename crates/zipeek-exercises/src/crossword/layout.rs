use super::*;

pub(super) fn candidate_cells(clue: &Clue) -> Vec<((usize, usize), char)> {
    clue.answer
        .chars()
        .enumerate()
        .map(|(i, ch)| (clue_position(clue, i), ch))
        .collect()
}

fn placement_score(existing: &[Clue], word: &Clue) -> Option<i32> {
    let occupied: BTreeMap<_, _> = existing
        .iter()
        .flat_map(|clue| candidate_cells(clue))
        .collect();
    let mut crosses = 0;
    let length = word.answer.chars().count();
    let (end_row, end_col) = clue_position(word, length - 1);
    if end_row >= BOARD_SIZE || end_col >= BOARD_SIZE {
        return None;
    }
    let before = match word.direction {
        Direction::Across => word.col.checked_sub(1).map(|c| (word.row, c)),
        Direction::Down => word.row.checked_sub(1).map(|r| (r, word.col)),
    };
    let after = match word.direction {
        Direction::Across => (end_col + 1 < BOARD_SIZE).then_some((end_row, end_col + 1)),
        Direction::Down => (end_row + 1 < BOARD_SIZE).then_some((end_row + 1, end_col)),
    };
    if before.is_some_and(|p| occupied.contains_key(&p))
        || after.is_some_and(|p| occupied.contains_key(&p))
    {
        return None;
    }
    for (pos, ch) in candidate_cells(word) {
        if let Some(&other) = occupied.get(&pos) {
            if ch != other
                || existing.iter().any(|clue| {
                    clue.direction == word.direction
                        && candidate_cells(clue).iter().any(|(p, _)| *p == pos)
                })
            {
                return None;
            }
            crosses += 1;
        } else {
            let neighbors = match word.direction {
                Direction::Across => [
                    pos.0.checked_sub(1).map(|r| (r, pos.1)),
                    (pos.0 + 1 < BOARD_SIZE).then_some((pos.0 + 1, pos.1)),
                ],
                Direction::Down => [
                    pos.1.checked_sub(1).map(|c| (pos.0, c)),
                    (pos.1 + 1 < BOARD_SIZE).then_some((pos.0, pos.1 + 1)),
                ],
            };
            if neighbors
                .into_iter()
                .flatten()
                .any(|p| occupied.contains_key(&p))
            {
                return None;
            }
        }
    }
    if !existing.is_empty() && crosses == 0 {
        return None;
    }
    let mut points = occupied.keys().copied().collect::<Vec<_>>();
    points.extend(candidate_cells(word).into_iter().map(|(p, _)| p));
    let min_r = points.iter().map(|p| p.0).min()?;
    let max_r = points.iter().map(|p| p.0).max()?;
    let min_c = points.iter().map(|p| p.1).min()?;
    let max_c = points.iter().map(|p| p.1).max()?;
    let area = ((max_r - min_r + 1) * (max_c - min_c + 1)) as i32;
    Some(crosses * 80 - area * 2 + length as i32)
}

pub(super) fn next_random(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545F4914F6CDD1D)
}

pub(super) fn build_layout(seed: u64, vocabulary: &[CrosswordWord]) -> Vec<Clue> {
    let mut best = Vec::new();
    for attempt in 0..10u64 {
        let mut rng = seed.wrapping_add(attempt.wrapping_mul(0x9e3779b97f4a7c15)) | 1;
        let mut choices = vocabulary.to_vec();
        if choices.is_empty() {
            return Vec::new();
        }
        for i in (1..choices.len()).rev() {
            let j = (next_random(&mut rng) as usize) % (i + 1);
            choices.swap(i, j);
        }

        let frequency = character_frequencies(&choices);
        let scores: Vec<usize> = choices
            .iter()
            .map(|word| connection_score(word, &frequency))
            .collect();
        let strongest = scores.iter().copied().max().unwrap_or(0);
        let anchors: Vec<usize> = scores
            .iter()
            .enumerate()
            .filter_map(|(index, &score)| (score * 4 >= strongest * 3).then_some(index))
            .collect();
        let anchor_index = anchors[(next_random(&mut rng) as usize) % anchors.len()];
        let first = choices.remove(anchor_index);
        let direction = if next_random(&mut rng) & 1 == 0 {
            Direction::Across
        } else {
            Direction::Down
        };
        let first = positioned(&first, BOARD_SIZE / 2, BOARD_SIZE / 2 - 1, direction);
        let mut placed = vec![first];
        loop {
            if placed.len() >= 20 {
                break;
            }
            let mut selected: Option<(usize, Clue, i32)> = None;

            let occupied: Vec<((usize, usize), char)> =
                placed.iter().flat_map(candidate_cells).collect();
            for (index, source) in choices.iter().enumerate() {
                for direction in [Direction::Across, Direction::Down] {
                    for (offset, character) in source.answer.chars().enumerate() {
                        for &((cross_row, cross_col), existing_character) in &occupied {
                            if character != existing_character {
                                continue;
                            }
                            let origin = match direction {
                                Direction::Across => {
                                    cross_col.checked_sub(offset).map(|col| (cross_row, col))
                                }
                                Direction::Down => {
                                    cross_row.checked_sub(offset).map(|row| (row, cross_col))
                                }
                            };
                            let Some((row, col)) = origin else {
                                continue;
                            };
                            let candidate = positioned(source, row, col, direction);
                            if let Some(score) = placement_score(&placed, &candidate) {
                                let tie = (next_random(&mut rng) % 7) as i32;
                                if selected
                                    .as_ref()
                                    .is_none_or(|(_, _, old)| score + tie > *old)
                                {
                                    selected = Some((index, candidate, score + tie));
                                }
                            }
                        }
                    }
                }
            }
            let Some((index, candidate, _)) = selected else {
                break;
            };
            placed.push(candidate);
            choices.remove(index);
        }
        let coverage: usize = placed.iter().map(|c| c.answer.chars().count()).sum();
        let best_coverage: usize = best.iter().map(|c: &Clue| c.answer.chars().count()).sum();
        if placed.len() > best.len() || (placed.len() == best.len() && coverage > best_coverage) {
            best = placed;
        }
    }
    let min_r = best.iter().map(|c| c.row).min().unwrap_or(0);
    let min_c = best.iter().map(|c| c.col).min().unwrap_or(0);
    let max_r = best
        .iter()
        .map(|c| clue_position(c, c.answer.chars().count() - 1).0)
        .max()
        .unwrap_or(0);
    let max_c = best
        .iter()
        .map(|c| clue_position(c, c.answer.chars().count() - 1).1)
        .max()
        .unwrap_or(0);
    let offset_r = (BOARD_SIZE - (max_r - min_r + 1)) / 2;
    let offset_c = (BOARD_SIZE - (max_c - min_c + 1)) / 2;
    for (index, clue) in best.iter_mut().enumerate() {
        clue.row = clue.row - min_r + offset_r;
        clue.col = clue.col - min_c + offset_c;
        clue.number = (index + 1) as u8;
    }
    best
}

pub(super) fn character_frequencies(words: &[CrosswordWord]) -> BTreeMap<char, usize> {
    let mut frequency = BTreeMap::new();
    for word in words {
        for character in word.answer.chars().collect::<BTreeSet<_>>() {
            *frequency.entry(character).or_insert(0) += 1;
        }
    }
    frequency
}

fn connection_score(word: &CrosswordWord, frequency: &BTreeMap<char, usize>) -> usize {
    word.answer
        .chars()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|character| {
            frequency
                .get(&character)
                .copied()
                .unwrap_or(1)
                .saturating_sub(1)
        })
        .sum()
}

pub(super) fn normalize_vocabulary(words: Vec<CrosswordWord>, seed: u64) -> Vec<CrosswordWord> {
    let mut unique = BTreeMap::new();
    for word in words {
        let answer = word.answer.trim();
        if !(2..=5).contains(&answer.chars().count())
            || !answer
                .chars()
                .all(|ch| ('\u{3400}'..='\u{9fff}').contains(&ch))
            || word.english.trim().is_empty()
            || word.pinyin.trim().is_empty()
        {
            continue;
        }
        unique.entry(answer.to_owned()).or_insert(word);
    }
    let mut vocabulary: Vec<_> = unique.into_values().collect();
    let frequency = character_frequencies(&vocabulary);
    let mut rng = seed | 1;
    let mut ranked: Vec<_> = vocabulary
        .drain(..)
        .map(|word| {
            let score = connection_score(&word, &frequency);
            let tie = next_random(&mut rng);
            (score, tie, word)
        })
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    ranked.truncate(128);

    let mut chosen: Vec<CrosswordWord> = ranked.into_iter().map(|(_, _, word)| word).collect();
    chosen.sort_by(|left, right| left.answer.cmp(&right.answer));
    chosen
}
