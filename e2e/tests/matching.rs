#[path = "suites/matching.rs"]
mod matching_suite;
use e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::test]
async fn matching_scenarios() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = async {
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
        Ok(())
    }
    .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
