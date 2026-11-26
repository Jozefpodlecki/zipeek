#[cfg(test)]
mod hanzi_progress_tests {
    use super::super::progress::{HanziCompletionPreview, completion_percentage, preview_progress};

    #[test]
    fn completion_percentage_handles_empty_and_partial_sessions() {
        assert_eq!(completion_percentage(0, 0), 0);
        assert_eq!(completion_percentage(0, 5), 0);
        assert_eq!(completion_percentage(3, 5), 60);
        assert_eq!(completion_percentage(5, 5), 100);
        assert_eq!(completion_percentage(6, 5), 100);
    }

    #[test]
    fn previews_finished_hanzi_only_for_the_active_step() {
        let pending = HanziCompletionPreview {
            step_id: "step-2".to_owned(),
            ready: true,
        };
        assert_eq!(preview_progress(1, Some("step-2"), Some(&pending)), 2);
        assert_eq!(preview_progress(1, Some("step-3"), Some(&pending)), 1);
        assert_eq!(preview_progress(1, None, Some(&pending)), 1);
        let not_ready = HanziCompletionPreview {
            ready: false,
            ..pending
        };
        assert_eq!(preview_progress(1, Some("step-2"), Some(&not_ready)), 1);
        assert_eq!(preview_progress(1, Some("step-2"), None), 1);
    }
}
