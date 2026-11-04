use rand::{SeedableRng, rngs::ChaCha12Rng};

use super::{
    BuildError, CandidatePool, CandidateView, ContextPool, RoundContent, ViewAttempt,
    content::shuffled,
    labels::eligible,
    pool::validate_order,
    recovery::{recover, seed_filter},
    selection::attempt,
};
use crate::{
    Word, WordId,
    practice::{
        ExerciseVariant, NormalizedPracticeQuery, PracticeFilter, SeedDisposition, fnv1a64,
    },
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildTrace {
    pub recovery_draws: Vec<WordId>,
    pub recovery_fallback: bool,
    pub views: Vec<ViewAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round {
    pub content: RoundContent,
    pub context: PracticeFilter,
    pub selected_view: CandidateView,
    pub effective_offset: u64,
    pub seed: Option<WordId>,
    pub canonical_input: String,
    pub trace: BuildTrace,
}

pub struct RoundCandidates<'a> {
    pub initial: &'a CandidatePool,
    pub original_seed: Option<&'a Word>,
    pub recovery: &'a [Word],
    pub recovered_views: &'a [ContextPool],
}

pub fn build_round(
    candidates: RoundCandidates<'_>,
    variant: ExerciseVariant,
    query: &NormalizedPracticeQuery,
) -> Result<Round, BuildError> {
    let canonical_input = query.canonical_seed_input(variant);
    let mut rng = ChaCha12Rng::seed_from_u64(fnv1a64(&canonical_input));
    let mut trace = BuildTrace::default();
    let (seed, pool, context) = match query.seed {
        SeedDisposition::Unseeded => (None, candidates.initial, query.filter.clone()),
        SeedDisposition::Ready(id) => {
            let word = checked_seed(&candidates, id, query)?;
            if !eligible(word, variant) {
                return Err(BuildError::InvalidSeed { id });
            }
            let context = seed_filter(word).ok_or(BuildError::InvalidSeed { id })?;
            (Some(word), candidates.initial, context)
        }
        SeedDisposition::RecoveryRequired(id) => {
            checked_seed(&candidates, id, query)?;
            validate_order(candidates.recovery, CandidateView::AnyPos)?;
            if let Some(word) = candidates
                .recovery
                .iter()
                .find(|word| query.exclusions.contains(&word.id))
            {
                return Err(BuildError::InvalidPool {
                    view: CandidateView::AnyPos,
                    id: word.id,
                    reason: "excluded ID present in recovery pool",
                });
            }
            let (word, fallback) = recover(
                candidates.recovery,
                variant,
                &mut rng,
                &mut trace.recovery_draws,
            )?;
            trace.recovery_fallback = fallback;
            let context = seed_filter(word).ok_or(BuildError::InvalidSeed { id: word.id })?;
            let pool = candidates
                .recovered_views
                .iter()
                .find(|entry| {
                    entry.filter.level == context.level
                        && entry.filter.pos_tokens() == context.pos_tokens()
                })
                .map(|entry| &entry.pool)
                .ok_or_else(|| BuildError::MissingRecoveryViews {
                    filter: context.clone(),
                })?;
            (Some(word), pool, context)
        }
    };
    pool.validate(&query.exclusions)?;
    pool.validate_filter(&context)?;
    if let Some(seed) = seed
        && pool
            .strict
            .binary_search_by_key(&seed.id, |word| word.id)
            .ok()
            .is_none_or(|index| pool.strict[index] != *seed)
    {
        return Err(BuildError::InvalidSeed { id: seed.id });
    }
    for (view, words) in pool.views() {
        let (selection, attempt_trace) =
            attempt(words, view, variant, query.offset, seed, &mut rng);
        let effective_offset = attempt_trace.window_start.unwrap_or(0) as u64;
        trace.views.push(attempt_trace);
        if let Some(selection) = selection {
            return Ok(Round {
                content: shuffled(&selection.correct, &selection.reserves, variant, &mut rng),
                context,
                selected_view: view,
                effective_offset,
                seed: seed.map(|word| word.id),
                canonical_input,
                trace,
            });
        }
    }
    Err(BuildError::InsufficientPool {
        variant,
        attempts: trace.views,
    })
}

fn checked_seed<'a>(
    candidates: &RoundCandidates<'a>,
    id: WordId,
    query: &NormalizedPracticeQuery,
) -> Result<&'a Word, BuildError> {
    candidates
        .original_seed
        .filter(|word| word.id == id && !query.exclusions.contains(&id))
        .ok_or(BuildError::InvalidSeed { id })
}
