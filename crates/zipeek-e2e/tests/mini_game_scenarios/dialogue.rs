use zipeek_e2e::{HarnessResult, ScenarioContext, page_objects::WebDriverActions};

pub async fn setup(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/dialogue-quest").await?;
    driver
        .query_single("[data-game-setup='Dialogue Quest']")
        .await?;
    driver.query_single("[data-game-source]").await?;
    let start = driver.query_single("[data-game-start]").await?;
    zipeek_e2e::e2e_ensure!(start.is_enabled().await?);
    ctx.checkpoint("dialogue-quest-ai-setup").await?;
    Ok(())
}
