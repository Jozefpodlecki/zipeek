use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionKey {
    pub variant: ExerciseVariant,
    pub query: NormalizedPracticeQuery,
    pub types: Vec<ExerciseVariant>,
}
impl SessionKey {
    pub fn new(
        variant: ExerciseVariant,
        mut query: NormalizedPracticeQuery,
        types: Vec<ExerciseVariant>,
    ) -> Self {
        query.warnings.clear();
        Self {
            variant,
            query,
            types,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SessionStats {
    pub correct: u64,
    pub first_attempt: u64,
    pub mistakes: u64,
    pub streak: u64,
    pub highest_streak: u64,
    pub progress: Option<FixedProgress>,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub looped: bool,
}
