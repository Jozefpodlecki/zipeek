use super::{AnswerMode, Difficulty, LearningSupport, deck::Question};

pub const ROUND_LIMIT: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    Answering,
    Reviewing { selected_id: String, correct: bool },
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    NewGame(Session),
    SelectMap(String),
    SelectHanzi(String),
    RevealHint,
    Continue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub difficulty: Difficulty,
    pub support: LearningSupport,
    pub questions: Vec<Question>,
    pub round: usize,
    pub mistakes: usize,
    pub assisted_answers: usize,
    pub hint_revealed: bool,
    pub phase: Phase,
}

impl Session {
    pub fn new(difficulty: Difficulty, support: LearningSupport, questions: Vec<Question>) -> Self {
        let phase = if questions.is_empty() {
            Phase::Complete
        } else {
            Phase::Answering
        };
        Self {
            difficulty,
            support,
            questions,
            round: 0,
            mistakes: 0,
            assisted_answers: 0,
            hint_revealed: false,
            phase,
        }
    }

    pub fn current(&self) -> Option<&Question> {
        if matches!(self.phase, Phase::Complete) {
            None
        } else {
            self.questions.get(self.round)
        }
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::NewGame(new_game) => *self = new_game,
            Action::RevealHint if matches!(self.phase, Phase::Answering) => {
                self.hint_revealed = true
            }
            Action::SelectMap(id) if matches!(self.phase, Phase::Answering) => {
                self.submit(id, AnswerMode::SelectMap)
            }
            Action::SelectHanzi(id) if matches!(self.phase, Phase::Answering) => {
                self.submit(id, AnswerMode::SelectHanzi)
            }
            Action::Continue if matches!(self.phase, Phase::Reviewing { .. }) => {
                self.round += 1;
                self.hint_revealed = false;
                self.phase = if self.round >= self.questions.len() {
                    Phase::Complete
                } else {
                    Phase::Answering
                };
            }
            _ => {}
        }
    }

    fn submit(&mut self, id: String, mode: AnswerMode) {
        let Some(question) = self.current() else {
            return;
        };
        if question.answer_mode != mode {
            return;
        }
        if mode == AnswerMode::SelectHanzi && !question.options.contains(&id) {
            return;
        }
        let correct = question.region_id == id;
        if !correct {
            self.mistakes += 1;
        }
        if self.hint_revealed && correct {
            self.assisted_answers += 1;
        }
        self.phase = Phase::Reviewing {
            selected_id: id,
            correct,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::province_quest::QuestionKind;

    fn game(mode: AnswerMode) -> Session {
        Session::new(
            Difficulty::Easy,
            LearningSupport::Guided,
            vec![Question {
                region_id: "53".into(),
                kind: QuestionKind::FindName,
                parent_id: None,
                answer_mode: mode,
                options: if mode == AnswerMode::SelectHanzi {
                    vec!["53".into(), "64".into(), "11".into()]
                } else {
                    Vec::new()
                },
            }],
        )
    }

    #[test]
    fn wrong_answer_counts_once() {
        let mut game = game(AnswerMode::SelectMap);
        game.apply(Action::SelectMap("64".into()));
        game.apply(Action::SelectMap("64".into()));
        assert_eq!(game.mistakes, 1);
        assert!(matches!(
            game.phase,
            Phase::Reviewing { correct: false, .. }
        ));
    }

    #[test]
    fn answers_cannot_cross_question_formats() {
        let mut game = game(AnswerMode::SelectHanzi);
        game.apply(Action::SelectMap("53".into()));
        game.apply(Action::SelectHanzi("invalid".into()));
        assert_eq!(game.phase, Phase::Answering);
        game.apply(Action::SelectHanzi("64".into()));
        assert_eq!(game.mistakes, 1);
    }

    #[test]
    fn hint_is_scored_and_reset_on_advance() {
        let mut game = game(AnswerMode::SelectHanzi);
        game.apply(Action::RevealHint);
        game.apply(Action::SelectHanzi("53".into()));
        assert_eq!(game.assisted_answers, 1);
        game.apply(Action::Continue);
        assert!(!game.hint_revealed);
        assert_eq!(game.phase, Phase::Complete);
    }
}
