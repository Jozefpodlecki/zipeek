use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const PAGE: &str = "[data-component='write-hanzi-page']";
pub const SOURCES: &str = "[aria-label='Vocabulary source'] button";
pub const ACTIVE_SOURCE: &str = "[aria-label='Vocabulary source'] button[aria-pressed='true']";
pub const SEARCH: &str = "#write-hanzi-search";

pub async fn select_source(driver: &WebDriver, index: usize) -> crate::HarnessResult<()> {
    let choices = driver.find_all_css(SOURCES).await?;
    let choice = choices.get(index).ok_or_else(|| {
        format!(
            "writing choice {index} unavailable; {} choices rendered",
            choices.len()
        )
    })?;
    choice.click().await?;
    let selected = driver.find_all_css(ACTIVE_SOURCE).await?;
    crate::e2e_ensure_eq!(
        selected.len(),
        1,
        "Exactly one writing vocabulary source must be active"
    );
    crate::e2e_ensure_eq!(
        driver
            .find_all_css(SOURCES)
            .await?
            .get(index)
            .ok_or("Selected writing source disappeared")?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true"),
        "Requested writing source must become active"
    );
    Ok(())
}

pub async fn selected_count(driver: &WebDriver) -> WebDriverResult<usize> {
    Ok(driver.find_all_css(ACTIVE_SOURCE).await?.len())
}

pub const HANZI_TRAINING: &str = "[data-component='hanzi-writing-page']";
pub const HANZI_ASSESSMENT_RESOLVED: &str =
    "[data-hanzi-assessment='ready'], [data-hanzi-assessment='unavailable']";
pub const HANZI_NEXT_STROKE: &str = "[data-hanzi-next-stroke-start]";
pub const HANZI_TRAINING_STATUS: &str = "[data-hanzi-training-status]";

pub const HANZI_CANVAS: &str = "[data-hanzi-canvas]";
pub const HANZI_GUIDE_TOGGLE: &str = "[data-action='toggle-guide']";
pub const HANZI_GUIDE: &str = "[data-hanzi-direction-guide]";
pub const HANZI_RESET: &str = "[data-action='reset-character']";
pub const HANZI_UNDO: &str = "[data-action='undo-stroke']";
pub const HANZI_CHARACTER_CHOICE: &str = "[data-action='select-hanzi']";
pub const HANZI_FEEDBACK: &str = "[data-hanzi-feedback='rejected']";
pub const HANZI_REPLAY: &str = "[data-action='replay-hanzi-strokes']";
pub const HANZI_RETRY: &str = "[data-action='retry-hanzi-reference']";

pub async fn assessment_phase(driver: &WebDriver) -> crate::HarnessResult<String> {
    let resolved = driver
        .query_wait_first(
            HANZI_ASSESSMENT_RESOLVED,
            std::time::Duration::from_secs(15),
            std::time::Duration::from_millis(200),
        )
        .await?;
    Ok(resolved
        .attr("data-hanzi-assessment")
        .await?
        .unwrap_or_default())
}

pub async fn wait_for_replay_to_finish(driver: &WebDriver) -> crate::HarnessResult<()> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(40);
    loop {
        if !driver.is_present("[data-hanzi-replay]").await? {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err("Hanzi replay did not finish within 40 seconds".into());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
