#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedSentence {
    pub english: String,
    pub tokens: Vec<String>,
    pub order: Vec<usize>,
}

impl GeneratedSentence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.english.trim().is_empty() || self.english.chars().count() > 200 {
            return Err("Invalid English prompt");
        }
        if !(3..=8).contains(&self.tokens.len()) || self.order.len() != self.tokens.len() {
            return Err("Expected 3 to 8 sentence tokens");
        }
        if self
            .tokens
            .iter()
            .any(|word| word.trim().is_empty() || word.chars().count() > 12)
        {
            return Err("Invalid sentence token");
        }
        let mut seen = vec![false; self.tokens.len()];
        for &position in &self.order {
            if position >= seen.len() || seen[position] {
                return Err("Answer is not a permutation");
            }
            seen[position] = true;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeneratedSentenceAction {
    Select(usize),
    Remove(usize),
    Undo,
    Clear,
    Check,
    Restart,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedSentenceState {
    pub rounds: Vec<GeneratedSentence>,
    pub step: usize,
    pub mistakes: u32,
    pub selected: Vec<usize>,
    pub rejected: bool,
}

impl GeneratedSentenceState {
    pub fn new(rounds: Vec<GeneratedSentence>) -> Result<Self, &'static str> {
        if rounds.is_empty() || rounds.len() > 10 {
            return Err("Expected 1 to 10 AI sentence rounds");
        }
        for round in &rounds {
            round.validate()?;
        }
        Ok(Self {
            rounds,
            step: 0,
            mistakes: 0,
            selected: Vec::new(),
            rejected: false,
        })
    }
    pub fn current(&self) -> Option<&GeneratedSentence> {
        self.rounds.get(self.step)
    }
    pub fn complete(&self) -> bool {
        self.step == self.rounds.len()
    }
    pub fn apply(&mut self, action: GeneratedSentenceAction) {
        if action == GeneratedSentenceAction::Restart {
            self.step = 0;
            self.mistakes = 0;
            self.selected.clear();
            self.rejected = false;
            return;
        }
        let Some(round) = self.current() else { return };
        let count = round.tokens.len();
        let correct = round.order.clone();
        match action {
            GeneratedSentenceAction::Select(index)
                if index < count && !self.selected.contains(&index) =>
            {
                self.selected.push(index);
                self.rejected = false;
            }
            GeneratedSentenceAction::Remove(index) => {
                self.selected.retain(|item| *item != index);
                self.rejected = false;
            }
            GeneratedSentenceAction::Undo => {
                self.selected.pop();
                self.rejected = false;
            }
            GeneratedSentenceAction::Clear => {
                self.selected.clear();
                self.rejected = false;
            }
            GeneratedSentenceAction::Check if self.selected.len() == count && !self.rejected => {
                if self.selected == correct {
                    self.step += 1;
                    self.selected.clear();
                } else {
                    self.mistakes = self.mistakes.saturating_add(1);
                    self.rejected = true;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> GeneratedSentence {
        GeneratedSentence {
            english: "I drink tea".into(),
            tokens: vec!["茶".into(), "我".into(), "喝".into()],
            order: vec![1, 2, 0],
        }
    }
    #[test]
    fn generated_rounds_reject_invalid_model_answers() {
        let mut round = sample();
        round.order = vec![1, 1, 0];
        assert!(round.validate().is_err());
        round.order = vec![1, 2, 0];
        assert!(round.validate().is_ok());
        round.tokens.clear();
        assert!(round.validate().is_err());
    }
    #[test]
    fn generated_state_retries_without_duplicate_mistakes() {
        let mut state = GeneratedSentenceState::new(vec![sample()]).unwrap();
        for index in [0, 1, 2] {
            state.apply(GeneratedSentenceAction::Select(index));
        }
        state.apply(GeneratedSentenceAction::Check);
        state.apply(GeneratedSentenceAction::Check);
        assert_eq!(state.mistakes, 1);
        state.apply(GeneratedSentenceAction::Clear);
        for index in [1, 2, 0] {
            state.apply(GeneratedSentenceAction::Select(index));
        }
        state.apply(GeneratedSentenceAction::Check);
        assert!(state.complete());
    }
}
