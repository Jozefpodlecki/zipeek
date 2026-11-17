use zipeek_e2e::{HarnessResult, ScenarioContext, page_objects::WebDriverActions};

pub async fn play(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/radical-forge").await?;
    driver
        .query_single("[data-game-setup='Radical Forge']")
        .await?;
    driver
        .query_single("[data-game-start]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-radical-forge]").await?;
    for answer in [0, 2, 0, 1, 0] {
        driver
            .query_single(&format!("[data-radical-choice='{answer}']"))
            .await?
            .click()
            .await?;
    }
    driver.query_single("[data-game-complete]").await?;
    ctx.checkpoint("radical-forge-complete").await?;
    Ok(())
}
