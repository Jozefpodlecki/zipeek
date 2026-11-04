mod page_objects;
mod suites;

use e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::test]
async fn root_scenarios() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = suites::run(&mut harness).await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
