use super::{KnowledgeEntityId, KnowledgeState, ScoringPolicy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DimensionScore {
    pub earned_chunks: u16,
    pub required_chunks: u16,
    pub dimension_count: u16,
}

impl DimensionScore {
    pub fn as_f64(self) -> f64 {
        f64::from(self.earned_chunks)
            / (f64::from(self.dimension_count) * f64::from(self.required_chunks))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnowledgeScore {
    pub earned_units: u32,
    pub maximum_units: u32,
}

impl KnowledgeScore {
    pub fn as_f64(self) -> f64 {
        f64::from(self.earned_units) / f64::from(self.maximum_units)
    }
}

impl KnowledgeState {
    pub fn score(&self, policy: ScoringPolicy) -> DimensionScore {
        DimensionScore {
            earned_chunks: self.chunks,
            required_chunks: policy.required_days,
            dimension_count: self.entity.category.dimensions().len() as u16,
        }
    }
}

pub fn score_entity(states: &[KnowledgeState], policy: ScoringPolicy) -> Option<KnowledgeScore> {
    policy.validate().ok()?;
    let first = states.first()?;
    let dimensions = first.entity.category.dimensions();
    if states.len() != dimensions.len()
        || states.iter().any(|state| {
            state.entity != first.entity
                || state.policy_epoch != first.policy_epoch
                || !dimensions.contains(&state.dimension)
                || state.chunks > policy.required_days
        })
        || dimensions.iter().any(|dimension| {
            states
                .iter()
                .filter(|state| state.dimension == *dimension)
                .count()
                != 1
        })
    {
        return None;
    }
    Some(KnowledgeScore {
        earned_units: states.iter().map(|state| u32::from(state.chunks)).sum(),
        maximum_units: u32::from(policy.required_days) * dimensions.len() as u32,
    })
}

pub fn score_catalogue<'a>(
    entities: impl IntoIterator<Item = (KnowledgeEntityId, &'a [KnowledgeState])>,
    policy: ScoringPolicy,
) -> Option<KnowledgeScore> {
    policy.validate().ok()?;
    let mut keys = std::collections::BTreeSet::new();
    let mut earned: u64 = 0;
    let mut maximum: u64 = 0;
    for (id, states) in entities {
        if !keys.insert(id.clone()) || states.first().is_none_or(|state| state.entity != id) {
            return None;
        }
        let score = score_entity(states, policy)?;
        earned += u64::from(score.earned_units) * (12 / id.category.dimensions().len() as u64);
        maximum += u64::from(score.maximum_units) * (12 / id.category.dimensions().len() as u64);
    }
    Some(KnowledgeScore {
        earned_units: u32::try_from(earned).ok()?,
        maximum_units: u32::try_from(maximum).ok()?,
    })
}
