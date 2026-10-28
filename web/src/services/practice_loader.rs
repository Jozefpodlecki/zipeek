use super::database::{Database, DbError};
use std::collections::BTreeMap;
use thiserror::Error;
use zipeek_core::{
    HskLevel, Word,
    practice::{
        ExerciseVariant, NormalizedPracticeQuery, ParsedPracticeQuery, PracticeFilter,
        PracticeMode, SeedDisposition,
    },
    round::{BuildError, ContextPool, Round, RoundCandidates, build_round},
};

#[derive(Debug, Error)]
pub enum PracticeLoadError {
    #[error(transparent)]
    Database(#[from] DbError),
    #[error(transparent)]
    Build(#[from] BuildError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedPractice {
    pub query: NormalizedPracticeQuery,
    pub round: Round,
}

pub async fn prepare_practice(
    database: &Database,
    variant: ExerciseVariant,
    parsed: &ParsedPracticeQuery,
) -> Result<PreparedPractice, PracticeLoadError> {
    let mut ids = parsed.exclusions.clone();
    if !matches!(parsed.mode, PracticeMode::Fixed(_))
        && let Some(id) = parsed.seed
    {
        ids.insert(id);
    }
    let rows = database.words_by_ids(&ids).await?;
    let rows = rows
        .iter()
        .map(|word| (word.id, word))
        .collect::<BTreeMap<_, _>>();
    let query = parsed.normalize(variant, |id| rows.get(&id).copied());
    let original_seed = query
        .seed
        .original_id()
        .and_then(|id| rows.get(&id).copied());
    let initial_filter = match query.seed {
        SeedDisposition::Ready(id) => original_seed
            .and_then(seed_context)
            .ok_or(BuildError::InvalidSeed { id })?,
        _ => query.filter.clone(),
    };
    let initial = database
        .candidate_pool(&initial_filter, &query.exclusions)
        .await?;
    let mut recovery = Vec::new();
    let mut recovered_views = Vec::new();
    if matches!(query.seed, SeedDisposition::RecoveryRequired(_)) {
        recovery = database.hsk_union(&query.exclusions).await?;
        let mut contexts = BTreeMap::new();
        for word in &recovery {
            if let Some(filter) = seed_context(word) {
                contexts
                    .entry((
                        filter.level.system() as u8,
                        filter.level.as_i32(),
                        filter.pos_tokens(),
                    ))
                    .or_insert(filter);
            }
        }
        for filter in contexts.into_values() {
            let pool = database.candidate_pool(&filter, &query.exclusions).await?;
            recovered_views.push(ContextPool { filter, pool });
        }
    }
    let round = build_round(
        RoundCandidates {
            initial: &initial,
            original_seed,
            recovery: &recovery,
            recovered_views: &recovered_views,
        },
        variant,
        &query,
    )?;
    Ok(PreparedPractice { query, round })
}

fn seed_context(word: &Word) -> Option<PracticeFilter> {
    let level = word
        .hsk3_level
        .map(HskLevel::Hsk3)
        .or_else(|| word.hsk2_level.map(HskLevel::Hsk2))?;
    let mut part_of_speech = word.part_of_speech.clone();
    part_of_speech.sort_by_key(|pos| pos.as_str());
    part_of_speech.dedup();
    Some(PracticeFilter {
        level,
        part_of_speech,
    })
}
