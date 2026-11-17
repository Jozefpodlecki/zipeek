use std::collections::{BTreeMap, BTreeSet};

pub const MIN_PAIRS: usize = 6;
pub const MAX_PAIRS: usize = 9;
pub const WAVE_SIZE: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameLength {
    Fixed(usize),
    Endless,
}

impl GameLength {
    pub fn target(self) -> Option<usize> {
        match self {
            Self::Fixed(count) => Some(count),
            Self::Endless => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnMode {
    Waves,
    Continuous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifeLimit {
    Fixed(u8),
    Endless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchMode {
    HanziMeaning,
    HanziPinyin,
    PinyinMeaning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Calm,
    Normal,
    Storm,
}

impl Difficulty {
    pub fn seconds_per_wave(self) -> u32 {
        match self {
            Self::Calm => 105,
            Self::Normal => 70,
            Self::Storm => 48,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudPair {
    pub id: i32,
    pub hanzi: String,
    pub pinyin: String,
    pub meaning: String,
}

impl CloudPair {
    pub fn labels(&self, mode: MatchMode) -> (&str, &str) {
        match mode {
            MatchMode::HanziMeaning => (&self.hanzi, &self.meaning),
            MatchMode::HanziPinyin => (&self.hanzi, &self.pinyin),
            MatchMode::PinyinMeaning => (&self.pinyin, &self.meaning),
        }
    }
}

fn normalize(label: &str) -> String {
    label
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

pub fn eligible_pairs(
    candidates: impl IntoIterator<Item = CloudPair>,
    mode: MatchMode,
) -> Vec<CloudPair> {
    let mut left = BTreeSet::new();
    let mut right = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut accepted = Vec::new();
    for pair in candidates {
        let (a, b) = pair.labels(mode);
        let a = normalize(a);
        let b = normalize(b);
        if pair.id <= 0 || a.is_empty() || b.is_empty() || !ids.insert(pair.id) {
            continue;
        }
        if left.contains(&a) || right.contains(&b) {
            continue;
        }
        left.insert(a);
        right.insert(b);
        accepted.push(pair);
    }
    accepted
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloudSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapResult {
    Selected,
    Correct,
    Incorrect,
    Ignored,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReviewReason {
    pub incorrect_pair: bool,
    pub wave_expired: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudGame {
    pub pairs: Vec<CloudPair>,
    pub mode: MatchMode,
    pub difficulty: Difficulty,
    pub seed: u64,
    pub wave: usize,
    pub elapsed: u32,
    pub selected: Option<(CloudSide, i32)>,
    pub matched: BTreeSet<i32>,
    pub score: u32,
    pub mistakes: u32,
    pub timed_out_waves: u32,
    pub review_ids: BTreeSet<i32>,
    pub review_reasons: BTreeMap<i32, ReviewReason>,
    pub best_combo: u32,
    pub elapsed_total: u32,
    pub combo: u32,
    pub lives: u8,
    pub infinite_lives: bool,
    pub length: GameLength,
    pub spawn_mode: SpawnMode,
    pub flow_roster: Vec<CloudPair>,
    pub flow_lanes: BTreeMap<i32, usize>,
    pub flow_next: usize,
    pub matched_total: u32,
    pub paused: bool,
    pub finished: bool,
}

impl CloudGame {
    pub fn new(
        candidates: Vec<CloudPair>,
        mode: MatchMode,
        difficulty: Difficulty,
        seed: u64,
    ) -> Result<Self, String> {
        Self::new_with_lives(candidates, mode, difficulty, seed, false)
    }

    pub fn new_with_lives(
        candidates: Vec<CloudPair>,
        mode: MatchMode,
        difficulty: Difficulty,
        seed: u64,
        infinite_lives: bool,
    ) -> Result<Self, String> {
        let lives = if infinite_lives {
            LifeLimit::Endless
        } else {
            LifeLimit::Fixed(3)
        };
        Self::new_configured(
            candidates,
            mode,
            difficulty,
            seed,
            GameLength::Fixed(MAX_PAIRS),
            lives,
        )
    }

    pub fn new_configured(
        candidates: Vec<CloudPair>,
        mode: MatchMode,
        difficulty: Difficulty,
        seed: u64,
        length: GameLength,
        life_limit: LifeLimit,
    ) -> Result<Self, String> {
        Self::new_with_spawn_mode(
            candidates,
            mode,
            difficulty,
            seed,
            length,
            life_limit,
            SpawnMode::Waves,
        )
    }

    pub fn new_with_spawn_mode(
        candidates: Vec<CloudPair>,
        mode: MatchMode,
        difficulty: Difficulty,
        seed: u64,
        length: GameLength,
        life_limit: LifeLimit,
        spawn_mode: SpawnMode,
    ) -> Result<Self, String> {
        if !matches!(
            length,
            GameLength::Fixed(10 | 20 | 30) | GameLength::Endless | GameLength::Fixed(MAX_PAIRS)
        ) {
            return Err("Cloud Match supports 10, 20, 30 or endless pairs".to_owned());
        }
        if !matches!(life_limit, LifeLimit::Fixed(1..=3) | LifeLimit::Endless) {
            return Err("Cloud Match supports 1, 2, 3 or endless lives".to_owned());
        }
        let mut pairs = eligible_pairs(candidates, mode);
        let required = match length {
            GameLength::Fixed(MAX_PAIRS) => MIN_PAIRS,
            _ => length.target().unwrap_or(MIN_PAIRS).max(MIN_PAIRS),
        };
        if pairs.len() < required {
            return Err(format!(
                "Cloud Match needs at least {required} distinct playable pairs; only {} available.",
                pairs.len()
            ));
        }
        pairs.sort_by_key(|pair| (mix((pair.id as u64) ^ seed), pair.id));
        if let Some(count) = length.target() {
            pairs.truncate(count);
        } else {
            let complete_waves = pairs.len() / WAVE_SIZE * WAVE_SIZE;
            pairs.truncate(complete_waves);
        }
        let infinite_lives = matches!(life_limit, LifeLimit::Endless);
        let lives = match life_limit {
            LifeLimit::Fixed(count) => count,
            LifeLimit::Endless => 0,
        };
        let flow_roster = if spawn_mode == SpawnMode::Continuous {
            pairs.iter().take(WAVE_SIZE).cloned().collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let flow_lanes = flow_roster
            .iter()
            .enumerate()
            .map(|(lane, pair)| (pair.id, lane))
            .collect();
        let flow_next = flow_roster.len();
        Ok(Self {
            pairs,
            mode,
            difficulty,
            seed,
            wave: 0,
            elapsed: 0,
            selected: None,
            matched: BTreeSet::new(),
            score: 0,
            mistakes: 0,
            timed_out_waves: 0,
            review_ids: BTreeSet::new(),
            review_reasons: BTreeMap::new(),
            best_combo: 0,
            elapsed_total: 0,
            combo: 0,
            lives,
            infinite_lives,
            length,
            spawn_mode,
            flow_roster,
            flow_lanes,
            flow_next,
            matched_total: 0,
            paused: false,
            finished: false,
        })
    }

    pub fn wave_pairs(&self) -> &[CloudPair] {
        if self.spawn_mode == SpawnMode::Continuous {
            return &self.flow_roster;
        }
        let start = match self.length {
            GameLength::Fixed(_) => (self.wave * WAVE_SIZE).min(self.pairs.len()),
            GameLength::Endless => (self.wave * WAVE_SIZE) % self.pairs.len(),
        };
        let end = (start + WAVE_SIZE).min(self.pairs.len());
        &self.pairs[start..end]
    }

    pub fn reverse_pair_labels(&self, id: i32) -> bool {
        let slot = if self.spawn_mode == SpawnMode::Continuous {
            self.flow_lanes.get(&id).copied().unwrap_or(0)
        } else {
            self.wave_pairs()
                .iter()
                .position(|pair| pair.id == id)
                .unwrap_or(0)
        };
        let wave_parity = if self.spawn_mode == SpawnMode::Continuous {
            0
        } else {
            self.wave
        };
        (slot + wave_parity + (self.seed as usize & 1)) % 2 == 1
    }

    pub fn label_on_side<'a>(&self, pair: &'a CloudPair, side: CloudSide) -> &'a str {
        let (first, second) = pair.labels(self.mode);
        match (side, self.reverse_pair_labels(pair.id)) {
            (CloudSide::Left, false) | (CloudSide::Right, true) => first,
            (CloudSide::Right, false) | (CloudSide::Left, true) => second,
        }
    }

    pub fn has_hanzi_on_side(&self, pair: &CloudPair, side: CloudSide) -> bool {
        self.mode != MatchMode::PinyinMeaning && self.label_on_side(pair, side) == pair.hanzi
    }

    pub fn ordered_wave_all(&self, side: CloudSide) -> Vec<(usize, &CloudPair)> {
        if self.spawn_mode == SpawnMode::Continuous {
            return self
                .flow_roster
                .iter()
                .map(|pair| {
                    let slot = self.flow_lanes[&pair.id];
                    let lane = match side {
                        CloudSide::Left => slot,
                        CloudSide::Right => (slot * 2 + 1) % WAVE_SIZE,
                    };
                    (lane, pair)
                })
                .collect();
        }
        let mut list = self.wave_pairs().iter().collect::<Vec<_>>();
        let salt = match side {
            CloudSide::Left => 0xabc1,
            CloudSide::Right => 0xdef2,
        };
        list.sort_by_key(|pair| mix((pair.id as u64) ^ self.seed ^ salt ^ (self.wave as u64)));
        list.into_iter().enumerate().collect()
    }

    pub fn ordered_wave(&self, side: CloudSide) -> Vec<(usize, &CloudPair)> {
        self.ordered_wave_all(side)
            .into_iter()
            .filter(|(_, pair)| !self.matched.contains(&pair.id))
            .collect()
    }

    pub fn completed_all_pairs(&self) -> bool {
        self.finished
            && self.length.target().is_some()
            && self.matched_total as usize == self.pairs.len()
    }

    pub fn review_pairs(&self) -> impl Iterator<Item = &CloudPair> {
        self.pairs
            .iter()
            .filter(|pair| self.review_ids.contains(&pair.id))
    }

    pub fn review_with_reasons(&self) -> impl Iterator<Item = (&CloudPair, ReviewReason)> {
        self.pairs.iter().filter_map(|pair| {
            self.review_reasons
                .get(&pair.id)
                .copied()
                .map(|reason| (pair, reason))
        })
    }

    pub fn is_continuous(&self) -> bool {
        self.spawn_mode == SpawnMode::Continuous
    }

    fn replace_continuous_pair(&mut self, id: i32) {
        let Some(index) = self.flow_roster.iter().position(|pair| pair.id == id) else {
            return;
        };
        let lane = self.flow_lanes.remove(&id).unwrap_or(index);
        let replacement = if self.flow_next < self.pairs.len() {
            let next = self.pairs[self.flow_next].clone();
            self.flow_next += 1;
            Some(next)
        } else if self.length == GameLength::Endless {
            let mut next = None;
            for _ in 0..self.pairs.len() {
                let candidate = self.pairs[self.flow_next % self.pairs.len()].clone();
                self.flow_next += 1;
                if !self.flow_roster.iter().any(|pair| pair.id == candidate.id) {
                    next = Some(candidate);
                    break;
                }
            }
            next
        } else {
            None
        };
        if let Some(next) = replacement {
            self.matched.remove(&next.id);
            self.flow_lanes.insert(next.id, lane);
            self.flow_roster[index] = next;
        } else {
            self.flow_roster.remove(index);
        }
        if self.flow_roster.is_empty() {
            self.finished = true;
        }
    }

    pub fn remaining_seconds(&self) -> u32 {
        self.difficulty
            .seconds_per_wave()
            .saturating_sub(self.elapsed)
    }

    pub fn tick(&mut self) {
        if self.paused || self.finished {
            return;
        }
        self.elapsed = self.elapsed.saturating_add(1);
        self.elapsed_total = self.elapsed_total.saturating_add(1);
        if self.is_continuous() {
            return;
        }
        if self.elapsed >= self.difficulty.seconds_per_wave() {
            self.timed_out_waves = self.timed_out_waves.saturating_add(1);
            let skipped = self
                .wave_pairs()
                .iter()
                .filter(|pair| !self.matched.contains(&pair.id))
                .map(|pair| pair.id)
                .collect::<Vec<_>>();
            for id in skipped {
                self.review_ids.insert(id);
                self.review_reasons.entry(id).or_default().wave_expired = true;
            }
            self.combo = 0;
            if !self.infinite_lives {
                self.lives = self.lives.saturating_sub(1);
            }
            self.advance();
        }
    }

    pub fn tap(&mut self, side: CloudSide, id: i32) -> TapResult {
        if self.finished
            || self.paused
            || self.matched.contains(&id)
            || !self.wave_pairs().iter().any(|pair| pair.id == id)
        {
            return TapResult::Ignored;
        }
        match self.selected {
            None => {
                self.selected = Some((side, id));
                TapResult::Selected
            }
            Some((previous, _)) if previous == side => {
                self.selected = Some((side, id));
                TapResult::Selected
            }
            Some((_, previous_id)) => {
                self.selected = None;
                if previous_id == id {
                    if !self.is_continuous() {
                        self.matched.insert(id);
                    }
                    self.matched_total = self.matched_total.saturating_add(1);
                    self.combo = self.combo.saturating_add(1);
                    self.best_combo = self.best_combo.max(self.combo);
                    self.score =
                        self.score
                            .saturating_add(100u32.saturating_add(
                                20u32.saturating_mul(self.combo.saturating_sub(1)),
                            ));
                    if self.is_continuous() {
                        self.replace_continuous_pair(id);
                    } else if self
                        .wave_pairs()
                        .iter()
                        .all(|pair| self.matched.contains(&pair.id))
                    {
                        self.advance();
                    }
                    TapResult::Correct
                } else {
                    self.mistakes = self.mistakes.saturating_add(1);
                    self.review_ids.insert(previous_id);
                    self.review_ids.insert(id);
                    self.review_reasons
                        .entry(previous_id)
                        .or_default()
                        .incorrect_pair = true;
                    self.review_reasons.entry(id).or_default().incorrect_pair = true;
                    self.combo = 0;
                    if !self.infinite_lives {
                        self.lives = self.lives.saturating_sub(1);
                    }
                    if !self.infinite_lives && self.lives == 0 {
                        self.finished = true;
                    }
                    TapResult::Incorrect
                }
            }
        }
    }

    fn advance(&mut self) {
        self.selected = None;
        self.elapsed = 0;
        self.wave += 1;
        if !self.infinite_lives && self.lives == 0 {
            self.finished = true;
        } else if matches!(self.length, GameLength::Endless) {
            if (self.wave * WAVE_SIZE) % self.pairs.len() == 0 {
                self.matched.clear();
            }
        } else if self.wave * WAVE_SIZE >= self.pairs.len() {
            self.finished = true;
        }
    }

    pub fn finish_endless(&mut self) {
        if (matches!(self.length, GameLength::Endless) || self.is_continuous()) && !self.finished {
            self.finished = true;
            self.paused = false;
            self.selected = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixtures() -> Vec<CloudPair> {
        (1..=12)
            .map(|i| CloudPair {
                id: i,
                hanzi: format!("汉{i}"),
                pinyin: format!("han{i}"),
                meaning: format!("word {i}"),
            })
            .collect()
    }
    #[test]
    fn continuous_replenishment_preserves_unmatched_pair_ids_and_lanes() {
        let mut game = CloudGame::new_with_spawn_mode(
            fixtures(),
            MatchMode::HanziMeaning,
            Difficulty::Normal,
            19,
            GameLength::Fixed(10),
            LifeLimit::Fixed(3),
            SpawnMode::Continuous,
        )
        .unwrap();
        let original = game
            .ordered_wave_all(CloudSide::Left)
            .into_iter()
            .map(|(lane, pair)| (pair.id, lane))
            .collect::<Vec<_>>();
        let chosen = original[0].0;
        assert_eq!(game.tap(CloudSide::Left, chosen), TapResult::Selected);
        assert_eq!(game.tap(CloudSide::Right, chosen), TapResult::Correct);
        assert_eq!(game.matched_total, 1);
        assert_eq!(game.wave, 0);
        assert_eq!(game.wave_pairs().len(), 3);
        assert!(!game.wave_pairs().iter().any(|pair| pair.id == chosen));
        let current = game
            .ordered_wave_all(CloudSide::Left)
            .into_iter()
            .map(|(lane, pair)| (pair.id, lane))
            .collect::<Vec<_>>();
        for (id, lane) in original.iter().skip(1) {
            assert!(current.contains(&(*id, *lane)));
        }
        assert!(
            current
                .iter()
                .any(|(id, lane)| *id != chosen && *lane == original[0].1)
        );
    }

    #[test]
    fn continuous_fixed_length_finishes_only_after_target_matches() {
        let mut game = CloudGame::new_with_spawn_mode(
            fixtures(),
            MatchMode::HanziMeaning,
            Difficulty::Storm,
            29,
            GameLength::Fixed(10),
            LifeLimit::Endless,
            SpawnMode::Continuous,
        )
        .unwrap();
        for count in 1..=10 {
            let id = game.wave_pairs()[0].id;
            assert_eq!(game.tap(CloudSide::Left, id), TapResult::Selected);
            assert_eq!(game.tap(CloudSide::Right, id), TapResult::Correct);
            assert_eq!(game.matched_total, count);
            assert_eq!(game.finished, count == 10);
        }
        assert!(game.completed_all_pairs());
    }

    #[test]
    fn continuous_endless_never_duplicates_an_active_pair() {
        let mut game = CloudGame::new_with_spawn_mode(
            fixtures(),
            MatchMode::HanziMeaning,
            Difficulty::Calm,
            33,
            GameLength::Endless,
            LifeLimit::Endless,
            SpawnMode::Continuous,
        )
        .unwrap();
        for _ in 0..60 {
            let id = game.wave_pairs()[0].id;
            game.tap(CloudSide::Left, id);
            assert_eq!(game.tap(CloudSide::Right, id), TapResult::Correct);
            let ids = game
                .wave_pairs()
                .iter()
                .map(|pair| pair.id)
                .collect::<BTreeSet<_>>();
            assert_eq!(ids.len(), WAVE_SIZE);
            assert!(
                !ids.contains(&id),
                "Continuous mode must not immediately repeat a matched word"
            );
        }
        assert!(!game.finished);
        game.finish_endless();
        assert!(game.finished);
    }

    #[test]
    fn matched_pairs_remain_in_stable_render_slots_until_wave_advances() {
        let candidates = (1..=12)
            .map(|id| CloudPair {
                id,
                hanzi: format!("字{id}"),
                pinyin: format!("zi{id}"),
                meaning: format!("meaning {id}"),
            })
            .collect::<Vec<_>>();
        let mut game = CloudGame::new(candidates, MatchMode::HanziMeaning, Difficulty::Normal, 23)
            .expect("playable fixture");
        let original_left = game
            .ordered_wave_all(CloudSide::Left)
            .into_iter()
            .map(|(lane, pair)| (lane, pair.id))
            .collect::<Vec<_>>();
        let original_right = game
            .ordered_wave_all(CloudSide::Right)
            .into_iter()
            .map(|(lane, pair)| (lane, pair.id))
            .collect::<Vec<_>>();
        let first_id = original_left[0].1;
        assert_eq!(game.tap(CloudSide::Left, first_id), TapResult::Selected);
        assert_eq!(game.tap(CloudSide::Right, first_id), TapResult::Correct);
        let updated_left = game
            .ordered_wave_all(CloudSide::Left)
            .into_iter()
            .map(|(lane, pair)| (lane, pair.id))
            .collect::<Vec<_>>();
        let updated_right = game
            .ordered_wave_all(CloudSide::Right)
            .into_iter()
            .map(|(lane, pair)| (lane, pair.id))
            .collect::<Vec<_>>();
        assert_eq!(original_left, updated_left);
        assert_eq!(original_right, updated_right);
        assert_eq!(
            game.ordered_wave(CloudSide::Left).len(),
            original_left.len() - 1
        );
    }

    #[test]
    fn reject_ambiguous_labels_and_missing_values() {
        let mut pairs = fixtures();
        pairs[1].meaning = pairs[0].meaning.clone();
        pairs[2].hanzi.clear();
        assert_eq!(eligible_pairs(pairs, MatchMode::HanziMeaning).len(), 10);
    }
    #[test]
    fn unlimited_lives_keep_mistakes_and_finish_after_all_waves() {
        let mut game = CloudGame::new_with_lives(
            fixtures(),
            MatchMode::HanziMeaning,
            Difficulty::Storm,
            12,
            true,
        )
        .unwrap();
        let ids = game
            .wave_pairs()
            .iter()
            .map(|pair| pair.id)
            .collect::<Vec<_>>();
        assert_eq!(game.tap(CloudSide::Left, ids[0]), TapResult::Selected);
        assert_eq!(game.tap(CloudSide::Right, ids[1]), TapResult::Incorrect);
        assert_eq!(game.mistakes, 1);
        assert_eq!(game.lives, 3);
        while !game.finished {
            for _ in 0..game.difficulty.seconds_per_wave() {
                game.tick();
            }
        }
        assert_eq!(game.lives, 3);
        assert!(game.timed_out_waves > 0);
        assert!(!game.completed_all_pairs());
    }

    #[test]
    fn seeded_order_is_stable() {
        let a =
            CloudGame::new(fixtures(), MatchMode::HanziMeaning, Difficulty::Normal, 42).unwrap();
        let b =
            CloudGame::new(fixtures(), MatchMode::HanziMeaning, Difficulty::Normal, 42).unwrap();
        assert_eq!(a.pairs, b.pairs);
        assert_eq!(
            a.ordered_wave(CloudSide::Right),
            b.ordered_wave(CloudSide::Right)
        );
        assert_eq!(a.pairs.len(), MAX_PAIRS);
    }
    #[test]
    fn matched_pairs_score_once_and_advance_by_wave() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::HanziMeaning, Difficulty::Calm, 7).unwrap();
        let ids = game.wave_pairs().iter().map(|p| p.id).collect::<Vec<_>>();
        for id in ids {
            assert_eq!(game.tap(CloudSide::Left, id), TapResult::Selected);
            assert_eq!(game.tap(CloudSide::Right, id), TapResult::Correct);
        }
        assert_eq!(game.wave, 1);
        assert_eq!(game.score, 360);
        assert_eq!(
            game.tap(CloudSide::Left, game.pairs[0].id),
            TapResult::Ignored
        );
    }
    #[test]
    fn matched_cloud_does_not_shift_other_cloud_lanes() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::HanziMeaning, Difficulty::Normal, 3).unwrap();
        let slots = game
            .ordered_wave(CloudSide::Left)
            .into_iter()
            .map(|(slot, pair)| (slot, pair.id))
            .collect::<Vec<_>>();
        let matched_id = slots[0].1;
        game.tap(CloudSide::Left, matched_id);
        game.tap(CloudSide::Right, matched_id);
        let next = game
            .ordered_wave(CloudSide::Left)
            .into_iter()
            .map(|(slot, pair)| (slot, pair.id))
            .collect::<Vec<_>>();
        assert_eq!(
            next,
            slots
                .into_iter()
                .filter(|(_, id)| *id != matched_id)
                .collect::<Vec<_>>()
        );
    }
    #[test]
    fn mistakes_and_timeouts_cost_lives_but_pause_freezes_time() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::HanziPinyin, Difficulty::Storm, 1).unwrap();
        let ids = game.wave_pairs().iter().map(|p| p.id).collect::<Vec<_>>();
        game.tap(CloudSide::Left, ids[0]);
        assert_eq!(game.tap(CloudSide::Right, ids[1]), TapResult::Incorrect);
        assert_eq!(game.lives, 2);
        game.paused = true;
        for _ in 0..50 {
            game.tick();
        }
        assert_eq!(game.elapsed, 0);
        game.paused = false;
        for _ in 0..Difficulty::Storm.seconds_per_wave() {
            game.tick();
        }
        assert_eq!(game.wave, 1);
        assert_eq!(game.lives, 1);
    }
    #[test]
    fn background_pause_retains_selection_and_wave_time() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::HanziMeaning, Difficulty::Normal, 9).unwrap();
        let id = game.wave_pairs()[0].id;
        assert_eq!(game.tap(CloudSide::Left, id), TapResult::Selected);
        game.paused = true;
        let before = game.clone();
        for _ in 0..100 {
            game.tick();
        }
        assert_eq!(game, before);
        game.paused = false;
        assert_eq!(game.tap(CloudSide::Right, id), TapResult::Correct);
    }
    #[test]
    fn minimum_distinct_pairs_is_enforced() {
        assert!(
            CloudGame::new(
                fixtures()[..5].to_vec(),
                MatchMode::PinyinMeaning,
                Difficulty::Normal,
                0
            )
            .is_err()
        );
    }

    #[test]
    fn incorrect_taps_record_both_confused_words_for_review() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::HanziPinyin, Difficulty::Normal, 42).unwrap();
        let ids = game
            .wave_pairs()
            .iter()
            .map(|pair| pair.id)
            .collect::<Vec<_>>();
        assert_eq!(game.tap(CloudSide::Left, ids[0]), TapResult::Selected);
        assert_eq!(game.tap(CloudSide::Right, ids[1]), TapResult::Incorrect);
        assert_eq!(game.mistakes, 1);
        assert_eq!(game.lives, 2);
        assert_eq!(
            game.review_pairs()
                .map(|pair| pair.id)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([ids[0], ids[1]])
        );
        assert!(!game.completed_all_pairs());
    }

    #[test]
    fn timeout_records_only_unmatched_words_and_preserves_review_across_waves() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::HanziMeaning, Difficulty::Storm, 5).unwrap();
        let first = game
            .wave_pairs()
            .iter()
            .map(|pair| pair.id)
            .collect::<Vec<_>>();
        game.tap(CloudSide::Left, first[0]);
        game.tap(CloudSide::Right, first[0]);
        for _ in 0..Difficulty::Storm.seconds_per_wave() {
            game.tick();
        }
        assert_eq!(game.wave, 1);
        assert_eq!(game.timed_out_waves, 1);
        assert_eq!(game.elapsed_total, Difficulty::Storm.seconds_per_wave());
        assert_eq!(game.review_ids, BTreeSet::from([first[1], first[2]]));
        assert_eq!(game.mistakes, 0);
        game.paused = true;
        game.tick();
        assert_eq!(game.elapsed_total, Difficulty::Storm.seconds_per_wave());
    }

    #[test]
    fn perfect_completion_has_no_review_and_tracks_best_combo() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::HanziMeaning, Difficulty::Calm, 8).unwrap();
        while !game.finished {
            let ids = game
                .wave_pairs()
                .iter()
                .map(|pair| pair.id)
                .collect::<Vec<_>>();
            for id in ids {
                game.tap(CloudSide::Left, id);
                game.tap(CloudSide::Right, id);
            }
        }
        assert!(game.completed_all_pairs());
        assert!(game.review_pairs().next().is_none());
        assert_eq!(game.best_combo, MAX_PAIRS as u32);
        assert_eq!(game.mistakes, 0);
        assert_eq!(game.timed_out_waves, 0);
    }

    #[test]
    fn failing_on_three_wrong_taps_is_not_reported_as_completion() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::PinyinMeaning, Difficulty::Calm, 10).unwrap();
        let ids = game
            .wave_pairs()
            .iter()
            .map(|pair| pair.id)
            .collect::<Vec<_>>();
        for _ in 0..3 {
            game.tap(CloudSide::Left, ids[0]);
            game.tap(CloudSide::Right, ids[1]);
        }
        assert!(game.finished);
        assert!(!game.completed_all_pairs());
        assert_eq!(game.mistakes, 3);
        assert_eq!(game.review_ids.len(), 2);
    }

    #[test]
    fn review_records_both_mistakes_and_timeouts_without_losing_either_reason() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::HanziMeaning, Difficulty::Storm, 12).unwrap();
        let ids = game
            .wave_pairs()
            .iter()
            .map(|pair| pair.id)
            .collect::<Vec<_>>();
        game.tap(CloudSide::Left, ids[0]);
        assert_eq!(game.tap(CloudSide::Right, ids[1]), TapResult::Incorrect);
        for _ in 0..Difficulty::Storm.seconds_per_wave() {
            game.tick();
        }
        let reviewed = game.review_with_reasons().collect::<Vec<_>>();
        assert_eq!(reviewed.len(), 3);
        let first = game.review_reasons.get(&ids[0]).unwrap();
        assert!(first.incorrect_pair && first.wave_expired);
        let second = game.review_reasons.get(&ids[1]).unwrap();
        assert!(second.incorrect_pair && second.wave_expired);
        let third = game.review_reasons.get(&ids[2]).unwrap();
        assert!(!third.incorrect_pair && third.wave_expired);
    }

    #[test]
    fn finish_after_final_wave() {
        let mut game =
            CloudGame::new(fixtures(), MatchMode::PinyinMeaning, Difficulty::Normal, 0).unwrap();
        while !game.finished {
            let ids = game.wave_pairs().iter().map(|p| p.id).collect::<Vec<_>>();
            for id in ids {
                game.tap(CloudSide::Left, id);
                game.tap(CloudSide::Right, id);
            }
        }
        assert_eq!(game.matched.len(), 9);
        assert_eq!(game.lives, 3);
    }
}

