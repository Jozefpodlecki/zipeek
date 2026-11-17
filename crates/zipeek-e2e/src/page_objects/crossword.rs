use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const SETUP: &str = "[data-crossword-setup]";
const START: &str = "[data-testid='crossword-start']";
const ENABLED_START: &str = "[data-testid='crossword-start']:not([disabled])";
const DEFAULT_LEVEL: &str = "[data-crossword-band='3-1']";
const CELL: &str = "[data-crossword-board] [data-crossword-cell]";
pub const TILE: &str = "[data-crossword-tile]";
pub const SELECTED_TILE: &str = "[data-crossword-tile][aria-pressed='true']";
const ENABLED_NEW_GAME: &str = "[data-testid='crossword-new-game']:not([disabled])";
const DISABLED_NEW_GAME: &str = "[data-testid='crossword-new-game'][disabled]";
const SCORE: &str = "[data-testid='crossword-score']";

pub async fn open_setup(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query_named(SETUP, "Crossword - open setup").await
}

pub async fn start(driver: &WebDriver) -> WebDriverResult<()> {
    let start = driver.find_css(START).await?;
    if !start.is_enabled().await? {
        driver.click(DEFAULT_LEVEL).await?;
    }
    driver
        .query_named(ENABLED_START, "Crossword - start")
        .await?
        .click()
        .await?;
    ready(driver).await?;
    Ok(())
}

pub async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query_first_named(CELL, "Crossword - ready").await
}

pub async fn new_game(driver: &WebDriver) -> WebDriverResult<()> {
    driver.click(ENABLED_NEW_GAME).await?;
    driver
        .query_named(DISABLED_NEW_GAME, "Crossword - new game disabled")
        .await?;
    driver
        .query_named(ENABLED_NEW_GAME, "Crossword - new game enabled")
        .await?;
    ready(driver).await?;
    Ok(())
}

pub async fn score(driver: &WebDriver) -> WebDriverResult<String> {
    driver.find_css(SCORE).await?.text().await
}
