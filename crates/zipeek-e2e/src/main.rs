use zipeek_e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::main]
async fn main() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("landing-brand", |context| async move {
            let brand = zipeek_e2e::page_objects::home::brand(&context.driver).await?;
            assert_eq!(brand.text().await?, "Zipeek");
            assert_eq!(brand.attr("href").await?.as_deref(), Some("#/"));
            Ok(())
        })
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
