use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListeningWord {
    pub id: i32,
    pub hanzi: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListeningQuestion {
    pub choices: Vec<ListeningWord>,
    pub correct_id: i32,
    pub spoken_hanzi: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListeningError {
    NotEnoughWords,
    DuplicateIds,
    InvalidAnswer,
}

pub fn prepare(words: &[ListeningWord], seed: u64) -> Result<ListeningQuestion, ListeningError> {
    let mut eligible = words
        .iter()
        .filter(|word| {
            word.id > 0
                && !word.hanzi.trim().is_empty()
                && word.hanzi.chars().count() <= 36
                && !word.hanzi.chars().any(char::is_control)
        })
        .cloned()
        .collect::<Vec<_>>();
    for word in &mut eligible {
        word.hanzi = word.hanzi.trim().to_owned();
    }
    eligible.sort_by_key(|word| word.id);
    if eligible.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(ListeningError::DuplicateIds);
    }
    let mut readings = BTreeSet::new();
    eligible.retain(|word| readings.insert(word.hanzi.clone()));
    if eligible.len() < 3 {
        return Err(ListeningError::NotEnoughWords);
    }
    let mut rng = seed.max(1);
    for index in (1..eligible.len()).rev() {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        eligible.swap(index, (rng as usize) % (index + 1));
    }
    let mut choices = eligible.into_iter().take(3).collect::<Vec<_>>();
    let correct_id = choices[0].id;
    let spoken_hanzi = choices[0].hanzi.clone();
    choices.rotate_left(((seed >> 16) as usize) % 3);
    Ok(ListeningQuestion {
        choices,
        correct_id,
        spoken_hanzi,
    })
}

impl ListeningQuestion {
    pub fn grade(&self, selected_id: i32) -> Result<bool, ListeningError> {
        if !self.choices.iter().any(|choice| choice.id == selected_id) {
            return Err(ListeningError::InvalidAnswer);
        }
        Ok(selected_id == self.correct_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Vec<ListeningWord> {
        (1..=6)
            .map(|id| ListeningWord {
                id,
                hanzi: format!("字{id}"),
            })
            .collect()
    }

    #[test]
    fn question_is_order_independent_and_reproducible() {
        let words = words();
        let mut reversed = words.clone();
        reversed.reverse();
        let question = prepare(&words, 17).unwrap();
        assert_eq!(question, prepare(&reversed, 17).unwrap());
        assert_eq!(question.choices.len(), 3);
        assert_eq!(question.grade(question.correct_id), Ok(true));
    }

    #[test]
    fn rejects_insufficient_words_and_unknown_answers() {
        assert_eq!(
            prepare(&words()[..2], 1),
            Err(ListeningError::NotEnoughWords)
        );
        let question = prepare(&words(), 3).unwrap();
        assert_eq!(question.grade(-1), Err(ListeningError::InvalidAnswer));
    }

    #[test]
    fn surrounding_whitespace_cannot_create_duplicate_visible_answers() {
        let words = vec![
            ListeningWord {
                id: 1,
                hanzi: "你好".into(),
            },
            ListeningWord {
                id: 2,
                hanzi: " 你好 ".into(),
            },
            ListeningWord {
                id: 3,
                hanzi: "谢谢".into(),
            },
            ListeningWord {
                id: 4,
                hanzi: "再见".into(),
            },
        ];
        let question = prepare(&words, 9).unwrap();
        assert_eq!(question.choices.len(), 3);
        assert!(
            question
                .choices
                .iter()
                .all(|choice| choice.hanzi.trim() == choice.hanzi)
        );
        assert_eq!(
            question
                .choices
                .iter()
                .filter(|choice| choice.hanzi == "你好")
                .count(),
            1
        );
    }

    #[test]
    fn unusable_audio_words_are_excluded_before_selection() {
        let words = vec![
            ListeningWord {
                id: 1,
                hanzi: "  ".into(),
            },
            ListeningWord {
                id: 2,
                hanzi: "你好".into(),
            },
            ListeningWord {
                id: 3,
                hanzi: "谢谢".into(),
            },
            ListeningWord {
                id: 4,
                hanzi: "再见".into(),
            },
        ];
        let question = prepare(&words, 77).unwrap();
        assert!(!question.spoken_hanzi.trim().is_empty());
        assert!(question.choices.iter().all(|word| word.id != 1));
    }

    #[test]
    fn overly_long_audio_labels_are_not_playable_candidates() {
        let mut words = words();
        words[0].hanzi = "汉".repeat(37);
        let question = prepare(&words, 13).unwrap();
        assert!(question.choices.iter().all(|choice| choice.id != 1));
        assert!(question.spoken_hanzi.chars().count() <= 36);
    }

    #[test]
    fn rejects_duplicate_ids_and_collapses_equal_hanzi() {
        let mut words = words();
        words.push(words[0].clone());
        assert_eq!(prepare(&words, 2), Err(ListeningError::DuplicateIds));
        let words = vec![
            ListeningWord {
                id: 1,
                hanzi: "你好".into(),
            },
            ListeningWord {
                id: 2,
                hanzi: "你好".into(),
            },
            ListeningWord {
                id: 3,
                hanzi: "谢谢".into(),
            },
        ];
        assert_eq!(prepare(&words, 1), Err(ListeningError::NotEnoughWords));
    }

    #[test]
    fn control_characters_are_excluded_from_synthesized_prompts() {
        let candidates = vec![
            ListeningWord {
                id: 1,
                hanzi: "你\n好".into(),
            },
            ListeningWord {
                id: 2,
                hanzi: "茶".into(),
            },
            ListeningWord {
                id: 3,
                hanzi: "书".into(),
            },
            ListeningWord {
                id: 4,
                hanzi: "水".into(),
            },
        ];
        let question = prepare(&candidates, 1).unwrap();
        assert!(
            question
                .choices
                .iter()
                .all(|word| !word.hanzi.chars().any(char::is_control))
        );
        assert_eq!(question.choices.len(), 3);
    }
}
