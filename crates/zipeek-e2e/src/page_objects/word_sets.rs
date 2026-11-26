use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

const NAME: &str = "[data-new-word-set]";
const CREATE: &str = "[data-create-word-set]";
const SELECTED: &str =
    "[data-target-word-set][data-selected-word-set-id]:not([data-selected-word-set-id=''])";

pub async fn create(driver: &WebDriver, name: &str) -> crate::HarnessResult<()> {
    let input = driver.query_named(NAME, "Word Sets - create").await?;
    input.clear().await?;
    input.send_keys(name).await?;
    driver.find_css(CREATE).await?.click().await?;
    driver
        .query_first_named(SELECTED, "Word Sets - create")
        .await?;
    crate::e2e_ensure!(
        persisted_set_names(driver)
            .await?
            .iter()
            .any(|saved| saved == name),
        "New Word Set {name:?} did not persist"
    );
    Ok(())
}

pub async fn selected_set_label(driver: &WebDriver) -> WebDriverResult<String> {
    driver
        .query_named("[data-target-word-set]", "Word Sets - selected set label")
        .await?
        .text()
        .await
}

pub async fn persisted_set_names(driver: &WebDriver) -> crate::HarnessResult<Vec<String>> {
    let sets = crate::page_objects::browser::saved_word_sets(driver).await?;
    sets.iter()
        .map(|set| {
            set.get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| "Saved Word Set is missing a name".into())
        })
        .collect()
}
