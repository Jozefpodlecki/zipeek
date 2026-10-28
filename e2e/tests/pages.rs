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
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
