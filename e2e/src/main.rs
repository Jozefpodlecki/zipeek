use e2e::{Harness, HarnessResult, SuiteKind};
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("landing-brand", |context| async move {
            let heading = context
                .driver
                .query(By::Css("header a[href='#/']"))
                .single()
                .await?;
            assert_eq!(heading.text().await?, "Zipeek");
            Ok(())
        })
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
