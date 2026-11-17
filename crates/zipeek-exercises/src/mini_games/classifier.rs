use super::validated_rounds;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassifierQuestion {
    pub noun: &'static str,
    pub meaning: &'static str,
    pub choices: [&'static str; 3],
    pub answer: usize,
}

pub const CLASSIFIER_QUESTIONS: [ClassifierQuestion; 6] = [
    ClassifierQuestion {
        noun: "书",
        meaning: "book",
        choices: ["本", "只", "张"],
        answer: 0,
    },
    ClassifierQuestion {
        noun: "猫",
        meaning: "cat",
        choices: ["杯", "只", "条"],
        answer: 1,
    },
    ClassifierQuestion {
        noun: "水",
        meaning: "glass of water",
        choices: ["本", "张", "杯"],
        answer: 2,
    },
    ClassifierQuestion {
        noun: "纸",
        meaning: "sheet of paper",
        choices: ["张", "个", "只"],
        answer: 0,
    },
    ClassifierQuestion {
        noun: "鱼",
        meaning: "fish",
        choices: ["杯", "条", "本"],
        answer: 1,
    },
    ClassifierQuestion {
        noun: "人",
        meaning: "person",
        choices: ["张", "条", "个"],
        answer: 2,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassifierAction {
    Answer(usize),
    Restart,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassifierState {
    pub step: usize,
    pub rounds: Vec<usize>,
    pub mistakes: u32,
    pub notice: Option<&'static str>,
    pub last_wrong_choice: Option<usize>,
}

impl Default for ClassifierState {
    fn default() -> Self {
        Self::from_rounds((0..CLASSIFIER_QUESTIONS.len()).collect())
    }
}

impl ClassifierState {
    pub fn from_rounds(rounds: Vec<usize>) -> Self {
        Self {
            step: 0,
            rounds: validated_rounds(rounds, CLASSIFIER_QUESTIONS.len()),
            mistakes: 0,
            notice: None,
            last_wrong_choice: None,
        }
    }

    pub fn total(&self) -> usize {
        self.rounds.len()
    }

    pub fn current(&self) -> Option<&'static ClassifierQuestion> {
        self.rounds
            .get(self.step)
            .and_then(|&index| CLASSIFIER_QUESTIONS.get(index))
    }

    pub fn complete(&self) -> bool {
        self.step >= self.total()
    }

    pub fn apply(&mut self, action: ClassifierAction) {
        match action {
            ClassifierAction::Restart => *self = Self::from_rounds(self.rounds.clone()),
            ClassifierAction::Answer(index) => {
                let Some(question) = self.current() else {
                    return;
                };
                if index >= question.choices.len() {
                    return;
                }
                if index == question.answer {
                    self.step += 1;
                    self.last_wrong_choice = None;
                    self.notice = Some("Correct");
                } else if self.last_wrong_choice != Some(index) {
                    self.last_wrong_choice = Some(index);
                    self.mistakes = self.mistakes.saturating_add(1);
                    self.notice = Some("Try another measure word");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incorrect_and_invalid_answers_do_not_advance() {
        let mut state = ClassifierState::default();
        state.apply(ClassifierAction::Answer(usize::MAX));
        assert_eq!(state, ClassifierState::default());
        state.apply(ClassifierAction::Answer(2));
        assert_eq!((state.step, state.mistakes), (0, 1));
        assert_eq!(state.notice, Some("Try another measure word"));
    }

    #[test]
    fn repeated_identical_wrong_choice_is_only_one_attempt() {
        let mut state = ClassifierState::default();
        state.apply(ClassifierAction::Answer(2));
        state.apply(ClassifierAction::Answer(2));
        assert_eq!(state.mistakes, 1);
        state.apply(ClassifierAction::Answer(1));
        assert_eq!(state.mistakes, 2);
        state.apply(ClassifierAction::Answer(0));
        assert_eq!(state.step, 1);
        assert_eq!(state.last_wrong_choice, None);
    }

    #[test]
    fn completion_is_terminal_until_restart() {
        let mut state = ClassifierState::default();
        for question in CLASSIFIER_QUESTIONS {
            state.apply(ClassifierAction::Answer(question.answer));
        }
        assert!(state.complete());
        let completed = state.clone();
        state.apply(ClassifierAction::Answer(0));
        assert_eq!(state, completed);
        state.apply(ClassifierAction::Restart);
        assert_eq!(state, ClassifierState::default());
    }

    #[test]
    fn all_questions_have_valid_unique_answers() {
        for question in CLASSIFIER_QUESTIONS {
            assert!(!question.noun.is_empty());
            assert!(!question.meaning.is_empty());
            assert!(question.answer < question.choices.len());
            let correct = question.choices[question.answer];
            assert_eq!(
                question
                    .choices
                    .iter()
                    .filter(|&&item| item == correct)
                    .count(),
                1
            );
        }
    }
}
