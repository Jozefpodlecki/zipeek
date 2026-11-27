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
    fn reviewed_character_builder_previews_only_the_current_step() {
        let reviewed = HanziCompletionPreview {
            step_id: "builder:0".to_owned(),
            ready: true,
        };
        assert_eq!(preview_progress(0, Some("builder:0"), Some(&reviewed)), 1);
        assert_eq!(preview_progress(0, Some("builder:1"), Some(&reviewed)), 0);
        assert_eq!(preview_progress(1, None, Some(&reviewed)), 1);
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

#[cfg(test)]
mod stale_exercise_callback_tests {
    use super::super::persistence::is_current_step;
    use crate::services::practice_session_store::{ExerciseOutcome, PracticeSession};

    #[test]
    fn late_result_cannot_target_the_following_step() {
        let mut saved = PracticeSession::new(
            uuid::Uuid::now_v7().to_string(),
            &["fill-blank".to_owned()].into_iter().collect(),
            2,
            42,
            Default::default(),
        )
        .expect("valid practice session");
        let original_id = saved.current_step().unwrap().id.clone();
        assert!(is_current_step(&saved, &original_id));
        saved
            .complete(&original_id, ExerciseOutcome::Completed)
            .expect("complete first step");
        assert!(!is_current_step(&saved, &original_id));
        let next_id = saved.current_step().unwrap().id.clone();
        assert!(is_current_step(&saved, &next_id));
    }
}
