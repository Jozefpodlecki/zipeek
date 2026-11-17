use std::collections::BTreeSet;

use super::{Answer, Candidate, CharacterHunt, Difficulty, HuntError, Tile};

impl CharacterHunt {
    pub fn generate(
        candidates: &[Candidate],
        difficulty: Difficulty,
        seed: u64,
    ) -> Result<Self, HuntError> {
        let area = difficulty.size() * difficulty.size();
        let mut words: Vec<String> = candidates
            .iter()
            .filter(|candidate| candidate.is_eligible())
            .map(|candidate| candidate.hanzi.trim().to_owned())
            .collect();
        words.sort();
        words.dedup();
        if words.is_empty() {
            return Err(HuntError::InsufficientVocabulary {
                required: area / 3,
                available: 0,
            });
        }

        let mut best: Option<(Vec<String>, BTreeSet<char>, usize)> = None;
        for attempt in 0..32_u64 {
            let mut available = words.clone();
            shuffle(
                &mut available,
                seed.wrapping_add(attempt.wrapping_mul(0x9e37_79b9)),
            );
            let mut taken = BTreeSet::new();
            let mut chosen = Vec::new();
            let mut remaining = area;
            while remaining >= 2 {
                let prefer_three = remaining % 2 == 1;
                let lengths = if prefer_three { [3, 2] } else { [2, 3] };
                let picked = lengths
                    .into_iter()
                    .filter(|length| *length <= remaining)
                    .find_map(|length| {
                        available.iter().position(|word| {
                            word.chars().count() == length
                                && word.chars().collect::<BTreeSet<_>>().len() == length
                                && word.chars().all(|character| !taken.contains(&character))
                        })
                    });
                let Some(index) = picked else {
                    break;
                };
                let word = available.swap_remove(index);
                remaining -= word.chars().count();
                taken.extend(word.chars());
                chosen.push(word);
            }
            if best
                .as_ref()
                .is_none_or(|(_, _, best_remaining)| remaining < *best_remaining)
            {
                best = Some((chosen, taken, remaining));
            }
            if remaining == 0 {
                break;
            }
        }
        let (chosen, taken, remaining) = best.expect("at least one board-generation attempt");
        if chosen.is_empty() || remaining > 1 {
            return Err(HuntError::InsufficientVocabulary {
                required: area / 3,
                available: chosen.len(),
            });
        }

        let mut tiles = Vec::with_capacity(area);
        let mut answers = Vec::with_capacity(chosen.len());
        for word in chosen {
            let mut matching = candidates
                .iter()
                .filter(|candidate| candidate.hanzi.trim() == word);
            let readings = matching
                .next()
                .and_then(|candidate| candidate.readings.as_ref())
                .filter(|parts| parts.len() == word.chars().count())
                .filter(|parts| {
                    matching.all(|candidate| candidate.readings.as_ref() == Some(*parts))
                });
            let indices = (tiles.len()..tiles.len() + word.chars().count()).collect();
            tiles.extend(word.chars().enumerate().map(|(index, character)| Tile {
                character,
                pinyin: readings.and_then(|parts| parts.get(index)).cloned(),
                completed: false,
                filler: false,
            }));
            let meanings: BTreeSet<String> = candidates
                .iter()
                .filter(|candidate| candidate.hanzi.trim() == word)
                .filter_map(|candidate| candidate.meaning.as_ref())
                .cloned()
                .collect();
            let meaning = (!meanings.is_empty())
                .then(|| meanings.into_iter().take(3).collect::<Vec<_>>().join(" · "));
            answers.push(Answer {
                word,
                indices,
                meaning,
            });
        }
        if remaining == 1 {
            let filler = candidates
                .iter()
                .flat_map(|item| item.hanzi.chars())
                .find(|character| {
                    !taken.contains(character) && ('\u{3400}'..='\u{9fff}').contains(character)
                })
                .ok_or(HuntError::InsufficientVocabulary {
                    required: area,
                    available: taken.len(),
                })?;
            tiles.push(Tile {
                character: filler,
                pinyin: None,
                completed: false,
                filler: true,
            });
        }
        let mut order: Vec<usize> = (0..tiles.len()).collect();
        shuffle(&mut order, seed ^ 0x9e37_79b9_7f4a_7c15);
        let mut positions = vec![0; tiles.len()];
        for (new_index, old_index) in order.iter().enumerate() {
            positions[*old_index] = new_index;
        }
        tiles = order
            .into_iter()
            .map(|old_index| tiles[old_index].clone())
            .collect();
        for answer in &mut answers {
            answer.indices = answer
                .indices
                .iter()
                .map(|index| positions[*index])
                .collect();
        }
        Ok(Self {
            difficulty,
            tiles,
            selected: Vec::new(),
            found: Vec::new(),
            mistakes: 0,
            answers,
        })
    }
}

fn shuffle<T>(values: &mut [T], seed: u64) {
    let mut state = seed | 1;
    for index in (1..values.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        values.swap(index, (state as usize) % (index + 1));
    }
}
