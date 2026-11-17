use thirtyfour::prelude::*;
use zipeek_e2e::{
    HarnessResult, ScenarioContext,
    page_objects::{WebDriverActions, games},
};

pub async fn play(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/classifier-market").await?;
    games::assert_game_vocabulary_picker(driver, "/games/classifier-market").await?;
    games::assert_mini_game_source(driver, "Starter questions").await?;
    driver
        .query_single("[data-game-setup='Classifier Market']")
        .await?;
    driver
        .query_single("[data-game-start]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-classifier-market]").await?;
    driver
        .query_single("[data-classifier-choice='2']")
        .await?
        .click()
        .await?;
    games::assert_mini_game_mistakes(driver, 1).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.query_single("[role='status']").await?.text().await?,
        "Try another measure word"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-game-progress]")
            .await?
            .text()
            .await?,
        "0 / 6"
    );
    for (round, index) in [0, 1, 2, 0, 1, 2].into_iter().enumerate() {
        driver
            .query_single(&format!("[data-classifier-choice='{index}']"))
            .await?
            .click()
            .await?;
        games::assert_mini_game_progress(driver, round + 1, 6).await?;
    }
    driver.query_single("[data-game-complete]").await?;
    driver
        .query(By::XPath("//button[normalize-space()='Play again']"))
        .single()
        .await?
        .click()
        .await?;
    driver.query_single("[data-classifier-market]").await?;
    games::assert_mini_game_mistakes(driver, 0).await?;
    games::assert_mini_game_progress(driver, 0, 6).await?;
    zipeek_e2e::e2e_ensure_eq!(driver.find_all_css("[role='status']").await?.len(), 0);
    ctx.checkpoint("classifier-market-play-and-restart").await?;
    Ok(())
}
