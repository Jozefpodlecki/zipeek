use thirtyfour::prelude::*;
use zipeek_e2e::{
    HarnessResult, ScenarioContext,
    page_objects::{WebDriverActions, games},
};

pub async fn play(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/listening-navigator").await?;
    games::assert_game_vocabulary_picker(driver, "/games/listening-navigator").await?;
    games::assert_mini_game_source(driver, "Starter questions").await?;
    driver
        .query_single("[data-game-setup='Listening Navigator']")
        .await?;
    driver
        .query_single("[data-game-start]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-listening-navigator]").await?;
    driver
        .query_single("[data-navigator-direction='Right']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-game-progress]")
            .await?
            .text()
            .await?,
        "0 / 6"
    );
    games::assert_mini_game_mistakes(driver, 1).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.query_single("[role='status']").await?.text().await?,
        "Try listening again"
    );
    for (round, direction) in ["Left", "Up", "Right", "Right", "Down", "Left"]
        .into_iter()
        .enumerate()
    {
        driver
            .query_single(&format!("[data-navigator-direction='{direction}']"))
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
    games::assert_mini_game_progress(driver, 0, 6).await?;
    games::assert_mini_game_mistakes(driver, 0).await?;
    let positions = driver
        .find_all_css("[aria-label='City grid'] .bg-brand-action")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(positions.len(), 1);
    ctx.checkpoint("listening-navigator-play-and-finish")
        .await?;
    Ok(())
}
