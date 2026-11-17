use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{WebDriverActions, games},
};

async fn gallery_layout_and_entry(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games").await?;
    driver.query_single(games::PROVINCE_FEATURED_CARD).await?;
    let cards = driver
        .find_all_css("[data-component='games-gallery'] a")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        cards.len(),
        6,
        "First page must have a featured tile and five normal game tiles"
    );
    let feature = driver
        .query_single("[data-component='games-gallery'] a[href*='province-quest']")
        .await?;
    feature.click().await?;
    driver.query_single(games::PROVINCE_SETUP).await?;
    Ok(())
}

#[tokio::test]
async fn province_featured_gallery_card_opens_setup() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run(
            "province_featured_gallery_card_opens_setup",
            gallery_layout_and_entry,
        )
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
