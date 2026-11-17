use super::{WebDriverActions, selectors::practice};
use thirtyfour::prelude::*;

pub const QUICK_PATH: &str = "[data-practice-path='quick']";
pub const CONFIGURE_EXERCISES: &str = "[data-practice-exercise-stage='configure']";
pub const CHARACTER_BUILDER_TAB: &str = "[data-practice-config-tab='character-builder']";
pub const TYPE_PINYIN_TAB: &str = "[data-practice-config-tab='type-pinyin']";
pub const RETRY: &str = "[data-practice-retry-incorrect]";
pub const AUDIO: &str = "[data-practice-show-audio]";

pub async fn toggle_exercise(driver: &WebDriver, id: &str) -> WebDriverResult<()> {
    driver.click(&practice::exercise(id)).await
}

pub async fn select_tab(driver: &WebDriver, selector: &str) -> WebDriverResult<()> {
    driver.click(selector).await?;
    let selected_selector = format!("{selector}[aria-selected='true']");
    driver
        .query_named(selected_selector.as_str(), "Practice Config - select tab")
        .await?;
    Ok(())
}

pub async fn set_checkbox(
    driver: &WebDriver,
    selector: &str,
    enabled: bool,
) -> WebDriverResult<()> {
    let input = driver
        .query_named(selector, "Practice Config - set checkbox")
        .await?;
    if input.is_selected().await? != enabled {
        input.click().await?;
    }
    let settled = if enabled {
        format!("{selector}:checked")
    } else {
        format!("{selector}:not(:checked)")
    };
    driver
        .query_named(settled, "Practice answer option settled")
        .await?;
    Ok(())
}

pub async fn checkbox_checked(driver: &WebDriver, selector: &str) -> WebDriverResult<bool> {
    driver
        .query_named(selector, "Practice Config - checkbox checked")
        .await?
        .is_selected()
        .await
}

pub async fn selected_exercises(driver: &WebDriver) -> crate::HarnessResult<Vec<String>> {
    let mut selected = Vec::new();
    for element in driver.find_all_css(practice::SELECTED_EXERCISES).await? {
        if let Some(exercise) = element.attr("data-practice-exercise").await? {
            selected.push(exercise);
        }
    }
    selected.sort();
    if selected.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("Practice setup exposed duplicate selected exercise IDs".into());
    }
    Ok(selected)
}
