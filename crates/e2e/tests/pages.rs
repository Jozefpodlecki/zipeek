#[path = "suites/fixed_mcq.rs"]
mod fixed_mcq_suite;
#[path = "suites/matching.rs"]
mod matching_suite;
#[path = "suites/mcq.rs"]
mod mcq_suite;
#[path = "suites/mixed.rs"]
mod mixed_suite;
#[path = "suites/pages.rs"]
mod pages_suite;
#[path = "suites/practice.rs"]
mod practice_suite;
#[path = "suites/setup.rs"]
mod setup_suite;
#[path = "page_objects/shell.rs"]
mod shell_page;
#[path = "suites/styling.rs"]
mod styling_suite;

use e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::test]
async fn pages_scenarios() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Pages).await?;
    let result = harness
        .run(
            "pages_dictionary_and_workers_use_the_zipeek_base_path",
            pages_suite::pages_dictionary_and_workers_use_the_zipeek_base_path,
        )
        .await;
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "shell_and_card_styling",
                    styling_suite::shell_and_card_styling,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "practice_rounds_normalize_and_replay",
                    practice_suite::practice_rounds_normalize_and_replay,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "late_queries_and_retry_keep_current_route",
                    practice_suite::late_queries_and_retry_keep_current_route,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "setup_controls_generate_all_six_practice_urls",
                    setup_suite::setup_controls_generate_all_six_practice_urls,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "setup_exclusion_failure_retry_and_reset_error",
                    setup_suite::setup_exclusion_failure_retry_and_reset_error,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "endless_mcq_answers_lock_and_continue",
                    mcq_suite::endless_mcq_answers_lock_and_continue,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "endless_mcq_seed_wrap_and_responsive_layout",
                    mcq_suite::endless_mcq_seed_wrap_and_responsive_layout,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "fixed_mcq_counts_stats_and_restart",
                    fixed_mcq_suite::fixed_mcq_counts_stats_and_restart,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "fixed_summary_motion_layout_and_setup",
                    fixed_mcq_suite::fixed_summary_motion_layout_and_setup,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "matching_interaction_and_layout",
                    matching_suite::matching_interaction_and_layout,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "matching_fixed_summary_and_restart",
                    matching_suite::matching_fixed_summary_and_restart,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "matching_recovery_seed_and_same_url",
                    matching_suite::matching_recovery_seed_and_same_url,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "mixed_setup_progress_summary",
                    mixed_suite::mixed_setup_progress_summary,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(()) => {
            harness
                .run(
                    "mixed_urls_replay_and_seed",
                    mixed_suite::mixed_urls_replay_and_seed,
                )
                .await
        }
        Err(error) => Err(error),
    };
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
