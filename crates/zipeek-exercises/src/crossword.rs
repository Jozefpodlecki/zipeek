use std::collections::{BTreeMap, BTreeSet};

pub const BOARD_SIZE: usize = 15;

pub const MIN_CROSSWORD_WORDS: usize = 6;
pub const MIN_CROSSWORD_INTERSECTIONS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Across,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clue {
    pub number: u8,
    pub row: usize,
    pub col: usize,
    pub direction: Direction,
    pub answer: String,
    pub english: String,
    pub pinyin: String,
}

fn clue_position(clue: &Clue, offset: usize) -> (usize, usize) {
    match clue.direction {
        Direction::Across => (clue.row, clue.col + offset),
        Direction::Down => (clue.row + offset, clue.col),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrosswordWord {
    pub answer: String,
    pub english: String,
    pub pinyin: String,
}

fn positioned(word: &CrosswordWord, row: usize, col: usize, direction: Direction) -> Clue {
    Clue {
        number: 0,
        row,
        col,
        direction,
        answer: word.answer.clone(),
        english: word.english.clone(),
        pinyin: word.pinyin.clone(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Correct,
    Wrong,
    Occupied,
    Invalid,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinyinTile {
    pub bank_index: usize,
    pub character: char,
    pub syllable: String,
    pub tone: u8,
}

pub fn normalize_pinyin(value: &str) -> (String, u8) {
    let mut base = String::new();
    let mut tone = 5;
    for ch in value.trim().to_lowercase().chars() {
        let pair = match ch {
            'ā' => ('a', 1),
            'á' => ('a', 2),
            'ǎ' => ('a', 3),
            'à' => ('a', 4),
            'ē' => ('e', 1),
            'é' => ('e', 2),
            'ě' => ('e', 3),
            'è' => ('e', 4),
            'ī' => ('i', 1),
            'í' => ('i', 2),
            'ǐ' => ('i', 3),
            'ì' => ('i', 4),
            'ō' => ('o', 1),
            'ó' => ('o', 2),
            'ǒ' => ('o', 3),
            'ò' => ('o', 4),
            'ū' => ('u', 1),
            'ú' => ('u', 2),
            'ǔ' => ('u', 3),
            'ù' => ('u', 4),
            'ǖ' => ('v', 1),
            'ǘ' => ('v', 2),
            'ǚ' => ('v', 3),
            'ǜ' => ('v', 4),
            'ü' => ('v', 5),
            _ => (ch, 5),
        };
        if pair.1 != 5 {
            tone = pair.1;
        }
        match pair.0 {
            '1'..='5' => tone = pair.0 as u8 - b'0',
            ':' => {
                if base.ends_with('u') {
                    base.pop();
                    base.push('v');
                }
            }
            'a'..='z' => base.push(pair.0),
            _ => {}
        }
    }
    (base, tone)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crossword {
    expected: BTreeMap<(usize, usize), char>,
    filled: BTreeMap<(usize, usize), char>,
    bank: Vec<char>,
    clues: Vec<Clue>,
    seed: u64,
    vocabulary: Vec<CrosswordWord>,
    pub score: u32,
    pub mistakes: u32,
}

impl Default for Crossword {
    fn default() -> Self {
        Self::empty()
    }
}

impl Crossword {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn empty() -> Self {
        Self {
            expected: BTreeMap::new(),
            filled: BTreeMap::new(),
            bank: Vec::new(),
            clues: Vec::new(),
            seed: 0,
            vocabulary: Vec::new(),
            score: 0,
            mistakes: 0,
        }
    }

    pub fn vocabulary(&self) -> &[CrosswordWord] {
        &self.vocabulary
    }

    pub fn clues(&self) -> &[Clue] {
        &self.clues
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn meets_quality_floor(&self) -> bool {
        self.clues.len() >= MIN_CROSSWORD_WORDS
            && self.intersection_count() >= MIN_CROSSWORD_INTERSECTIONS
            && self.expected.len() >= 10
    }

    pub fn cells(&self) -> impl Iterator<Item = ((usize, usize), char, Option<char>)> + '_ {
        self.expected
            .iter()
            .map(|(position, target)| (*position, *target, self.filled.get(position).copied()))
    }

    pub fn bank(&self) -> &[char] {
        &self.bank
    }

    pub fn pinyin_tiles(&self) -> Vec<PinyinTile> {
        let mut readings: BTreeMap<char, BTreeSet<(String, u8)>> = BTreeMap::new();
        for clue in &self.clues {
            for (character, syllable) in clue.answer.chars().zip(clue.pinyin.split_whitespace()) {
                let (base, tone) = normalize_pinyin(syllable);
                if !base.is_empty() {
                    readings.entry(character).or_default().insert((base, tone));
                }
            }
        }
        self.bank
            .iter()
            .copied()
            .enumerate()
            .flat_map(|(bank_index, character)| {
                readings
                    .get(&character)
                    .into_iter()
                    .flat_map(move |variants| {
                        variants.iter().map(move |(syllable, tone)| PinyinTile {
                            bank_index,
                            character,
                            syllable: syllable.clone(),
                            tone: *tone,
                        })
                    })
            })
            .collect()
    }

    pub fn next_empty_in_clue(&self, clue_number: u8) -> Option<(usize, usize)> {
        let clue = self.clues.iter().find(|clue| clue.number == clue_number)?;
        (0..clue.answer.chars().count())
            .map(|offset| clue_position(clue, offset))
            .find(|position| !self.filled.contains_key(position))
    }

    pub fn intersection_positions(&self) -> BTreeSet<(usize, usize)> {
        let mut memberships: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for clue in &self.clues {
            for (offset, _) in clue.answer.chars().enumerate() {
                let position = clue_position(clue, offset);
                *memberships.entry(position).or_default() += 1;
            }
        }
        memberships
            .into_iter()
            .filter_map(|(position, count)| (count > 1).then_some(position))
            .collect()
    }

    pub fn intersection_count(&self) -> usize {
        self.intersection_positions().len()
    }

    pub fn total(&self) -> usize {
        self.expected.len()
    }

    pub fn completed(&self) -> usize {
        self.filled.len()
    }

    pub fn finished(&self) -> bool {
        !self.expected.is_empty() && self.filled.len() == self.expected.len()
    }

    pub fn clue_completed(&self, clue: &Clue) -> bool {
        clue.answer.chars().enumerate().all(|(offset, expected)| {
            let position = match clue.direction {
                Direction::Across => (clue.row, clue.col + offset),
                Direction::Down => (clue.row + offset, clue.col),
            };
            self.filled.get(&position) == Some(&expected)
        })
    }

    pub fn place(&mut self, row: usize, col: usize, bank_index: usize) -> Placement {
        let position = (row, col);
        let Some(expected) = self.expected.get(&position) else {
            return Placement::Invalid;
        };
        if self.filled.contains_key(&position) {
            return Placement::Occupied;
        }
        let Some(&character) = self.bank.get(bank_index) else {
            return Placement::Invalid;
        };
        if character != *expected {
            self.mistakes += 1;
            return Placement::Wrong;
        }
        self.filled.insert(position, character);
        self.bank.remove(bank_index);
        self.score += 100;
        if self.finished() {
            Placement::Finished
        } else {
            Placement::Correct
        }
    }

    pub fn cell_numbers(&self) -> BTreeMap<(usize, usize), Vec<u8>> {
        let mut numbers: BTreeMap<(usize, usize), Vec<u8>> = BTreeMap::new();
        for clue in &self.clues {
            numbers
                .entry((clue.row, clue.col))
                .or_default()
                .push(clue.number);
        }
        numbers
    }

    pub fn solution_is_consistent(&self) -> bool {
        let unique: BTreeSet<_> = self.expected.keys().copied().collect();
        if unique.is_empty() || self.bank.len() + self.filled.len() != self.expected.len() {
            return false;
        }
        if self
            .filled
            .iter()
            .any(|(position, character)| self.expected.get(position) != Some(character))
        {
            return false;
        }
        let mut remaining = self.expected.values().copied().collect::<Vec<_>>();
        for character in self.filled.values().chain(self.bank.iter()) {
            let Some(index) = remaining.iter().position(|item| item == character) else {
                return false;
            };
            remaining.remove(index);
        }
        remaining.is_empty()
    }
}

fn candidate_cells(clue: &Clue) -> Vec<((usize, usize), char)> {
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

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545F4914F6CDD1D)
}

fn build_layout(seed: u64, vocabulary: &[CrosswordWord]) -> Vec<Clue> {
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

fn character_frequencies(words: &[CrosswordWord]) -> BTreeMap<char, usize> {
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

fn normalize_vocabulary(words: Vec<CrosswordWord>, seed: u64) -> Vec<CrosswordWord> {
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

impl Crossword {
    pub fn generated(seed: u64, vocabulary: Vec<CrosswordWord>) -> Self {
        let vocabulary = normalize_vocabulary(vocabulary, seed);
        let clues = build_layout(seed, &vocabulary);
        let mut expected = BTreeMap::new();
        for clue in &clues {
            for (position, ch) in candidate_cells(clue) {
                if let Some(previous) = expected.insert(position, ch) {
                    assert_eq!(previous, ch);
                }
            }
        }
        let mut bank: Vec<_> = expected.values().copied().collect();
        let mut rng = seed | 1;
        for i in (1..bank.len()).rev() {
            let j = (next_random(&mut rng) as usize) % (i + 1);
            bank.swap(i, j);
        }
        Self {
            expected,
            filled: BTreeMap::new(),
            bank,
            clues,
            seed,
            vocabulary,
            score: 0,
            mistakes: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn words() -> Vec<CrosswordWord> {
        [
            ("学生", "student", "xué shēng"),
            ("生活", "daily life", "shēng huó"),
            ("活动", "activity", "huó dòng"),
            ("动物", "animal", "dòng wù"),
            ("物理", "physics", "wù lǐ"),
            ("理想", "ideal", "lǐ xiǎng"),
            ("想法", "thought", "xiǎng fǎ"),
            ("老师", "teacher", "lǎo shī"),
            ("老人", "elder", "lǎo rén"),
            ("人生", "life", "rén shēng"),
            ("生日", "birthday", "shēng rì"),
            ("日记", "diary", "rì jì"),
            ("法师", "master", "fǎ shī"),
        ]
        .into_iter()
        .map(|(answer, english, pinyin)| CrosswordWord {
            answer: answer.into(),
            english: english.into(),
            pinyin: pinyin.into(),
        })
        .collect()
    }
    #[test]
    fn generated_puzzles_are_reproducible_and_valid() {
        for seed in 1..=12 {
            let game = Crossword::generated(seed, words());
            assert_eq!(game, Crossword::generated(seed, words()));
            assert!(game.solution_is_consistent());
            assert!(game.clues().len() >= 3);
            assert!(game.intersection_count() >= 2);
            assert_eq!(game.total(), game.bank().len());
        }
    }
    #[test]
    fn connected_candidates_survive_large_dictionary_samples() {
        let mut candidates = Vec::new();
        for first in 0..15u32 {
            for second in 0..15u32 {
                if first == second {
                    continue;
                }
                let answer = [
                    char::from_u32(0x4e00 + first).unwrap(),
                    char::from_u32(0x4e00 + second).unwrap(),
                ]
                .into_iter()
                .collect();
                candidates.push(CrosswordWord {
                    answer,
                    english: "synthetic vocabulary".into(),
                    pinyin: "test".into(),
                });
            }
        }
        let sample = normalize_vocabulary(candidates, 42);
        assert_eq!(sample.len(), 128);
        assert!(sample.iter().all(|word| word.answer.chars().count() == 2));
        assert!(
            character_frequencies(&sample)
                .values()
                .any(|count| *count >= 10)
        );
        assert_eq!(normalize_vocabulary(sample.clone(), 42), sample);
    }

    #[test]
    fn generated_layouts_vary_by_seed() {
        let layouts: BTreeSet<_> = (1..=24)
            .map(|seed| {
                Crossword::generated(seed, words())
                    .clues()
                    .iter()
                    .map(|clue| {
                        (
                            clue.answer.clone(),
                            clue.row,
                            clue.col,
                            clue.direction == Direction::Across,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(layouts.len() >= 3);
    }
    #[test]
    fn clue_navigation_skips_filled_crossings_and_completed_clues() {
        let mut game = Crossword::generated(123, words());
        let clue = game.clues().first().expect("generated clue").clone();
        let coordinates: Vec<_> = (0..clue.answer.chars().count())
            .map(|offset| clue_position(&clue, offset))
            .collect();
        assert_eq!(game.next_empty_in_clue(clue.number), Some(coordinates[0]));
        assert_eq!(game.next_empty_in_clue(u8::MAX), None);

        if let Some((row, col)) = coordinates
            .iter()
            .copied()
            .find(|position| game.intersection_positions().contains(position))
        {
            let character = game
                .cells()
                .find(|(pos, _, _)| *pos == (row, col))
                .unwrap()
                .1;
            let tile = game.bank().iter().position(|ch| *ch == character).unwrap();
            assert!(matches!(
                game.place(row, col, tile),
                Placement::Correct | Placement::Finished
            ));
        }
        for (index, (row, col)) in coordinates.iter().copied().enumerate() {
            if game
                .cells()
                .any(|(position, _, filled)| position == (row, col) && filled.is_some())
            {
                continue;
            }
            let character = clue.answer.chars().nth(index).unwrap();
            let tile = game.bank().iter().position(|ch| *ch == character).unwrap();
            assert!(matches!(
                game.place(row, col, tile),
                Placement::Correct | Placement::Finished
            ));
            assert_eq!(
                game.next_empty_in_clue(clue.number),
                coordinates
                    .iter()
                    .copied()
                    .skip(index + 1)
                    .find(|position| game
                        .cells()
                        .any(|(pos, _, filled)| pos == *position && filled.is_none()))
            );
        }
        assert!(game.clue_completed(&clue));
    }

    #[test]
    fn all_generated_cells_are_solvable_and_scored_once() {
        let mut game = Crossword::generated(123, words());
        let positions: Vec<_> = game.cells().map(|(pos, ch, _)| (pos, ch)).collect();
        assert!(!positions.is_empty());
        for ((row, col), character) in positions {
            let index = game.bank().iter().position(|ch| *ch == character).unwrap();
            assert!(matches!(
                game.place(row, col, index),
                Placement::Correct | Placement::Finished
            ));
        }
        assert!(game.finished());
        assert!(game.clues().iter().all(|clue| game.clue_completed(clue)));
        assert_eq!(game.score, game.total() as u32 * 100);
    }
    #[test]
    fn pinyin_normalizes_tone_marks_and_numeric_forms() {
        assert_eq!(normalize_pinyin("xué"), ("xue".into(), 2));
        assert_eq!(normalize_pinyin("xue2"), ("xue".into(), 2));
        assert_eq!(normalize_pinyin("xue1"), ("xue".into(), 1));
        assert_eq!(normalize_pinyin("nǚ"), ("nv".into(), 3));
        assert_eq!(normalize_pinyin("nu:3"), ("nv".into(), 3));
        assert_eq!(normalize_pinyin("lǜ"), ("lv".into(), 4));
    }

    #[test]
    fn pinyin_candidates_use_remaining_bank_and_dictionary_readings() {
        let mut game = Crossword::generated(123, words());

        let clue = game.clues().first().expect("generated puzzle has clues");
        let character = clue.answer.chars().next().expect("clue has a character");
        let syllable = clue
            .pinyin
            .split_whitespace()
            .next()
            .expect("clue has a reading");
        let (expected_syllable, expected_tone) = normalize_pinyin(syllable);
        let position = game
            .cells()
            .find(|(_, candidate, _)| *candidate == character)
            .expect("clue character must appear on board")
            .0;
        let bank_index = game
            .bank()
            .iter()
            .position(|candidate| *candidate == character)
            .expect("clue character must appear in bank");
        let before = game.pinyin_tiles();
        assert!(before.iter().any(|tile| {
            tile.bank_index == bank_index
                && tile.character == character
                && tile.syllable == expected_syllable
                && tile.tone == expected_tone
        }));
        assert!(matches!(
            game.place(position.0, position.1, bank_index),
            Placement::Correct | Placement::Finished
        ));

        assert_eq!(
            game.pinyin_tiles().len(),
            before
                .iter()
                .filter(|tile| tile.bank_index != bank_index)
                .count()
        );
    }

    #[test]
    fn rejects_tiny_crosswords_as_unplayable() {
        let small = Crossword::generated(
            42,
            vec![
                CrosswordWord {
                    answer: "学生".into(),
                    english: "student".into(),
                    pinyin: "xué shēng".into(),
                },
                CrosswordWord {
                    answer: "生活".into(),
                    english: "life".into(),
                    pinyin: "shēng huó".into(),
                },
            ],
        );
        assert!(!small.meets_quality_floor());
    }

    #[test]
    fn does_not_fallback_to_unselected_vocabulary() {
        let empty = Crossword::generated(7, Vec::new());
        assert_eq!(empty.total(), 0);
        assert!(!empty.finished());
        let malformed = Crossword::generated(
            7,
            vec![CrosswordWord {
                answer: "hello".into(),
                english: "hello".into(),
                pinyin: "hello".into(),
            }],
        );
        assert_eq!(malformed.total(), 0);
    }
}
