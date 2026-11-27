use std::collections::BTreeSet;

use crate::definition::{PRACTICE_PLAYLIST_ORDER, definition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preparation {
    Local,
    AiGenerated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledExercise {
    pub ordinal: usize,
    pub exercise_id: &'static str,
    pub instance_seed: u64,
    pub preparation: Preparation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    NoExercises,
    UnknownExercise(String),
    InvalidLength,
}

fn weight(id: &str) -> usize {
    match id {
        "sentence-translation" => 3,
        "matching" | "multiple-choice" => 2,
        _ => 1,
    }
}

pub fn requires_generation(id: &str) -> bool {
    crate::definition::needs_generated_content(id)
}

fn mix_seed(seed: u64, ordinal: usize) -> u64 {
    let mut x = seed ^ (ordinal as u64).wrapping_mul(0x9e3779b97f4a7c15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

pub fn schedule(
    selected: &BTreeSet<String>,
    count: usize,
    seed: u64,
) -> Result<Vec<ScheduledExercise>, ScheduleError> {
    if selected.is_empty() {
        return Err(ScheduleError::NoExercises);
    }
    if count == 0 || count > 1_000 {
        return Err(ScheduleError::InvalidLength);
    }
    for id in selected {
        if definition(id).is_none() {
            return Err(ScheduleError::UnknownExercise(id.clone()));
        }
    }
    let ids: Vec<&'static str> = PRACTICE_PLAYLIST_ORDER
        .into_iter()
        .filter(|id| selected.contains(*id))
        .collect();
    let total_weight: usize = ids.iter().map(|id| weight(id)).sum();
    let mut allocations: Vec<usize> = ids
        .iter()
        .map(|id| count * weight(id) / total_weight)
        .collect();
    let assigned: usize = allocations.iter().sum();
    let mut remainders: Vec<usize> = (0..ids.len()).collect();
    remainders.sort_by_key(|&i| {
        (
            std::cmp::Reverse(count * weight(ids[i]) % total_weight),
            (i + (seed as usize % ids.len())) % ids.len(),
        )
    });
    for &i in remainders.iter().take(count - assigned) {
        allocations[i] += 1;
    }
    let original = allocations.clone();
    let mut output = Vec::with_capacity(count);
    let mut previous = None;
    for ordinal in 0..count {
        let available_alternatives = allocations
            .iter()
            .enumerate()
            .any(|(i, left)| *left > 0 && Some(i) != previous);
        let next = (0..ids.len())
            .filter(|&i| allocations[i] > 0 && (!available_alternatives || Some(i) != previous))
            .max_by_key(|&i| {
                (
                    allocations[i] * count / original[i],
                    allocations[i],
                    std::cmp::Reverse(i),
                )
            })
            .expect("validated allocations must cover every slot");
        allocations[next] -= 1;
        previous = Some(next);
        output.push(ScheduledExercise {
            ordinal,
            exercise_id: ids[next],
            instance_seed: mix_seed(seed, ordinal),
            preparation: if requires_generation(ids[next]) {
                Preparation::AiGenerated
            } else {
                Preparation::Local
            },
        });
    }
    Ok(output)
}

pub fn endless_step(
    selected: &BTreeSet<String>,
    ordinal: usize,
    seed: u64,
) -> Result<ScheduledExercise, ScheduleError> {
    const CYCLE_LEN: usize = 32;
    let cycle = ordinal / CYCLE_LEN;
    let cycle_seed = mix_seed(seed, cycle);
    let cycle_steps = schedule(selected, CYCLE_LEN, cycle_seed)?;
    let mut step = cycle_steps[ordinal % CYCLE_LEN].clone();
    step.ordinal = ordinal;
    Ok(step)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selected(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn endless_scheduler_replays_mixed_cycles_without_repeating_step_ids() {
        let selected = selected(&["matching", "sentence-translation", "write-hanzi"]);
        let first = (0..96)
            .map(|ordinal| endless_step(&selected, ordinal, 2026).unwrap())
            .collect::<Vec<_>>();
        let replay = (0..96)
            .map(|ordinal| endless_step(&selected, ordinal, 2026).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(first, replay);
        assert!(
            first
                .iter()
                .enumerate()
                .all(|(ordinal, step)| step.ordinal == ordinal)
        );
        for id in selected {
            assert!(first.iter().any(|step| step.exercise_id == id));
        }
        assert!(
            first
                .windows(2)
                .any(|window| window[0].exercise_id != window[1].exercise_id)
        );
    }

    #[test]
    fn character_hunt_is_deterministic_local_preparation() {
        let ids = selected(&["character-hunt", "matching"]);
        let scheduled = schedule(&ids, 8, 9).unwrap();
        assert_eq!(scheduled, schedule(&ids, 8, 9).unwrap());
        assert!(
            scheduled
                .iter()
                .any(|step| step.exercise_id == "character-hunt")
        );
        assert!(
            scheduled
                .iter()
                .all(|step| step.preparation == Preparation::Local)
        );
    }

    #[test]
    fn mixed_local_hosts_are_stable_without_ai_preparation() {
        let exercises = selected(&["matching", "multiple-choice", "listening-choice"]);
        let steps = schedule(&exercises, 20, 98).unwrap();
        assert_eq!(steps, schedule(&exercises, 20, 98).unwrap());
        for exercise in &exercises {
            assert!(steps.iter().any(|step| step.exercise_id == exercise));
        }
        assert!(steps.iter().all(|step| {
            step.preparation == Preparation::Local
                && crate::definition::supports_resumable_session_host(step.exercise_id)
        }));
    }

    #[test]
    fn unsupported_and_generated_exercises_are_classified_without_readiness_claims() {
        let entries = schedule(
            &selected(&[
                "matching",
                "listening-choice",
                "sentence-translation",
                "image-vocabulary",
            ]),
            12,
            81,
        )
        .unwrap();
        assert_eq!(entries.len(), 12);
        assert!(entries.iter().all(|entry| {
            entry.preparation
                == if requires_generation(entry.exercise_id) {
                    Preparation::AiGenerated
                } else {
                    Preparation::Local
                }
        }));
        assert_eq!(
            schedule(&selected(&["matching"]), 1_001, 1),
            Err(ScheduleError::InvalidLength)
        );
    }

    #[test]
    fn generated_steps_are_recognized_by_the_resumable_host_registry() {
        let planned = schedule(
            &selected(&["sentence-translation", "image-vocabulary"]),
            8,
            17,
        )
        .unwrap();
        assert!(planned.iter().all(|step| {
            step.preparation == Preparation::AiGenerated
                && crate::definition::supports_resumable_session_host(step.exercise_id)
        }));
        assert_eq!(planned.len(), 8);
    }

    #[test]
    fn weighted_matching_translation_is_interleaved_and_repeatable() {
        let choice = selected(&["matching", "sentence-translation"]);
        let a = schedule(&choice, 5, 42).unwrap();
        assert_eq!(a, schedule(&choice, 5, 42).unwrap());
        assert_eq!(
            a.iter()
                .filter(|step| step.exercise_id == "matching")
                .count(),
            2,
        );
        assert_eq!(
            a.iter()
                .filter(|step| step.exercise_id == "sentence-translation")
                .count(),
            3,
        );
        assert!(
            a.windows(2)
                .any(|pair| pair[0].exercise_id != pair[1].exercise_id)
        );
        assert_eq!(
            a.iter()
                .map(|step| step.instance_seed)
                .collect::<BTreeSet<_>>()
                .len(),
            5,
        );
    }

    #[test]
    fn each_generated_step_has_a_distinct_seed_for_recovery() {
        let steps = schedule(
            &selected(&["image-vocabulary", "sentence-translation"]),
            20,
            0,
        )
        .unwrap();
        assert_eq!(
            steps
                .iter()
                .map(|step| step.instance_seed)
                .collect::<BTreeSet<_>>()
                .len(),
            20
        );
        assert!(
            steps
                .iter()
                .all(|step| requires_generation(step.exercise_id))
        );
    }

    #[test]
    fn one_type_can_repeat_with_distinct_instances() {
        let steps = schedule(&selected(&["sentence-translation"]), 5, 7).unwrap();
        assert_eq!(steps.len(), 5);
        assert!(
            steps
                .iter()
                .all(|step| step.preparation == Preparation::AiGenerated)
        );
        assert_eq!(
            steps
                .iter()
                .map(|step| step.instance_seed)
                .collect::<BTreeSet<_>>()
                .len(),
            5,
        );
    }

    #[test]
    fn authored_exercises_are_locally_ready_and_reproducibly_interleaved() {
        let exercises = selected(&[
            "fill-blank",
            "audio-dictation",
            "sentence-builder",
            "spot-mistake",
            "type-pinyin",
        ]);
        let first = schedule(&exercises, 5, 17).unwrap();
        let replay = schedule(&exercises, 5, 17).unwrap();
        assert_eq!(first, replay);
        assert!(
            first
                .iter()
                .all(|step| step.preparation == Preparation::Local)
        );
        assert!(first.iter().any(|step| step.exercise_id == "fill-blank"));
        assert!(
            first
                .iter()
                .any(|step| step.exercise_id == "audio-dictation")
        );
        assert!(
            schedule(&selected(&["sentence-builder"]), 5, 17)
                .unwrap()
                .iter()
                .all(|step| step.exercise_id == "sentence-builder"
                    && step.preparation == Preparation::Local)
        );
        assert!(
            schedule(&selected(&["spot-mistake"]), 5, 17)
                .unwrap()
                .iter()
                .all(|step| step.exercise_id == "spot-mistake"
                    && step.preparation == Preparation::Local)
        );
        assert_eq!(
            first.iter().map(|step| step.ordinal).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        assert_eq!(
            schedule(&BTreeSet::new(), 5, 0),
            Err(ScheduleError::NoExercises),
        );
        assert_eq!(
            schedule(&selected(&["matching"]), 0, 0),
            Err(ScheduleError::InvalidLength),
        );
        assert!(matches!(
            schedule(&selected(&["unknown"]), 5, 0),
            Err(ScheduleError::UnknownExercise(_)),
        ));
    }

    #[test]
    fn empty_and_unknown_selection_never_produces_a_partial_schedule() {
        assert_eq!(
            schedule(&selected(&[]), 5, 1),
            Err(ScheduleError::NoExercises)
        );
        assert_eq!(
            schedule(&selected(&["unknown"]), 5, 1),
            Err(ScheduleError::UnknownExercise("unknown".into()))
        );
        assert_eq!(
            schedule(&selected(&["matching"]), 0, 1),
            Err(ScheduleError::InvalidLength)
        );
        let steps = schedule(&selected(&["matching"]), 1_000, u64::MAX).unwrap();
        assert_eq!(steps.len(), 1_000);
        assert!(steps.iter().enumerate().all(|(i, step)| step.ordinal == i));
    }

    #[test]
    fn generated_exercises_cannot_be_scheduled_as_local_questions() {
        let exercises = selected(&["sentence-translation", "image-vocabulary"]);
        let scheduled = schedule(&exercises, 8, 42).unwrap();
        assert_eq!(scheduled.len(), 8);
        assert!(
            scheduled
                .iter()
                .all(|step| step.preparation == Preparation::AiGenerated)
        );
        assert_eq!(scheduled, schedule(&exercises, 8, 42).unwrap());
    }
}
