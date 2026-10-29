use std::{collections::BTreeSet, rc::Rc};
use yew::Reducible;
use zipeek_core::{
    Hsk2Level, Hsk3Level, HskLevel, HskSystem, WordId,
    practice::{
        ExerciseVariant, FixedRounds, NormalizedPracticeQuery, PracticeFilter, PracticeMode,
        SeedDisposition,
    },
};

#[derive(Clone, PartialEq)]
pub(crate) struct SetupState {
    pub types: Vec<ExerciseVariant>,
    pub level: HskLevel,
    pub fixed: bool,
    pub rounds: FixedRounds,
}

pub(crate) enum SetupAction {
    Activity(ExerciseVariant),
    System(HskSystem),
    Level(HskLevel),
    Fixed(bool),
    Rounds(FixedRounds),
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            types: vec![ExerciseVariant::HanziToMeaning],
            level: HskLevel::Hsk3(Hsk3Level::One),
            fixed: false,
            rounds: FixedRounds::Ten,
        }
    }
}

impl SetupState {
    pub fn query(&self, exclusions: BTreeSet<WordId>) -> NormalizedPracticeQuery {
        NormalizedPracticeQuery {
            mode: if self.fixed {
                PracticeMode::Fixed(self.rounds)
            } else {
                PracticeMode::Endless
            },
            filter: PracticeFilter {
                level: self.level,
                part_of_speech: Vec::new(),
            },
            exclusions,
            offset: 0,
            seed: SeedDisposition::Unseeded,
            warnings: Vec::new(),
        }
    }
}

impl Reducible for SetupState {
    type Action = SetupAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            SetupAction::Activity(value) => {
                if next.types.contains(&value) {
                    if next.types.len() == 1 {
                        return self;
                    }
                    next.types.retain(|variant| *variant != value);
                } else {
                    next.types.push(value);
                }
                next.types = crate::services::exercise_types::canonical(&next.types);
            }
            SetupAction::System(system) => {
                next.level = match system {
                    HskSystem::Hsk2 => HskLevel::Hsk2(
                        Hsk2Level::try_from(self.level.as_i32()).unwrap_or(Hsk2Level::One),
                    ),
                    HskSystem::Hsk3 => HskLevel::Hsk3(
                        Hsk3Level::try_from(self.level.as_i32()).unwrap_or(Hsk3Level::One),
                    ),
                };
            }
            SetupAction::Level(level) if level.system() == self.level.system() => {
                next.level = level
            }
            SetupAction::Level(_) => return self,
            SetupAction::Fixed(value) => next.fixed = value,
            SetupAction::Rounds(value) => next.rounds = value,
        }
        next.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiple_types_toggle_but_the_last_type_cannot_be_removed() {
        let state = Rc::new(SetupState::default());
        let unchanged = state
            .clone()
            .reduce(SetupAction::Activity(ExerciseVariant::HanziToMeaning));
        assert!(Rc::ptr_eq(&state, &unchanged));
        let state = state.reduce(SetupAction::Activity(ExerciseVariant::HanziEng));
        assert_eq!(
            state.types,
            vec![ExerciseVariant::HanziToMeaning, ExerciseVariant::HanziEng]
        );
        let state = state.reduce(SetupAction::Activity(ExerciseVariant::HanziToMeaning));
        assert_eq!(state.types, vec![ExerciseVariant::HanziEng]);
    }
    #[test]
    fn advanced_band_cannot_survive_switch_to_hsk2_or_stale_level_event() {
        let state = Rc::new(SetupState::default())
            .reduce(SetupAction::Level(HskLevel::Hsk3(Hsk3Level::SevenToNine)));
        let state = state.reduce(SetupAction::System(HskSystem::Hsk2));
        assert_eq!(state.level, HskLevel::Hsk2(Hsk2Level::One));
        let stale = state
            .clone()
            .reduce(SetupAction::Level(HskLevel::Hsk3(Hsk3Level::SevenToNine)));
        assert!(Rc::ptr_eq(&state, &stale));
    }

    #[test]
    fn switching_modes_preserves_count_but_endless_url_omits_it_and_has_no_seed() {
        let ids = BTreeSet::from([WordId::try_from(2).unwrap(), WordId::try_from(1).unwrap()]);
        let state = Rc::new(SetupState::default())
            .reduce(SetupAction::Rounds(FixedRounds::Five))
            .reduce(SetupAction::Fixed(true));
        let pairs = state
            .query(ids)
            .query_pairs()
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(pairs.get("rounds").map(String::as_str), Some("5"));
        assert_eq!(pairs.get("exclude").map(String::as_str), Some("1,2"));
        let state = state.reduce(SetupAction::Fixed(false));
        let pairs = state.query(BTreeSet::new()).query_pairs();
        assert!(
            !pairs
                .iter()
                .any(|(key, _)| matches!(key.as_str(), "rounds" | "seed" | "exclude" | "pos"))
        );
        assert!(pairs.contains(&("offset".into(), "0".into())));
        assert_eq!(
            state.reduce(SetupAction::Fixed(true)).rounds,
            FixedRounds::Five
        );
    }
}
