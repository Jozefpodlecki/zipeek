use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const ROOT: &str = "[data-component='translate-sentence-page']";
pub const DIRECTION: &str = "[data-translation-direction]";
pub const INPUT: &str = "[data-translation-source]";
pub const SUBMIT: &str = "[data-translate-submit]";
pub const CLEAR: &str = "button[aria-label='Clear sentence']";

pub async fn choose_direction(driver: &WebDriver, index: usize) -> crate::HarnessResult<()> {
    let choices = driver.find_all_css(DIRECTION).await?;
    let choice = choices.get(index).ok_or_else(|| {
        format!(
            "translator choice {index} unavailable; {} choices rendered",
            choices.len()
        )
    })?;
    choice.click().await?;
    let selected = driver
        .find_all_css(&format!("{DIRECTION}[aria-selected='true']"))
        .await?;
    crate::e2e_ensure_eq!(
        selected.len(),
        1,
        "Exactly one translation direction must be selected"
    );
    crate::e2e_ensure_eq!(
        driver
            .find_all_css(DIRECTION)
            .await?
            .get(index)
            .ok_or("Translation direction disappeared after selection")?
            .attr("aria-selected")
            .await?
            .as_deref(),
        Some("true"),
        "The requested translation direction must be selected"
    );
    Ok(())
}

pub async fn source_text(driver: &WebDriver) -> WebDriverResult<String> {
    Ok(driver
        .query_named(INPUT, "Translator - source text")
        .await?
        .prop("value")
        .await?
        .unwrap_or_default())
}
