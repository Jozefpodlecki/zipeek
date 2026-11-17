#[path = "suites/practice/fixed_mcq.rs"]
mod fixed_mcq_suite;
#[path = "suites/practice/matching.rs"]
mod matching_suite;
#[path = "suites/practice/mcq.rs"]
mod mcq_suite;
#[path = "suites/practice/mixed.rs"]
mod mixed_suite;
#[path = "suites/platform/pages.rs"]
mod pages_suite;
#[path = "suites/practice/practice.rs"]
mod practice_suite;
#[path = "suites/practice/setup.rs"]
mod setup_suite;
#[path = "suites/platform/styling.rs"]
mod styling_suite;

use zipeek_e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::test]
async fn pages_scenarios() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Pages).await?;
    harness
        .run(
            "quick_write_hanzi_summary",
            setup_suite::quick_write_hanzi_summary,
        )
        .await?;
    harness
        .run(
            "pages_dictionary_and_workers_use_the_zipeek_base_path",
            pages_suite::pages_dictionary_and_workers_use_the_zipeek_base_path,
        )
        .await?;
    harness
        .run(
            "shell_and_card_styling",
            styling_suite::shell_and_card_styling,
        )
        .await?;
    harness
        .run(
            "practice_rounds_normalize_and_replay",
            practice_suite::practice_rounds_normalize_and_replay,
        )
        .await?;
    harness
        .run(
            "late_queries_and_retry_keep_current_route",
            practice_suite::late_queries_and_retry_keep_current_route,
        )
        .await?;
    harness
        .run(
            "setup_progress_mode_and_responsive_layout",
            setup_suite::setup_progress_mode_and_responsive_layout,
        )
        .await?;
    harness
        .run(
            "setup_selection_guard_and_history",
            setup_suite::setup_selection_guard_and_history,
        )
        .await?;
    harness
        .run(
            "listening_choice_session_and_restore",
            setup_suite::listening_choice_session_and_restore,
        )
        .await?;
    harness
        .run(
            "sentence_builder_session_and_restore",
            setup_suite::sentence_builder_session_and_restore,
        )
        .await?;
    harness
        .run(
            "spot_mistake_identify_repair_and_restore",
            setup_suite::spot_mistake_identify_repair_and_restore,
        )
        .await?;
    harness
        .run(
            "authored_practice_continue_and_restore",
            setup_suite::authored_practice_continue_and_restore,
        )
        .await?;
    harness
        .run(
            "standalone_practice_playlist_selection",
            setup_suite::standalone_practice_playlist_selection,
        )
        .await?;
    harness
        .run(
            "endless_mcq_answers_lock_and_continue",
            mcq_suite::endless_mcq_answers_lock_and_continue,
        )
        .await?;
    harness
        .run(
            "endless_mcq_seed_wrap_and_responsive_layout",
            mcq_suite::endless_mcq_seed_wrap_and_responsive_layout,
        )
        .await?;
    harness
        .run(
            "fixed_mcq_counts_stats_and_restart",
            fixed_mcq_suite::fixed_mcq_counts_stats_and_restart,
        )
        .await?;
    harness
        .run(
            "fixed_summary_motion_layout_and_setup",
            fixed_mcq_suite::fixed_summary_motion_layout_and_setup,
        )
        .await?;
    harness
        .run(
            "matching_interaction_and_layout",
            matching_suite::matching_interaction_and_layout,
        )
        .await?;
    harness
        .run(
            "matching_fixed_summary_and_restart",
            matching_suite::matching_fixed_summary_and_restart,
        )
        .await?;
    harness
        .run(
            "matching_recovery_seed_and_same_url",
            matching_suite::matching_recovery_seed_and_same_url,
        )
        .await?;
    harness
        .run(
            "mixed_setup_progress_summary",
            mixed_suite::mixed_setup_progress_summary,
        )
        .await?;
    harness
        .run(
            "mixed_urls_replay_and_seed",
            mixed_suite::mixed_urls_replay_and_seed,
        )
        .await?;
    harness
        .run(
            "sentence_transformation_answer_and_restore",
            setup_suite::sentence_transformation_answer_and_restore,
        )
        .await?;
    harness
        .run(
            "audio_dictation_play_and_restore",
            setup_suite::audio_dictation_play_and_restore,
        )
        .await?;
    harness.finish().await
}
