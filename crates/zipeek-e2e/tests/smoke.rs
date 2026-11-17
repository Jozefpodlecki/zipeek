mod suites;

use zipeek_e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::test]
async fn root_scenarios() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = suites::run(&mut harness).await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}

#[tokio::test]
async fn deterministic_ai_practice() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = suites::run_ai_fixture(&mut harness).await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
