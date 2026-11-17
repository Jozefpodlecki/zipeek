use super::validated_rounds;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SentenceQuestion {
    pub english: &'static str,
    pub tokens: [&'static str; 5],
    pub count: usize,
    pub order: [usize; 5],
}

pub const SENTENCE_QUESTIONS: [SentenceQuestion; 4] = [
    SentenceQuestion {
        english: "I like drinking tea.",
        tokens: ["茶", "我", "喝", "喜欢", ""],
        count: 4,
        order: [1, 3, 2, 0, 0],
    },
    SentenceQuestion {
        english: "He is studying Chinese.",
        tokens: ["中文", "在", "他", "学习", ""],
        count: 4,
        order: [2, 1, 3, 0, 0],
    },
    SentenceQuestion {
        english: "We will go to school tomorrow.",
        tokens: ["学校", "明天", "去", "我们", ""],
        count: 4,
        order: [3, 1, 2, 0, 0],
    },
    SentenceQuestion {
        english: "Do you have time today?",
        tokens: ["时间", "你", "今天", "有", "吗"],
        count: 5,
        order: [1, 2, 3, 0, 4],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SentenceAction {
    Select(usize),
    Remove(usize),
    Undo,
    Clear,
    Check,
    Restart,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SentenceState {
    pub step: usize,
    pub rounds: Vec<usize>,
    pub mistakes: u32,
    pub selected: Vec<usize>,
    pub notice: Option<&'static str>,
}

impl Default for SentenceState {
    fn default() -> Self {
        Self::from_rounds((0..SENTENCE_QUESTIONS.len()).collect())
    }
}

impl SentenceState {
    pub fn from_rounds(rounds: Vec<usize>) -> Self {
        Self {
            step: 0,
            rounds: validated_rounds(rounds, SENTENCE_QUESTIONS.len()),
            mistakes: 0,
            selected: Vec::new(),
            notice: None,
        }
    }

    pub fn total(&self) -> usize {
        self.rounds.len()
    }

    pub fn current(&self) -> Option<&'static SentenceQuestion> {
        self.rounds
            .get(self.step)
            .and_then(|&index| SENTENCE_QUESTIONS.get(index))
    }

    pub fn complete(&self) -> bool {
        self.step >= self.total()
    }

    pub fn apply(&mut self, action: SentenceAction) {
        if action == SentenceAction::Restart {
            *self = Self::from_rounds(self.rounds.clone());
            return;
        }
        let Some(question) = self.current() else {
            return;
        };
        match action {
            SentenceAction::Select(token) => {
                if token < question.count && !self.selected.contains(&token) {
                    self.selected.push(token);
                    self.notice = None;
                }
            }
            SentenceAction::Remove(token) => {
                if let Some(position) = self.selected.iter().position(|&item| item == token) {
                    self.selected.remove(position);
                    self.notice = None;
                }
            }
            SentenceAction::Undo => {
                if self.selected.pop().is_some() {
                    self.notice = None;
                }
            }
            SentenceAction::Clear => {
                self.selected.clear();
                self.notice = None;
            }
            SentenceAction::Check => {
                if self.selected.len() != question.count {
                    return;
                }
                if self.notice == Some("Tap any selected word to change its position") {
                    return;
                }
                if self.selected == question.order[..question.count] {
                    self.step += 1;
                    self.selected.clear();
                    self.notice = Some("Correct sentence");
                } else {
                    self.mistakes = self.mistakes.saturating_add(1);
                    self.notice = Some("Tap any selected word to change its position");
                }
            }
            SentenceAction::Restart => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_unique_and_correction_clears_stale_notice() {
        let mut state = SentenceState::default();
        for token in [0, 0, 1, 2, 3] {
            state.apply(SentenceAction::Select(token));
        }
        assert_eq!(state.selected, [0, 1, 2, 3]);
        state.apply(SentenceAction::Check);
        assert_eq!((state.step, state.mistakes), (0, 1));
        state.apply(SentenceAction::Remove(1));
        assert_eq!(state.notice, None);
        state.apply(SentenceAction::Clear);
        for token in [1, 3, 2, 0] {
            state.apply(SentenceAction::Select(token));
        }
        state.apply(SentenceAction::Check);
        assert_eq!(state.step, 1);
        assert!(state.selected.is_empty());
    }

    #[test]
    fn repeated_check_of_unchanged_wrong_sentence_is_one_attempt() {
        let mut state = SentenceState::default();
        for token in [0, 1, 2, 3] {
            state.apply(SentenceAction::Select(token));
        }
        state.apply(SentenceAction::Check);
        state.apply(SentenceAction::Check);
        assert_eq!(state.mistakes, 1);
        state.apply(SentenceAction::Undo);
        state.apply(SentenceAction::Select(3));
        state.apply(SentenceAction::Check);
        assert_eq!(state.mistakes, 2);
    }

    #[test]
    fn completed_state_ignores_input_until_restart() {
        let mut state = SentenceState::default();
        for question in SENTENCE_QUESTIONS {
            for token in &question.order[..question.count] {
                state.apply(SentenceAction::Select(*token));
            }
            state.apply(SentenceAction::Check);
        }
        assert!(state.complete());
        let completed = state.clone();
        state.apply(SentenceAction::Select(0));
        state.apply(SentenceAction::Check);
        assert_eq!(state, completed);
        state.apply(SentenceAction::Restart);
        assert_eq!(state, SentenceState::default());
    }

    #[test]
    fn every_sentence_has_a_unique_permutation() {
        for question in SENTENCE_QUESTIONS {
            assert!((1..=question.tokens.len()).contains(&question.count));
            let mut seen = vec![false; question.count];
            for &index in &question.order[..question.count] {
                assert!(index < question.count);
                assert!(!seen[index]);
                assert!(!question.tokens[index].is_empty());
                seen[index] = true;
            }
            assert!(seen.into_iter().all(|used| used));
        }
    }
}
