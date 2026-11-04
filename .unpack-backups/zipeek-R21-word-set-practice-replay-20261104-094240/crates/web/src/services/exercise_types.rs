use std::collections::{BTreeMap, HashMap};
use yew_router::prelude::Location;
use zipeek_core::practice::{ExerciseVariant, NormalizedPracticeQuery};

pub(crate) const EXERCISES: [ExerciseVariant; 6] = [
    ExerciseVariant::HanziToMeaning,
    ExerciseVariant::MeaningToHanzi,
    ExerciseVariant::MeaningToPinyin,
    ExerciseVariant::EngPinyin,
    ExerciseVariant::PinyinHanzi,
    ExerciseVariant::HanziEng,
];

pub(crate) fn canonical(types: &[ExerciseVariant]) -> Vec<ExerciseVariant> {
    EXERCISES
        .into_iter()
        .filter(|variant| types.contains(variant))
        .collect()
}

pub(crate) fn read_types(
    location: Option<&Location>,
    current: ExerciseVariant,
) -> Vec<ExerciseVariant> {
    let raw = location
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .and_then(|query| query.get("types").cloned());
    parse_types(raw.as_deref(), current)
}

fn parse_types(raw: Option<&str>, current: ExerciseVariant) -> Vec<ExerciseVariant> {
    let Some(raw) = raw else { return vec![current] };
    let types = EXERCISES
        .into_iter()
        .filter(|variant| {
            raw.split(',')
                .any(|token| token.trim() == variant.seed_token())
        })
        .collect::<Vec<_>>();
    if types.contains(&current) {
        types
    } else {
        log::warn!("Practice types must include the current route; using this type only");
        vec![current]
    }
}

pub(crate) fn query_pairs(
    query: &NormalizedPracticeQuery,
    types: &[ExerciseVariant],
) -> BTreeMap<String, String> {
    let mut pairs = query.query_pairs().into_iter().collect::<BTreeMap<_, _>>();
    let types = canonical(types);
    if types.len() > 1 {
        pairs.insert(
            "types".into(),
            types
                .iter()
                .map(|variant| variant.seed_token())
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    pairs
}

pub(crate) fn choose(
    types: &[ExerciseVariant],
    previous: Option<ExerciseVariant>,
    sample: u64,
) -> Option<ExerciseVariant> {
    let candidates = types
        .iter()
        .copied()
        .filter(|variant| types.len() == 1 || Some(*variant) != previous)
        .collect::<Vec<_>>();
    candidates
        .get((sample % candidates.len().max(1) as u64) as usize)
        .copied()
}

pub(crate) fn random_type(
    types: &[ExerciseVariant],
    previous: Option<ExerciseVariant>,
) -> Result<ExerciseVariant, String> {
    let sample = if types.len() > 1 {
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
        u64::from_le_bytes(bytes)
    } else {
        0
    };
    choose(types, previous, sample).ok_or_else(|| "Choose at least one exercise type".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn type_lists_are_canonical_and_route_safe_without_changing_round_queries() {
        let current = ExerciseVariant::HanziToMeaning;
        assert_eq!(
            parse_types(
                Some("hanzi-eng,unknown,hanzi-to-meaning,hanzi-eng"),
                current
            ),
            vec![current, ExerciseVariant::HanziEng]
        );
        assert_eq!(parse_types(Some("hanzi-eng"), current), vec![current]);
        let query =
            zipeek_core::practice::ParsedPracticeQuery::default().normalize(current, |_| None);
        let before = query.canonical_seed_input(current);
        assert_eq!(
            query_pairs(&query, &[current, ExerciseVariant::HanziEng])["types"],
            "hanzi-to-meaning,hanzi-eng"
        );
        assert!(!query_pairs(&query, &[current]).contains_key("types"));
        assert_eq!(before, query.canonical_seed_input(current));
    }
    #[test]
    fn random_choice_excludes_previous_except_for_single_type_sessions() {
        for previous in EXERCISES {
            for sample in [0, 1, 5, u64::MAX] {
                assert_ne!(choose(&EXERCISES, Some(previous), sample), Some(previous));
            }
        }
        assert_eq!(
            choose(&[EXERCISES[0]], Some(EXERCISES[0]), 0),
            Some(EXERCISES[0])
        );
        assert_eq!(choose(&[], None, 0), None);
    }
}
