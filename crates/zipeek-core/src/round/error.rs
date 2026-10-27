use std::{error::Error, fmt};

use super::{CandidateView, ViewAttempt};
use crate::{
    WordId,
    practice::{ExerciseVariant, PracticeFilter},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    InvalidPool {
        view: CandidateView,
        id: WordId,
        reason: &'static str,
    },
    InvalidSeed {
        id: WordId,
    },
    MissingRecoveryViews {
        filter: PracticeFilter,
    },
    RecoveryUnavailable {
        attempts: Vec<WordId>,
    },
    InsufficientPool {
        variant: ExerciseVariant,
        attempts: Vec<ViewAttempt>,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPool { view, id, reason } => {
                write!(f, "invalid {view:?} candidate pool at word {id}: {reason}")
            }
            Self::InvalidSeed { id } => write!(
                f,
                "seed {id} does not match normalized seed state; supply the original row and validate it before building"
            ),
            Self::MissingRecoveryViews { filter } => write!(
                f,
                "supply candidate views for recovered seed context {filter:?}"
            ),
            Self::RecoveryUnavailable { attempts } => write!(
                f,
                "no eligible recovery seed after {} draws and ordered fallback",
                attempts.len()
            ),
            Self::InsufficientPool { variant, attempts } => write!(
                f,
                "not enough distinct words for {}; {} candidate views failed; try a different HSK level or reset exclusions",
                variant.seed_token(),
                attempts.len()
            ),
        }
    }
}
impl Error for BuildError {}
