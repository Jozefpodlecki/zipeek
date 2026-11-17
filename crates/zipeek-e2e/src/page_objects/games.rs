use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const CLOUD: &str = "[data-component='cloud-match']";
pub const CLOUD_BACK: &str = "a[aria-label='All games']";
pub const CROSSWORD_START: &str = "[data-testid='crossword-start']";
pub const CROSSWORD_BANDS: &str = "[data-crossword-band]";

pub async fn choose_crossword_band(
    driver: &WebDriver,
    standard: u8,
    level: u8,
) -> WebDriverResult<()> {
    driver
        .query_named(
            &format!("[data-crossword-band='{standard}-{level}']"),
            "Games - choose crossword band",
        )
        .await?
        .click()
        .await
}

pub async fn choose_cloud_difficulty(driver: &WebDriver, label: &str) -> WebDriverResult<()> {
    driver
        .query(By::XPath(&format!(
            "//main[@data-component='cloud-match']//button[normalize-space()='{label}']"
        )))
        .desc("Games - choose cloud difficulty")
        .single()
        .await?
        .click()
        .await
}

pub const MINI_GAME_PROGRESS: &str = "[data-game-progress]";
pub const MINI_GAME_MISTAKES: &str = "[data-game-mistakes]";
pub const MINI_GAME_COMPLETE: &str = "[data-game-complete]";

pub async fn assert_mini_game_progress(
    driver: &WebDriver,
    completed: usize,
    total: usize,
) -> crate::HarnessResult<()> {
    let visible = driver
        .query_single(MINI_GAME_PROGRESS)
        .await?
        .text()
        .await?;
    crate::e2e_ensure_eq!(
        visible,
        format!("{completed} / {total}"),
        "Game progress must match accepted rounds"
    );
    Ok(())
}

pub async fn assert_mini_game_mistakes(
    driver: &WebDriver,
    mistakes: u32,
) -> crate::HarnessResult<()> {
    let visible = driver
        .query_single(MINI_GAME_MISTAKES)
        .await?
        .text()
        .await?;
    crate::e2e_ensure_eq!(
        visible,
        format!("{mistakes} mistakes"),
        "Invalid choices must count exactly once"
    );
    Ok(())
}

pub const MINI_GAME_VOCABULARY: &str = "[aria-label='Game vocabulary']";

pub async fn assert_game_vocabulary_picker(
    driver: &WebDriver,
    path: &str,
) -> crate::HarnessResult<()> {
    driver.query_single(MINI_GAME_VOCABULARY).await?;
    let starter = driver.query_single(&format!("a[href='#{}']", path)).await?;
    crate::e2e_ensure_eq!(starter.text().await?, "Starter");
    driver
        .query_single("[data-vocabulary-hsk] summary")
        .await?
        .click()
        .await?;
    let level = driver
        .query_single(&format!(
            "a[href='#{}?source=hsk&standard=3&level=1']",
            path
        ))
        .await?;
    crate::e2e_ensure_eq!(level.text().await?, "HSK 3.0 · 1");
    Ok(())
}

pub async fn assert_mini_game_source(driver: &WebDriver, name: &str) -> crate::HarnessResult<()> {
    let visible = driver
        .query_single("[data-game-source]")
        .await?
        .text()
        .await?;
    crate::e2e_ensure_eq!(
        visible,
        name,
        "Game must identify the active vocabulary source"
    );
    Ok(())
}

pub const PROVINCE_FEATURED_CARD: &str = "[data-game-featured='Province Quest']";
pub const SENTENCE_FEATURED_CARD: &str = "[data-game-featured='Sentence Relay']";
pub const PROVINCE_SETUP: &str = "[data-game-setup='Province Quest']";
pub const PROVINCE_MAP: &str = "[data-province-map]";
pub const PROVINCE_PROMPT: &str = "[data-province-prompt]";
pub const PROVINCE_REVIEW: &str = "[data-province-review]";
