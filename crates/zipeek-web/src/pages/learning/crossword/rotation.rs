use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use zipeek_exercises::crossword::{Crossword, CrosswordWord};

pub(super) const HISTORY_KEY: &str = "zipeek.crosswords.rotation.v1";
const MAX_WORDS: usize = 100;
const MAX_GAMES: usize = 10;
const CANDIDATE_COUNT: u64 = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CrosswordRotation {
    words: Vec<WordUsage>,
    games: Vec<RecentGame>,
    next_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WordUsage {
    answer: String,
    last_used: u64,
    times_used: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecentGame {
    session_id: String,
    answers: Vec<String>,
}

fn answers(game: &Crossword) -> Vec<String> {
    game.clues()
        .iter()
        .map(|clue| clue.answer.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

impl CrosswordRotation {
    pub(super) fn valid(&self) -> bool {
        self.words.len() <= MAX_WORDS
            && self.games.len() <= MAX_GAMES
            && self.words.iter().all(|word| {
                (2..=5).contains(&word.answer.chars().count())
                    && word.answer.chars().all(|ch| ('\u{3400}'..='\u{9fff}').contains(&ch))
                    && word.last_used <= self.next_sequence
                    && word.times_used > 0
            })
            && self.words.iter().map(|word| &word.answer).collect::<BTreeSet<_>>().len()
                == self.words.len()
            && self.games.iter().all(|game| {
                game.session_id.len() == 36
                    && game.answers.len() <= 128
                    && game.answers.iter().all(|answer| answer.len() <= 20)
            })
            && self.games.iter().map(|game| &game.session_id).collect::<BTreeSet<_>>().len()
                == self.games.len()
    }

    fn candidate_score(&self, game: &Crossword) -> i64 {
        let candidate = answers(game);
        let candidate_set: BTreeSet<_> = candidate.iter().collect();
        let mut score = game.clues().len() as i64 * 12 + game.intersection_count() as i64 * 2;
        for answer in &candidate {
            if let Some(entry) = self.words.iter().find(|entry| entry.answer == *answer) {
                let age = self.next_sequence.saturating_sub(entry.last_used);
                score += (age.min(20) as i64) * 3 - 80;
                score -= i64::from(entry.times_used.min(10)) * 2;
            } else {
                score += 110;
            }
        }
        for (index, recent) in self.games.iter().enumerate() {
            let overlap = recent
                .answers
                .iter()
                .filter(|answer| candidate_set.contains(answer))
                .count();
            score -= overlap as i64 * (50 / (index as i64 + 1));
            if candidate == recent.answers {
                score -= 2000;
            }
        }
        score
    }

    pub(super) fn generate(&self, seed: u64, vocabulary: &[CrosswordWord]) -> Option<Crossword> {
        (0..CANDIDATE_COUNT)
            .map(|retry| Crossword::generated(seed.wrapping_add(retry), vocabulary.to_vec()))
            .filter(Crossword::meets_quality_floor)
            .max_by_key(|game| self.candidate_score(game))
    }

    pub(super) fn record(&mut self, session_id: &str, game: &Crossword) -> bool {
        self.record_answers(session_id, answers(game))
    }

    fn record_answers(&mut self, session_id: &str, selected: Vec<String>) -> bool {
        if selected.is_empty()
            || self.games.iter().any(|recent| recent.session_id == session_id)
        {
            return false;
        }
        for answer in &selected {
            self.next_sequence = self.next_sequence.saturating_add(1);
            if let Some(entry) = self.words.iter_mut().find(|entry| entry.answer == *answer) {
                entry.last_used = self.next_sequence;
                entry.times_used = entry.times_used.saturating_add(1);
            } else {
                self.words.push(WordUsage {
                    answer: answer.clone(),
                    last_used: self.next_sequence,
                    times_used: 1,
                });
            }
        }
        self.words.sort_by_key(|entry| std::cmp::Reverse(entry.last_used));
        self.words.truncate(MAX_WORDS);
        self.games.insert(0, RecentGame {
            session_id: session_id.to_owned(),
            answers: selected,
        });
        self.games.truncate(MAX_GAMES);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocabulary() -> Vec<CrosswordWord> {
        [
            ("学生", "student", "xué shēng"), ("生活", "life", "shēng huó"),
            ("活动", "activity", "huó dòng"), ("动物", "animal", "dòng wù"),
            ("物理", "physics", "wù lǐ"), ("理想", "ideal", "lǐ xiǎng"),
            ("想法", "thought", "xiǎng fǎ"), ("老师", "teacher", "lǎo shī"),
            ("老人", "elder", "lǎo rén"), ("人生", "life", "rén shēng"),
            ("生日", "birthday", "shēng rì"), ("日记", "diary", "rì jì"),
            ("法师", "master", "fǎ shī"),
        ]
        .into_iter()
        .map(|(answer, english, pinyin)| CrosswordWord {
            answer: answer.to_owned(), english: english.to_owned(), pinyin: pinyin.to_owned(),
        })
        .collect()
    }

    #[test]
    fn records_each_session_once_and_caps_recent_games() {
        let game = Crossword::generated(15, vocabulary());
        let mut history = CrosswordRotation::default();
        assert!(history.record("first", &game));
        assert!(!history.record("first", &game));
        for n in 0..20 {
            history.record(&format!("game-{n}"), &game);
        }
        assert_eq!(history.games.len(), 10);
        assert_eq!(history.words.iter().map(|entry| entry.times_used).max(), Some(21));
    }

    #[test]
    fn evicts_least_recently_used_unique_word() {
        let mut history = CrosswordRotation::default();
        for n in 0..100u32 {
            let answer = format!("中{}", char::from_u32(0x4e00 + n).unwrap());
            assert!(history.record_answers(
                &format!("{n:08x}-0000-4000-8000-000000000000"),
                vec![answer],
            ));
        }
        assert_eq!(history.words.len(), 100);
        let first = history.words.last().unwrap().answer.clone();
        let second = history.words[98].answer.clone();
        assert!(history.record_answers("00000064-0000-4000-8000-000000000000", vec![first.clone()]));
        assert!(history.record_answers("00000065-0000-4000-8000-000000000000", vec!["中文".to_owned()]));
        assert_eq!(history.words.len(), 100);
        assert!(history.words.iter().any(|entry| entry.answer == first));
        assert!(!history.words.iter().any(|entry| entry.answer == second));
        assert!(history.valid());
    }

    #[test]
    fn selection_is_repeatable_and_penalizes_previous_puzzle() {
        let pool = vocabulary();
        let empty = CrosswordRotation::default();
        let first = empty.generate(123, &pool).expect("playable puzzle");
        let mut history = CrosswordRotation::default();
        history.record("first", &first);
        let selected = history.generate(124, &pool).expect("playable puzzle");
        assert_eq!(selected, history.generate(124, &pool).unwrap());
        let fresh_score = empty.candidate_score(&selected);
        let repeated_score = history.candidate_score(&selected);
        assert!(repeated_score < fresh_score);
        let best_score = (0..CANDIDATE_COUNT)
            .map(|retry| Crossword::generated(124u64.wrapping_add(retry), pool.clone()))
            .filter(Crossword::meets_quality_floor)
            .map(|candidate| history.candidate_score(&candidate))
            .max()
            .unwrap();
        assert_eq!(history.candidate_score(&selected), best_score);
    }

    #[test]
    fn rejects_oversized_or_duplicate_history() {
        let mut history = CrosswordRotation::default();
        history.words.push(WordUsage { answer: "学生".to_owned(), last_used: 1, times_used: 1 });
        history.next_sequence = 1;
        assert!(history.valid());
        history.words.push(history.words[0].clone());
        assert!(!history.valid());
    }
}
