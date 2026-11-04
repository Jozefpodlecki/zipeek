#[path = "suites/mixed.rs"]
mod mixed_suite;
use e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::test]
async fn mixed_scenarios() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = async {
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
        Ok(())
    }
    .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
