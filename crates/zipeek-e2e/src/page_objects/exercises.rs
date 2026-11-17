use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const HUNT_PAGE: &str = "[data-component='character-hunt']";
pub const HUNT_SESSION: &str = "[data-hunt-session]";
pub const HUNT_HINT: &str = "[data-hunt-hint][data-action='hint']";
pub const HUNT_FINISHED: &str =
    "[data-hunt-next], [data-hunt-new-practice], [data-hunt-session-continue]";
pub const IMAGE: &str = "[data-image-vocabulary]";
pub const LISTENING: &str = "[data-listening-choice]";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageAnswerMode {
    OneOfThree,
    SelectAll,
}

impl ImageAnswerMode {
    pub fn action(self) -> &'static str {
        match self {
            Self::OneOfThree => "one-of-three",
            Self::SelectAll => "select-all",
        }
    }
}

pub async fn choose_image_mode(
    driver: &WebDriver,
    mode: ImageAnswerMode,
) -> crate::HarnessResult<()> {
    let action = mode.action();
    let button = driver
        .query_named(
            &format!("{IMAGE} button[data-action='{action}']"),
            "Exercises - choose image mode",
        )
        .await?;
    button.click().await?;
    for candidate in [ImageAnswerMode::OneOfThree, ImageAnswerMode::SelectAll] {
        let choice = driver
            .query_single(&format!(
                "{IMAGE} button[data-action='{}']",
                candidate.action()
            ))
            .await?;
        crate::e2e_ensure_eq!(
            choice.attr("aria-pressed").await?.as_deref(),
            Some(if candidate == mode { "true" } else { "false" }),
            "Image Vocabulary answer modes must be mutually exclusive"
        );
    }
    Ok(())
}

pub async fn hunt_dimension(driver: &WebDriver) -> crate::HarnessResult<usize> {
    let raw = driver
        .query_named("[data-hunt-grid-size]", "Exercises - hunt dimension")
        .await?
        .attr("data-hunt-grid-size")
        .await?
        .ok_or("Character Hunt grid size attribute is missing")?;
    let size = raw
        .parse::<usize>()
        .map_err(|error| format!("Invalid hunt grid size {raw:?}: {error}"))?;
    if size == 0 {
        return Err("Character Hunt returned an empty grid".into());
    }
    Ok(size)
}
