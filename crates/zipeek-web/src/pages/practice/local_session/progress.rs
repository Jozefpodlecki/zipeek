#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HanziCompletionPreview {
    pub(super) step_id: String,
    pub(super) ready: bool,
}

pub(super) fn preview_progress(
    completed: usize,
    step_id: Option<&str>,
    preview: Option<&HanziCompletionPreview>,
) -> usize {
    completed
        + usize::from(step_id.is_some_and(|id| {
            preview.is_some_and(|pending| pending.step_id == id && pending.ready)
        }))
}

pub(super) fn completion_percentage(completed: usize, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        (completed.saturating_mul(100) / total).min(100)
    }
}