#[cfg(test)]
mod configured_tests {
    use super::*;

    fn candidates(count: i32) -> Vec<CloudPair> {
        (1..=count)
            .map(|id| CloudPair {
                id,
                hanzi: format!("汉{id}"),
                pinyin: format!("han{id}"),
                meaning: format!("meaning {id}"),
            })
            .collect()
    }

    #[test]
    fn fixed_lengths_and_lives_are_respected() {
        for count in [10, 20, 30] {
            for lives in [1, 2, 3] {
                let game = CloudGame::new_configured(
                    candidates(40),
                    MatchMode::HanziMeaning,
                    Difficulty::Normal,
                    7,
                    GameLength::Fixed(count),
                    LifeLimit::Fixed(lives),
                )
                .unwrap();
                assert_eq!(game.pairs.len(), count);
                assert_eq!(game.lives, lives);
                assert!(!game.infinite_lives);
            }
        }
    }

    #[test]
    fn endless_cycles_without_losing_score_or_progress() {
        let mut game = CloudGame::new_configured(
            candidates(6),
            MatchMode::HanziMeaning,
            Difficulty::Normal,
            7,
            GameLength::Endless,
            LifeLimit::Endless,
        )
        .unwrap();
        for _ in 0..3 {
            let ids: Vec<i32> = game.wave_pairs().iter().map(|pair| pair.id).collect();
            for id in ids {
                assert_eq!(game.tap(CloudSide::Left, id), TapResult::Selected);
                assert_eq!(game.tap(CloudSide::Right, id), TapResult::Correct);
            }
        }
        assert!(!game.finished);
        assert_eq!(game.matched_total, 9);
        game.finish_endless();
        assert!(game.finished);
        assert_eq!(game.matched_total, 9);
    }

