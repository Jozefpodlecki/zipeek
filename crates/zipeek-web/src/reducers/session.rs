use super::{
    matching::{CardChoice, MatchAction, MatchKey, MatchQuestion, MatchState},
    mcq::{McqAction, McqKey, McqQuestion, McqState},
};
use std::rc::Rc;
use yew::Reducible;
use zipeek_core::{
    practice::{ExerciseVariant, FixedProgress, NormalizedPracticeQuery},
    round::RoundContent,
};
use zipeek_exercises::session::{ExerciseInstance, ExerciseResult, QuizSession};

mod reducer;
mod round;
#[cfg(test)]
mod tests;
mod types;

pub(crate) use reducer::{SessionAction, SessionState};
pub(crate) use round::ActiveRound;
pub(crate) use types::{SessionKey, SessionStats};
