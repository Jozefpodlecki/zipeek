#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TonePrompt {
    pub hanzi: &'static str,
    pub pinyin: &'static str,
    pub meaning: &'static str,
    pub tone: u8,
}

pub static PROMPTS: [TonePrompt; 12] = [
    TonePrompt {
        hanzi: "妈",
        pinyin: "mā",
        meaning: "mother",
        tone: 1,
    },
    TonePrompt {
        hanzi: "麻",
        pinyin: "má",
        meaning: "hemp",
        tone: 2,
    },
    TonePrompt {
        hanzi: "马",
        pinyin: "mǎ",
        meaning: "horse",
        tone: 3,
    },
    TonePrompt {
        hanzi: "骂",
        pinyin: "mà",
        meaning: "scold",
        tone: 4,
    },
    TonePrompt {
        hanzi: "书",
        pinyin: "shū",
        meaning: "book",
        tone: 1,
    },
    TonePrompt {
        hanzi: "十",
        pinyin: "shí",
        meaning: "ten",
        tone: 2,
    },
    TonePrompt {
        hanzi: "水",
        pinyin: "shuǐ",
        meaning: "water",
        tone: 3,
    },
    TonePrompt {
        hanzi: "是",
        pinyin: "shì",
        meaning: "to be",
        tone: 4,
    },
    TonePrompt {
        hanzi: "花",
        pinyin: "huā",
        meaning: "flower",
        tone: 1,
    },
    TonePrompt {
        hanzi: "茶",
        pinyin: "chá",
        meaning: "tea",
        tone: 2,
    },
    TonePrompt {
        hanzi: "火",
        pinyin: "huǒ",
        meaning: "fire",
        tone: 3,
    },
    TonePrompt {
        hanzi: "看",
        pinyin: "kàn",
        meaning: "look",
        tone: 4,
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToneGame {
    order: Vec<usize>,
    position: usize,
    answer: Option<u8>,
    correct: usize,
    misses: Vec<(usize, u8)>,
}

impl ToneGame {
    pub fn new(seed: u64) -> Self {
        let mut order: Vec<usize> = (0..PROMPTS.len()).collect();
        let mut state = seed | 1;
        for i in (1..order.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            order.swap(i, (state % (i as u64 + 1)) as usize);
        }
        Self {
            order,
            position: 0,
            answer: None,
            correct: 0,
            misses: Vec::new(),
        }
    }

    pub fn prompt(&self) -> Option<&'static TonePrompt> {
        self.order.get(self.position).map(|&index| &PROMPTS[index])
    }

    pub fn answer(&self) -> Option<u8> {
        self.answer
    }

    pub fn choose(&mut self, tone: u8) -> bool {
        let Some(prompt) = self.prompt() else {
            return false;
        };
        if !(1..=4).contains(&tone) || self.answer.is_some() {
            return false;
        }
        if tone == prompt.tone {
            self.correct += 1;
        } else {
            self.misses.push((self.order[self.position], tone));
        }
        self.answer = Some(tone);
        true
    }

    pub fn next(&mut self) -> bool {
        if self.answer.is_none() || self.prompt().is_none() {
            return false;
        }
        self.position += 1;
        self.answer = None;
        true
    }

    pub fn correct(&self) -> usize {
        self.correct
    }
    pub fn misses(&self) -> impl Iterator<Item = (&'static TonePrompt, u8)> + '_ {
        self.misses
            .iter()
            .map(|(index, selected)| (&PROMPTS[*index], *selected))
    }
    pub fn completed(&self) -> usize {
        self.position + usize::from(self.answer.is_some())
    }
    pub fn total(&self) -> usize {
        self.order.len()
    }
    pub fn finished(&self) -> bool {
        self.position >= self.order.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_prompts_have_all_four_tones() {
        for tone in 1..=4 {
            assert!(PROMPTS.iter().filter(|prompt| prompt.tone == tone).count() >= 2);
        }
        assert!(
            PROMPTS
                .iter()
                .all(|prompt| !prompt.hanzi.is_empty() && !prompt.pinyin.is_empty())
        );
    }

    #[test]
    fn a_question_scores_once_and_cannot_skip_unanswered() {
        let mut game = ToneGame::new(7);
        assert!(!game.next());
        assert!(!game.choose(0));
        let tone = game.prompt().unwrap().tone;
        assert!(game.choose(tone));
        assert!(!game.choose(tone));
        assert_eq!(game.correct(), 1);
        assert!(game.next());
        assert_eq!(game.completed(), 1);
        assert_eq!(game.answer(), None);
    }

    #[test]
    fn incorrect_guess_does_not_score_and_all_questions_finish() {
        let mut game = ToneGame::new(42);
        while let Some(prompt) = game.prompt() {
            let wrong = prompt.tone % 4 + 1;
            assert!(game.choose(wrong));
            assert!(game.next());
        }
        assert_eq!(game.correct(), 0);
        assert_eq!(game.misses().count(), PROMPTS.len());
        assert!(
            game.misses()
                .all(|(prompt, selected)| prompt.tone != selected)
        );
        assert_eq!(game.completed(), PROMPTS.len());
        assert!(game.finished());
        assert!(!game.choose(1));
        assert!(!game.next());
    }

    #[test]
    fn seed_orders_are_deterministic_without_repeating_prompts() {
        let mut first = ToneGame::new(3);
        let mut second = ToneGame::new(3);
        let mut seen = std::collections::HashSet::new();
        while let Some(prompt) = first.prompt() {
            assert_eq!(Some(prompt), second.prompt());
            assert!(seen.insert(prompt.hanzi));
            first.choose(1);
            second.choose(1);
            first.next();
            second.next();
        }
        assert_eq!(seen.len(), PROMPTS.len());
    }
}
