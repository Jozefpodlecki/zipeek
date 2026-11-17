use std::collections::HashSet;

pub const ROUND_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarketDifficulty {
    Easy,
    Medium,
    Hard,
}

impl MarketDifficulty {
    pub fn label(self) -> &'static str {
        match self {
            Self::Easy => "Whole yuan",
            Self::Medium => "Yuan and jiao (including below ¥1)",
            Self::Hard => "Yuan, jiao and fen (including below ¥1)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarketSupport {
    Guided,
    Reading,
    Listening,
}

impl MarketSupport {
    pub fn label(self) -> &'static str {
        match self {
            Self::Guided => "Guided",
            Self::Reading => "Reading practice",
            Self::Listening => "Listening challenge",
        }
    }

    pub fn show_hanzi(self, revealed: bool) -> bool {
        self == Self::Guided || self == Self::Reading || revealed
    }

    pub fn show_pinyin(self, revealed: bool) -> bool {
        self == Self::Guided || revealed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketQuestion {
    pub prompt: String,
    pub pinyin: String,
    pub choices: [String; 3],
    pub answer: usize,
    pub amount_fen: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarketAction {
    Answer(usize),
    NewGame(MarketDifficulty, MarketSupport, u64),
    RevealHelp,
    Continue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketState {
    pub difficulty: MarketDifficulty,
    pub support: MarketSupport,
    pub help_revealed: bool,
    pub awaiting_continue: bool,
    pub assisted_rounds: u32,
    pub questions: Vec<MarketQuestion>,
    pub step: usize,
    pub mistakes: u32,
    pub rejected: Vec<usize>,
}

impl Default for MarketState {
    fn default() -> Self {
        Self::new(MarketDifficulty::Easy, MarketSupport::Guided, 0)
    }
}

impl MarketState {
    pub fn new(difficulty: MarketDifficulty, support: MarketSupport, seed: u64) -> Self {
        Self {
            difficulty,
            support,
            help_revealed: false,
            awaiting_continue: false,
            assisted_rounds: 0,
            questions: generate_questions(difficulty, seed, ROUND_COUNT),
            step: 0,
            mistakes: 0,
            rejected: Vec::new(),
        }
    }

    pub fn current(&self) -> Option<&MarketQuestion> {
        self.questions.get(self.step)
    }

    pub fn complete(&self) -> bool {
        self.step == self.questions.len()
    }

    pub fn apply(&mut self, action: MarketAction) {
        match action {
            MarketAction::NewGame(difficulty, support, seed) => {
                *self = Self::new(difficulty, support, seed);
            }
            MarketAction::Continue => {
                if self.awaiting_continue {
                    self.step += 1;
                    self.rejected.clear();
                    self.help_revealed = false;
                    self.awaiting_continue = false;
                }
            }
            MarketAction::RevealHelp => {
                if !self.complete()
                    && !self.awaiting_continue
                    && self.support != MarketSupport::Guided
                {
                    self.help_revealed = true;
                }
            }
            MarketAction::Answer(choice) => {
                let Some(question) = self.current() else {
                    return;
                };
                if self.awaiting_continue
                    || choice >= question.choices.len()
                    || self.rejected.contains(&choice)
                {
                    return;
                }
                if choice == question.answer {
                    if self.help_revealed && self.support != MarketSupport::Guided {
                        self.assisted_rounds = self.assisted_rounds.saturating_add(1);
                    }
                    self.awaiting_continue = true;
                } else {
                    self.mistakes = self.mistakes.saturating_add(1);
                    self.rejected.push(choice);
                }
            }
        }
    }
}

struct SeededGenerator(u64);

impl SeededGenerator {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }

    fn below(&mut self, limit: u32) -> u32 {
        (self.next() % u64::from(limit)) as u32
    }
}

fn random_amount(difficulty: MarketDifficulty, rng: &mut SeededGenerator, sub_yuan: bool) -> u32 {
    let yuan = if difficulty != MarketDifficulty::Easy && sub_yuan {
        0
    } else {
        1 + rng.below(99)
    };
    match difficulty {
        MarketDifficulty::Easy => yuan * 100,
        MarketDifficulty::Medium => yuan * 100 + (1 + rng.below(9)) * 10,
        MarketDifficulty::Hard => yuan * 100 + 1 + rng.below(99),
    }
}

fn chinese_number(value: u32) -> String {
    const DIGITS: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    if value == 2 {
        return "两".to_owned();
    }
    if value < 10 {
        return DIGITS[value as usize].to_owned();
    }
    let tens = value / 10;
    let ones = value % 10;
    let prefix = if tens == 1 {
        String::new()
    } else {
        DIGITS[tens as usize].to_owned()
    };
    if ones == 0 {
        format!("{prefix}十")
    } else {
        format!("{prefix}十{}", DIGITS[ones as usize])
    }
}

fn pinyin_number(value: u32) -> String {
    const DIGITS: [&str; 10] = [
        "líng", "yī", "èr", "sān", "sì", "wǔ", "liù", "qī", "bā", "jiǔ",
    ];
    if value == 2 {
        return "liǎng".to_owned();
    }
    if value < 10 {
        return DIGITS[value as usize].to_owned();
    }
    let tens = value / 10;
    let ones = value % 10;
    let prefix = if tens == 1 {
        "shí".to_owned()
    } else {
        format!("{} shí", DIGITS[tens as usize])
    };
    if ones == 0 {
        prefix
    } else {
        format!("{prefix} {}", DIGITS[ones as usize])
    }
}

pub fn pinyin_price(amount_fen: u32) -> String {
    const DIGITS: [&str; 10] = [
        "líng", "yī", "èr", "sān", "sì", "wǔ", "liù", "qī", "bā", "jiǔ",
    ];
    let yuan = amount_fen / 100;
    let jiao = (amount_fen / 10) % 10;
    let fen = amount_fen % 10;
    let mut parts = Vec::with_capacity(4);
    if yuan > 0 {
        let yuan_reading = pinyin_number(yuan);
        let yuan_reading = if yuan % 10 == 1 {
            format!(
                "{}yí",
                yuan_reading.strip_suffix("yī").unwrap_or(&yuan_reading)
            )
        } else {
            yuan_reading
        };
        parts.push(format!("{yuan_reading} kuài"));
    }
    if jiao > 0 {
        let reading = match jiao {
            1 => "yì",
            2 => "liǎng",
            _ => DIGITS[jiao as usize],
        };
        parts.push(format!("{reading} máo"));
    } else if yuan > 0 && fen > 0 {
        parts.push("líng".to_owned());
    }
    if fen > 0 {
        let reading = match fen {
            1 => "yì",
            2 => "liǎng",
            _ => DIGITS[fen as usize],
        };
        parts.push(format!("{reading} fēn"));
    }
    parts.join(" ")
}

pub fn spoken_price(amount_fen: u32) -> String {
    const DIGITS: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    let yuan = amount_fen / 100;
    let jiao = (amount_fen / 10) % 10;
    let fen = amount_fen % 10;
    let mut result = String::new();
    if yuan > 0 {
        result.push_str(&chinese_number(yuan));
        result.push('块');
    }
    if jiao > 0 {
        result.push_str(if jiao == 2 {
            "两"
        } else {
            DIGITS[jiao as usize]
        });
        result.push('毛');
    } else if yuan > 0 && fen > 0 {
        result.push('零');
    }
    if fen > 0 {
        result.push_str(if fen == 2 {
            "两"
        } else {
            DIGITS[fen as usize]
        });
        result.push('分');
    }
    result
}

fn formatted_amount(amount_fen: u32) -> String {
    format!("¥{}.{:02}", amount_fen / 100, amount_fen % 100)
}

pub fn generate_questions(
    difficulty: MarketDifficulty,
    seed: u64,
    count: usize,
) -> Vec<MarketQuestion> {
    let mut rng = SeededGenerator(seed);
    let maximum = match difficulty {
        MarketDifficulty::Easy => 99,
        MarketDifficulty::Medium => 100 * 9,
        MarketDifficulty::Hard => 100 * 99,
    };
    let count = count.min(maximum);
    let sub_yuan_round = if count > 0 && difficulty != MarketDifficulty::Easy {
        Some(rng.below(count as u32) as usize)
    } else {
        None
    };
    let mut questions = Vec::with_capacity(count);
    let mut used_prices = HashSet::new();
    for round in 0..count {
        let amount = loop {
            let sub_yuan = Some(round) == sub_yuan_round
                || (difficulty != MarketDifficulty::Easy && rng.below(4) == 0);
            let candidate = random_amount(difficulty, &mut rng, sub_yuan);
            if used_prices.insert(candidate) {
                break candidate;
            }
        };
        let mut alternatives = vec![amount];
        while alternatives.len() < 3 {
            let candidate = random_amount(difficulty, &mut rng, amount < 100);
            if !alternatives.contains(&candidate) {
                alternatives.push(candidate);
            }
        }
        for i in (1..3).rev() {
            let j = rng.below((i + 1) as u32) as usize;
            alternatives.swap(i, j);
        }
        let answer = alternatives
            .iter()
            .position(|candidate| *candidate == amount)
            .unwrap_or(0);
        questions.push(MarketQuestion {
            prompt: spoken_price(amount),
            pinyin: pinyin_price(amount),
            choices: std::array::from_fn(|index| formatted_amount(alternatives[index])),
            answer,
            amount_fen: amount,
        });
    }
    questions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_seeded_and_varies_between_seeds() {
        let a = generate_questions(MarketDifficulty::Hard, 42, ROUND_COUNT);
        assert_eq!(
            a,
            generate_questions(MarketDifficulty::Hard, 42, ROUND_COUNT)
        );
        assert_ne!(
            a,
            generate_questions(MarketDifficulty::Hard, 43, ROUND_COUNT)
        );
        assert_eq!(a.len(), ROUND_COUNT);
        assert_eq!(
            a.iter().map(|q| q.amount_fen).collect::<HashSet<_>>().len(),
            ROUND_COUNT
        );
    }

    #[test]
    fn difficulties_limit_precision_and_distractors_are_valid() {
        for difficulty in [
            MarketDifficulty::Easy,
            MarketDifficulty::Medium,
            MarketDifficulty::Hard,
        ] {
            for question in generate_questions(difficulty, 23, ROUND_COUNT) {
                assert!(question.answer < 3);
                assert_eq!(
                    question.choices[question.answer],
                    formatted_amount(question.amount_fen)
                );
                assert_eq!(question.choices.iter().collect::<HashSet<_>>().len(), 3);
                assert!(
                    question
                        .choices
                        .iter()
                        .all(|choice| choice.starts_with('¥'))
                );
                assert!(
                    question
                        .choices
                        .iter()
                        .all(|choice| { choice.split('.').nth(1).map(str::len) == Some(2) })
                );
                match difficulty {
                    MarketDifficulty::Easy => assert_eq!(question.amount_fen % 100, 0),
                    MarketDifficulty::Medium => {
                        assert_eq!(question.amount_fen % 10, 0);
                        assert_ne!(question.amount_fen % 100, 0);
                    }
                    MarketDifficulty::Hard => assert_ne!(question.amount_fen % 100, 0),
                }
            }
        }
    }

    #[test]
    fn mandarin_prices_include_fen_and_zero_jiao() {
        assert_eq!(spoken_price(2500), "二十五块");
        assert_eq!(spoken_price(2530), "二十五块三毛");
        assert_eq!(spoken_price(2534), "二十五块三毛四分");
        assert_eq!(spoken_price(2504), "二十五块零四分");
        assert_eq!(spoken_price(1011), "十块一毛一分");
        assert_eq!(spoken_price(222), "两块两毛两分");
    }

    #[test]
    fn sub_yuan_pronunciations_do_not_include_zero_yuan() {
        for (amount, hanzi, pinyin) in [
            (1, "一分", "yì fēn"),
            (2, "两分", "liǎng fēn"),
            (9, "九分", "jiǔ fēn"),
            (10, "一毛", "yì máo"),
            (20, "两毛", "liǎng máo"),
            (25, "两毛五分", "liǎng máo wǔ fēn"),
            (50, "五毛", "wǔ máo"),
            (75, "七毛五分", "qī máo wǔ fēn"),
            (99, "九毛九分", "jiǔ máo jiǔ fēn"),
        ] {
            assert_eq!(spoken_price(amount), hanzi);
            assert_eq!(pinyin_price(amount), pinyin);
            assert_eq!(formatted_amount(amount), format!("¥0.{amount:02}"));
        }
    }

    #[test]
    fn each_decimal_difficulty_includes_sub_yuan_prices() {
        for difficulty in [MarketDifficulty::Medium, MarketDifficulty::Hard] {
            for seed in 0..32 {
                let questions = generate_questions(difficulty, seed, ROUND_COUNT);
                assert!(questions.iter().any(|question| question.amount_fen < 100));
                assert!(questions.iter().all(|question| question.amount_fen > 0));
                for question in &questions {
                    assert_eq!(
                        question.choices[question.answer],
                        formatted_amount(question.amount_fen)
                    );
                    assert_eq!(question.prompt, spoken_price(question.amount_fen));
                    assert_eq!(question.pinyin, pinyin_price(question.amount_fen));
                }
            }
        }
    }

    #[test]
    fn every_sub_yuan_cent_amount_has_a_reading() {
        for amount in 1..100 {
            assert!(!spoken_price(amount).contains('块'));
            assert!(!pinyin_price(amount).contains("kuài"));
            assert!(!spoken_price(amount).starts_with('零'));
            assert!(!pinyin_price(amount).starts_with("líng"));
        }
    }

    #[test]
    fn rejected_choices_are_counted_once_and_new_game_resets() {
        let mut state = MarketState::new(MarketDifficulty::Hard, MarketSupport::Reading, 7);
        let correct = state.current().unwrap().answer;
        let wrong = (correct + 1) % 3;
        state.apply(MarketAction::Answer(wrong));
        state.apply(MarketAction::Answer(wrong));
        assert_eq!(state.mistakes, 1);
        state.apply(MarketAction::Answer(correct));
        assert!(state.awaiting_continue);
        assert_eq!(state.step, 0);
        state.apply(MarketAction::Answer(correct));
        assert_eq!(state.step, 0);
        state.apply(MarketAction::Continue);
        assert_eq!(state.step, 1);
        assert!(state.rejected.is_empty());
        state.apply(MarketAction::NewGame(
            MarketDifficulty::Easy,
            MarketSupport::Guided,
            8,
        ));
        assert_eq!(state.step, 0);
        assert_eq!(state.mistakes, 0);
        assert_eq!(state.difficulty, MarketDifficulty::Easy);
    }

    #[test]
    fn pinyin_matches_spoken_price_units() {
        assert_eq!(pinyin_price(2500), "èr shí wǔ kuài");
        assert_eq!(pinyin_price(2530), "èr shí wǔ kuài sān máo");
        assert_eq!(pinyin_price(2534), "èr shí wǔ kuài sān máo sì fēn");
        assert_eq!(pinyin_price(2504), "èr shí wǔ kuài líng sì fēn");
        assert_eq!(pinyin_price(1011), "shí kuài yì máo yì fēn");
        assert_eq!(pinyin_price(100), "yí kuài");
        assert_eq!(pinyin_price(2100), "èr shí yí kuài");
        assert_eq!(pinyin_price(222), "liǎng kuài liǎng máo liǎng fēn");
    }

    #[test]
    fn support_visibility_and_reveal_are_per_round() {
        assert!(MarketSupport::Guided.show_hanzi(false));
        assert!(MarketSupport::Guided.show_pinyin(false));
        assert!(MarketSupport::Reading.show_hanzi(false));
        assert!(!MarketSupport::Reading.show_pinyin(false));
        assert!(!MarketSupport::Listening.show_hanzi(false));
        assert!(!MarketSupport::Listening.show_pinyin(false));
        assert!(MarketSupport::Listening.show_pinyin(true));
        let mut state = MarketState::new(MarketDifficulty::Hard, MarketSupport::Listening, 5);
        state.apply(MarketAction::RevealHelp);
        assert!(state.help_revealed);
        let correct = state.current().unwrap().answer;
        state.apply(MarketAction::Answer(correct));
        assert_eq!(state.assisted_rounds, 1);
        assert!(state.awaiting_continue);
        state.apply(MarketAction::Answer(correct));
        assert_eq!(state.assisted_rounds, 1);
        state.apply(MarketAction::Continue);
        assert!(!state.help_revealed);
        state.apply(MarketAction::Answer(state.current().unwrap().answer));
        assert_eq!(state.assisted_rounds, 1);
        state.apply(MarketAction::NewGame(
            MarketDifficulty::Easy,
            MarketSupport::Reading,
            6,
        ));
        assert_eq!(state.assisted_rounds, 0);
        assert_eq!(state.support, MarketSupport::Reading);
    }
}
