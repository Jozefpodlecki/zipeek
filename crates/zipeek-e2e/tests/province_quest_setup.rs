use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{WebDriverActions, games},
};

async fn setup_modes_are_independent(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/province-quest").await?;
    driver.query_single(games::PROVINCE_SETUP).await?;
    let buttons = driver
        .find_all_css("[data-game-setup='Province Quest'] button[aria-pressed]")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        buttons.len(),
        6,
        "Three difficulty modes and three learning-support modes must be selectable"
    );
    buttons[2].click().await?;
    buttons[4].click().await?;
    zipeek_e2e::e2e_ensure_eq!(
        buttons[2].attr("aria-pressed").await?.as_deref(),
        Some("true")
    );
    zipeek_e2e::e2e_ensure_eq!(
        buttons[4].attr("aria-pressed").await?.as_deref(),
        Some("true")
    );
    zipeek_e2e::e2e_ensure_eq!(
        buttons[0].attr("aria-pressed").await?.as_deref(),
        Some("false")
    );
    Ok(())
}

#[tokio::test]
async fn province_difficulty_and_reading_support_are_separate() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run(
            "province_difficulty_and_reading_support_are_separate",
            setup_modes_are_independent,
        )
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
