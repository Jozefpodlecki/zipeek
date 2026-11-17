use super::{CharacterHunt, FoundWord, HuntError, TapResult};

impl CharacterHunt {
    pub fn completed_words(&self) -> Vec<FoundWord> {
        self.found
            .iter()
            .filter_map(|word| {
                let answer = self.answers.iter().find(|answer| &answer.word == word)?;
                let syllables = answer
                    .indices
                    .iter()
                    .map(|index| self.tiles[*index].pinyin.as_deref())
                    .collect::<Option<Vec<_>>>();
                Some(FoundWord {
                    hanzi: answer.word.clone(),
                    pinyin: syllables.map(|parts| parts.join(" ")),
                    meaning: answer.meaning.clone(),
                })
            })
            .collect()
    }

    pub fn total_words(&self) -> usize {
        self.answers.len()
    }

    pub fn complete(&self) -> bool {
        !self.answers.is_empty() && self.found.len() == self.answers.len()
    }

    pub fn hint(&self) -> Option<(char, usize)> {
        self.answers
            .iter()
            .find(|answer| !self.found.contains(&answer.word))
            .map(|answer| {
                (
                    answer.word.chars().next().expect("answer is nonempty"),
                    answer.word.chars().count(),
                )
            })
    }

    pub fn hint_indices(&self) -> Option<Vec<usize>> {
        self.answers
            .iter()
            .find(|answer| !self.found.contains(&answer.word))
            .map(|answer| answer.indices.clone())
    }

    pub fn reveal_word(&mut self) -> Option<String> {
        let answer = self
            .answers
            .iter()
            .find(|answer| !self.found.contains(&answer.word))?
            .clone();
        for index in &answer.indices {
            self.tiles[*index].completed = true;
        }
        self.found.push(answer.word.clone());
        self.selected.clear();
        Some(answer.word)
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn tap(&mut self, index: usize) -> Result<TapResult, HuntError> {
        let tile = self.tiles.get(index).ok_or(HuntError::InvalidTile)?;
        if tile.completed || tile.filler || self.complete() {
            return Ok(TapResult::Ignored);
        }
        if self.selected.contains(&index) {
            self.selected.retain(|selected| *selected != index);
            return Ok(TapResult::Selected);
        }
        self.selected.push(index);
        let selected_word: String = self
            .selected
            .iter()
            .map(|selected| self.tiles[*selected].character)
            .collect();
        let found = self.answers.iter().find(|answer| {
            answer.word == selected_word
                && answer.indices.len() == self.selected.len()
                && answer.indices.iter().all(|idx| self.selected.contains(idx))
        });
        if let Some(answer) = found {
            self.found.push(answer.word.clone());
            for selected in &self.selected {
                self.tiles[*selected].completed = true;
            }
            self.selected.clear();
            return Ok(TapResult::Found);
        }
        if self.selected.len() == 1 {
            return Ok(TapResult::Selected);
        }
        let possible = self.answers.iter().any(|answer| {
            !self.found.contains(&answer.word)
                && answer.word.starts_with(&selected_word)
                && self.selected.iter().all(|idx| answer.indices.contains(idx))
        });
        if !possible || self.selected.len() == 3 {
            self.selected.clear();
            self.mistakes = self.mistakes.saturating_add(1);
            return Ok(TapResult::Incorrect);
        }
        Ok(TapResult::Selected)
    }
}
