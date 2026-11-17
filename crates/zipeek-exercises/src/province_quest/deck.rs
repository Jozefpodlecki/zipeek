use std::collections::HashSet;

use super::{AnswerMode, Difficulty, GeoDataset, MapRegion, QuestionKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub region_id: String,
    pub kind: QuestionKind,
    pub parent_id: Option<String>,
    pub answer_mode: AnswerMode,
    pub options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeckError {
    NoRegions,
    MissingPrefectureMap,
    NotEnoughDistinctRegions,
}

fn next_random(rng: &mut u64) -> u64 {
    *rng ^= *rng << 13;
    *rng ^= *rng >> 7;
    *rng ^= *rng << 17;
    *rng
}

fn shuffle<T>(items: &mut [T], rng: &mut u64) {
    for position in (1..items.len()).rev() {
        let swap = (next_random(rng) as usize) % (position + 1);
        items.swap(position, swap);
    }
}

fn options_for(target: &MapRegion, regions: &[&MapRegion], rng: &mut u64) -> Option<Vec<String>> {
    let mut candidates: Vec<_> = regions
        .iter()
        .copied()
        .filter(|region| region.id != target.id && region.hanzi != target.hanzi)
        .collect();
    shuffle(&mut candidates, rng);
    let mut used = HashSet::from([target.hanzi.as_str()]);
    let mut choices = vec![target.id.clone()];
    for candidate in candidates {
        if used.insert(candidate.hanzi.as_str()) {
            choices.push(candidate.id.clone());
        }
        if choices.len() == 3 {
            break;
        }
    }
    if choices.len() != 3 {
        return None;
    }
    shuffle(&mut choices, rng);
    Some(choices)
}

pub fn build_deck(
    dataset: &GeoDataset,
    difficulty: Difficulty,
    seed: u64,
    count: usize,
) -> Result<Vec<Question>, DeckError> {
    let mut regions: Vec<_> = dataset
        .regions
        .iter()
        .filter(|region| match difficulty {
            Difficulty::Easy | Difficulty::Medium => region.parent_id.is_none(),
            Difficulty::Hard => region.parent_id.is_some(),
        })
        .collect();
    if regions.is_empty() {
        return Err(if difficulty == Difficulty::Hard {
            DeckError::MissingPrefectureMap
        } else {
            DeckError::NoRegions
        });
    }
    if regions
        .iter()
        .map(|region| region.hanzi.as_str())
        .collect::<HashSet<_>>()
        .len()
        < 3
    {
        return Err(DeckError::NotEnoughDistinctRegions);
    }
    let mut rng = seed | 1;
    shuffle(&mut regions, &mut rng);
    let kinds = difficulty.kinds();
    let mut questions = Vec::new();
    for (index, region) in regions.iter().take(count).enumerate() {
        let answer_mode = if index % 2 == 0 {
            AnswerMode::SelectMap
        } else {
            AnswerMode::SelectHanzi
        };
        let kind = if answer_mode == AnswerMode::SelectHanzi {
            QuestionKind::FindName
        } else {
            let candidate = kinds[(index / 2) % kinds.len()];
            if candidate == QuestionKind::FindCapital && region.capital_hanzi.is_none() {
                QuestionKind::FindName
            } else {
                candidate
            }
        };
        let options = if answer_mode == AnswerMode::SelectHanzi {
            let local: Vec<_> = regions
                .iter()
                .copied()
                .filter(|candidate| candidate.parent_id == region.parent_id)
                .collect();
            options_for(region, &local, &mut rng)
                .or_else(|| options_for(region, &regions, &mut rng))
                .ok_or(DeckError::NotEnoughDistinctRegions)?
        } else {
            Vec::new()
        };
        questions.push(Question {
            region_id: region.id.clone(),
            kind,
            parent_id: region.parent_id.clone(),
            answer_mode,
            options,
        });
    }
    Ok(questions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::province_quest::RegionKind;

    fn fixture() -> GeoDataset {
        let regions = (0..8)
            .map(|number| MapRegion {
                id: format!("{number}"),
                parent_id: None,
                hanzi: format!("地区{number}"),
                pinyin: format!("region {number}"),
                english: format!("Region {number}"),
                capital_hanzi: Some(format!("首府{number}")),
                capital_pinyin: Some(format!("Capital {number}")),
                kind: RegionKind::Province,
                path: "M0,0Z".into(),
                bbox: [0.0, 0.0, 10.0, 10.0],
            })
            .collect();
        GeoDataset {
            source: "source".into(),
            version: "1".into(),
            attribution: "credit".into(),
            view_box: "0 0 10 10".into(),
            regions,
        }
    }

    #[test]
    fn both_question_formats_are_reproducible() {
        let data = fixture();
        let first = build_deck(&data, Difficulty::Easy, 42, 8).unwrap();
        assert_eq!(first, build_deck(&data, Difficulty::Easy, 42, 8).unwrap());
        assert!(
            first
                .iter()
                .any(|question| question.answer_mode == AnswerMode::SelectMap)
        );
        assert!(
            first
                .iter()
                .any(|question| question.answer_mode == AnswerMode::SelectHanzi)
        );
        for question in first {
            if question.answer_mode == AnswerMode::SelectHanzi {
                assert_eq!(question.options.len(), 3);
                assert!(question.options.contains(&question.region_id));
                assert_eq!(question.kind, QuestionKind::FindName);
            } else {
                assert!(question.options.is_empty());
            }
        }
    }

    #[test]
    fn hard_mode_uses_children_and_prefers_local_distractors() {
        let mut data = fixture();
        for region in &mut data.regions {
            region.parent_id = Some("99".into());
        }
        let questions = build_deck(&data, Difficulty::Hard, 8, 8).unwrap();
        assert!(
            questions
                .iter()
                .all(|question| question.parent_id.as_deref() == Some("99"))
        );
        assert!(
            questions
                .iter()
                .filter(|question| question.answer_mode == AnswerMode::SelectHanzi)
                .all(|question| question.options.len() == 3)
        );
    }

    #[test]
    fn not_enough_distinct_regions_is_rejected() {
        let mut data = fixture();
        data.regions.truncate(2);
        assert_eq!(
            build_deck(&data, Difficulty::Easy, 1, 10),
            Err(DeckError::NotEnoughDistinctRegions)
        );
    }
}