    #[test]
    fn rejects_insufficient_vocabulary_for_fixed_length() {
        assert!(
            CloudGame::new_configured(
                candidates(12),
                MatchMode::HanziMeaning,
                Difficulty::Normal,
                0,
                GameLength::Fixed(20),
                LifeLimit::Fixed(2),
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod mixed_side_tests {
    use super::*;

    fn pairs() -> Vec<CloudPair> {
        (1..=36)
            .map(|id| CloudPair {
                id,
                hanzi: format!("汉{id}"),
                pinyin: format!("han{id}"),
                meaning: format!("meaning{id}"),
            })
            .collect()
    }

    #[test]
    fn both_sides_present_hanzi_and_meaning_without_changing_pair_identity() {
        for spawn in [SpawnMode::Waves, SpawnMode::Continuous] {
            let mut game = CloudGame::new_with_spawn_mode(
                pairs(),
                MatchMode::HanziMeaning,
                Difficulty::Normal,
                18,
                GameLength::Fixed(20),
                LifeLimit::Endless,
                spawn,
            )
            .unwrap();
            for _ in 0..4 {
                let displayed = game.wave_pairs().to_vec();
                if displayed.len() >= 3 {
                    assert!(displayed.iter().any(|p| game.reverse_pair_labels(p.id)));
                    assert!(displayed.iter().any(|p| !game.reverse_pair_labels(p.id)));
                }
                for pair in &displayed {
                    let left = game.label_on_side(pair, CloudSide::Left);
                    let right = game.label_on_side(pair, CloudSide::Right);
                    assert_eq!(
                        (left, right),
                        if game.reverse_pair_labels(pair.id) {
                            (pair.meaning.as_str(), pair.hanzi.as_str())
                        } else {
                            (pair.hanzi.as_str(), pair.meaning.as_str())
                        }
                    );
                }
                let ids = if spawn == SpawnMode::Waves {
                    displayed.iter().map(|pair| pair.id).collect::<Vec<_>>()
                } else {
                    vec![displayed[0].id]
                };
                for id in ids {
                    assert_eq!(game.tap(CloudSide::Left, id), TapResult::Selected);
                    assert_eq!(game.tap(CloudSide::Right, id), TapResult::Correct);
                }
            }
        }
    }
}
