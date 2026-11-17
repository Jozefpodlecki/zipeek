use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const TARGET: &str = "[data-component='speaking-target']";
pub const CHOICE: &str = "[data-action='select-speaking-target']";
pub const SELECTOR: &str = "[data-component='speaking-target-selector']";
pub const CONTINUE: &str = "[data-speaking-session-continue]";

pub async fn choose(driver: &WebDriver, index: usize) -> crate::HarnessResult<()> {
    let choices = driver.find_all_css(CHOICE).await?;
    let choice = choices.get(index).ok_or_else(|| {
        format!(
            "speaking choice {index} unavailable; {} choices rendered",
            choices.len()
        )
    })?;
    choice.click().await?;
    let selected = driver.find_all_css(CHOICE).await?;
    crate::e2e_ensure_eq!(
        selected.iter().count(),
        choices.len(),
        "Speaking targets must not disappear after selection"
    );
    crate::e2e_ensure_eq!(
        selected
            .get(index)
            .ok_or("Speaking target disappeared after selection")?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true"),
        "Speaking target must become selected"
    );
    let mut pressed = 0;
    for target in &selected {
        if target.attr("aria-pressed").await?.as_deref() == Some("true") {
            pressed += 1;
        }
    }
    crate::e2e_ensure_eq!(pressed, 1, "Only one speaking target may be selected");
    Ok(())
}

pub async fn instance_id(driver: &WebDriver) -> crate::HarnessResult<String> {
    driver
        .query_named(TARGET, "Speaking - instance id")
        .await?
        .attr("data-exercise-instance")
        .await?
        .ok_or_else(|| "Speaking target is missing its exercise instance ID".into())
}
