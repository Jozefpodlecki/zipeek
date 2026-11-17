#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadicalQuestion {
    pub character: &'static str,
    pub meaning: &'static str,
    pub parts: [&'static str; 2],
    pub choices: [&'static str; 3],
    pub answer: usize,
}

pub const QUESTIONS: [RadicalQuestion; 5] = [
    RadicalQuestion {
        character: "明",
        meaning: "bright",
        parts: ["日", "月"],
        choices: ["日 + 月", "木 + 木", "人 + 木"],
        answer: 0,
    },
    RadicalQuestion {
        character: "林",
        meaning: "woods",
        parts: ["木", "木"],
        choices: ["日 + 月", "口 + 木", "木 + 木"],
        answer: 2,
    },
    RadicalQuestion {
        character: "休",
        meaning: "rest",
        parts: ["亻", "木"],
        choices: ["亻 + 木", "氵 + 木", "日 + 月"],
        answer: 0,
    },
    RadicalQuestion {
        character: "好",
        meaning: "good",
        parts: ["女", "子"],
        choices: ["口 + 子", "女 + 子", "女 + 木"],
        answer: 1,
    },
    RadicalQuestion {
        character: "从",
        meaning: "follow",
        parts: ["人", "人"],
        choices: ["人 + 人", "人 + 木", "人 + 口"],
        answer: 0,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadicalAction {
    Answer(usize),
    Restart,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RadicalState {
    pub step: usize,
    pub mistakes: u32,
    pub rejected: Option<usize>,
}

impl RadicalState {
    pub fn current(&self) -> Option<&'static RadicalQuestion> {
        QUESTIONS.get(self.step)
    }
    pub fn complete(&self) -> bool {
        self.step == QUESTIONS.len()
    }
    pub fn apply(&mut self, action: RadicalAction) {
        match action {
            RadicalAction::Restart => *self = Self::default(),
            RadicalAction::Answer(choice) => {
                let Some(question) = self.current() else {
                    return;
                };
                if choice >= question.choices.len() || self.rejected == Some(choice) {
                    return;
                }
                if choice == question.answer {
                    self.step += 1;
                    self.rejected = None;
                } else {
                    self.mistakes = self.mistakes.saturating_add(1);
                    self.rejected = Some(choice);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn forged_components_are_authored_and_validated() {
        for question in QUESTIONS {
            assert_eq!(
                question.choices[question.answer],
                format!("{} + {}", question.parts[0], question.parts[1])
            );
        }
    }
    #[test]
    fn invalid_repeated_answer_does_not_score() {
        let mut game = RadicalState::default();
        game.apply(RadicalAction::Answer(usize::MAX));
        game.apply(RadicalAction::Answer(2));
        game.apply(RadicalAction::Answer(2));
        assert_eq!(game.mistakes, 1);
        game.apply(RadicalAction::Restart);
        assert_eq!(game, RadicalState::default());
    }
}
