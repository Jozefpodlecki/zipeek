use zipeek_e2e::{
    HarnessResult, ScenarioContext,
    page_objects::{WebDriverActions, games},
};

pub async fn play(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/sentence-relay").await?;
    games::assert_game_vocabulary_picker(driver, "/games/sentence-relay").await?;
    games::assert_mini_game_source(driver, "Starter questions").await?;
    driver
        .query_single("[data-game-setup='Sentence Relay']")
        .await?;
    let start = driver.query_single("[data-game-start]").await?;
    zipeek_e2e::e2e_ensure!(start.is_enabled().await?);
    ctx.checkpoint("sentence-relay-ai-setup").await?;
    Ok(())
}
