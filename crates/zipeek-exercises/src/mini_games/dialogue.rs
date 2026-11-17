#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueRound {
    pub context: String,
    pub speaker: String,
    pub prompt: String,
    pub choices: [String; 3],
    pub answer: usize,
    pub explanation: String,
}

impl DialogueRound {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.context.trim().is_empty()
            || self.context.chars().count() > 240
            || self.speaker.trim().is_empty()
            || self.prompt.trim().is_empty()
            || self.prompt.chars().count() > 160
            || self.explanation.trim().is_empty()
            || self.explanation.chars().count() > 240
        {
            return Err("Invalid dialogue context");
        }
        if self.answer >= self.choices.len()
            || self
                .choices
                .iter()
                .any(|choice| choice.trim().is_empty() || choice.chars().count() > 160)
        {
            return Err("Invalid dialogue responses");
        }
        if self.choices[0] == self.choices[1]
            || self.choices[0] == self.choices[2]
            || self.choices[1] == self.choices[2]
        {
            return Err("Dialogue answers must be distinct");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogueAction {
    Answer(usize),
    Restart,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueState {
    pub rounds: Vec<DialogueRound>,
    pub step: usize,
    pub mistakes: u32,
    pub rejected: Option<usize>,
}

impl DialogueState {
    pub fn new(rounds: Vec<DialogueRound>) -> Result<Self, &'static str> {
        if rounds.is_empty() || rounds.len() > 10 {
            return Err("Expected 1 to 10 dialogue rounds");
        }
        for round in &rounds {
            round.validate()?
        }
        Ok(Self {
            rounds,
            step: 0,
            mistakes: 0,
            rejected: None,
        })
    }
    pub fn current(&self) -> Option<&DialogueRound> {
        self.rounds.get(self.step)
    }
    pub fn complete(&self) -> bool {
        self.step == self.rounds.len()
    }
    pub fn apply(&mut self, action: DialogueAction) {
        match action {
            DialogueAction::Restart => {
                self.step = 0;
                self.mistakes = 0;
                self.rejected = None;
            }
            DialogueAction::Answer(choice) => {
                let Some(round) = self.current() else { return };
                if choice >= 3 || self.rejected == Some(choice) {
                    return;
                }
                if choice == round.answer {
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
    fn sample() -> DialogueRound {
        DialogueRound {
            context: "At a cafe".into(),
            speaker: "Server".into(),
            prompt: "你喝什么？".into(),
            choices: ["我要茶。".into(), "我不去。".into(), "明天见。".into()],
            answer: 0,
            explanation: "The server asks for a drink.".into(),
        }
    }
    #[test]
    fn dialogue_rejects_duplicate_choices_and_scores_only_once() {
        let mut invalid = sample();
        invalid.choices[1] = invalid.choices[0].clone();
        assert!(invalid.validate().is_err());
        let mut game = DialogueState::new(vec![sample()]).unwrap();
        game.apply(DialogueAction::Answer(1));
        game.apply(DialogueAction::Answer(1));
        assert_eq!(game.mistakes, 1);
        game.apply(DialogueAction::Answer(0));
        assert!(game.complete());
    }
}
