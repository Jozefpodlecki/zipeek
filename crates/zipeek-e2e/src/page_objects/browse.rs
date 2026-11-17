use crate::page_objects::{WebDriverActions, WebElementQueryActions};
use std::time::{Duration, Instant};

use crate::HarnessResult;
use thirtyfour::prelude::*;
use tokio::time::sleep;

pub const RESULTS: &str = "[data-browse-results]";
pub const WORD_CARD: &str = "[data-component='browse-result']";
pub const ERROR: &str = "[data-browse-error]";
pub const QUERY: &str = "[data-browse-query]";

pub async fn ready(driver: &WebDriver) -> WebDriverResult<()> {
    driver.query_named(RESULTS, "Browse - ready").await?;
    Ok(())
}

pub async fn assert_healthy(driver: &WebDriver) -> HarnessResult<()> {
    if let Some(error) = driver.find_all_css(ERROR).await?.into_iter().next() {
        return Err(format!(
            "Browse returned a dictionary query error: {}",
            error.text().await?
        )
        .into());
    }
    Ok(())
}

pub async fn word_card(driver: &WebDriver, simplified: &str) -> HarnessResult<WebElement> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        assert_healthy(driver).await?;
        let cards = driver.find_all_css(WORD_CARD).await?;
        for card in &cards {
            let headword = card.find_css(HEADWORD).await?;
            if headword.text().await? == simplified {
                return Ok(card.clone());
            }
        }
        if let Some(empty) = driver.find_all_css(EMPTY).await?.into_iter().next() {
            return Err(format!(
                "Browse returned no results while searching for {simplified:?}: {}",
                empty.text().await?
            )
            .into());
        }
        if Instant::now() >= deadline {
            let loading = !driver
                .find_all_css("[data-browse-loading]")
                .await?
                .is_empty();
            let mut headwords = Vec::new();
            for headword in driver.find_all_css(HEADWORD).await?.into_iter().take(8) {
                headwords.push(headword.text().await?);
            }
            return Err(format!(
                "Browse did not display exact headword {simplified:?} within 30 seconds (loading={loading}, result_count={}, first_headwords={headwords:?})",
                cards.len()
            ).into());
        }
        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn assert_word_visible(driver: &WebDriver, simplified: &str) -> HarnessResult<()> {
    word_card(driver, simplified).await.map(|_| ())
}

pub const SAVE_BUTTON: &str = "[data-toggle-saved-word]";
pub const SAVE_ACTIVE: &str = "[data-toggle-saved-word][aria-pressed='true']";
pub const SAVE_INACTIVE: &str = "[data-toggle-saved-word][aria-pressed='false']";
pub const CHOOSE_SET: &str = "[data-choose-word-set] a";
pub const LISTEN: &str = "[data-listen]";
pub const EXPLORE_MEANINGS: &str = "[data-toggle-definitions]";
pub const SENSE_PANEL: &str = "[data-browse-sense-panel]";
pub const SENSE_COUNTER: &str = "[data-meaning-counter]";

pub async fn open_meanings(card: &WebElement) -> WebDriverResult<WebElement> {
    card.query_named(EXPLORE_MEANINGS, "Browse - expand meanings")
        .await?
        .click()
        .await?;
    card.query_named(SENSE_PANEL, "Browse - sense panel").await
}

pub async fn first_word(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query_named(WORD_CARD, "Browse - first word").await
}

pub async fn save_button(card: &WebElement) -> WebDriverResult<WebElement> {
    card.query_named(SAVE_BUTTON, "Browse - save button").await
}

pub const EMPTY: &str = "[data-browse-empty]";
pub const PAGE: &str = "[data-browse-page]";
pub const NEXT: &str = "[data-browse-next]";
pub const PREVIOUS: &str = "[data-browse-previous]";
pub const HEADWORD: &str = "[data-browse-headword]";

pub async fn visible_word_ids(driver: &WebDriver) -> HarnessResult<Vec<i32>> {
    let cards = driver.find_all_css(WORD_CARD).await?;
    let mut ids = Vec::with_capacity(cards.len());
    for card in cards {
        let raw = card
            .attr("data-browse-word-id")
            .await?
            .ok_or("Browse result is missing its word ID")?;
        let id = raw
            .parse::<i32>()
            .map_err(|error| format!("Invalid Browse word ID {raw:?}: {error}"))?;
        if ids.contains(&id) {
            return Err(format!("Browse rendered word ID {id} more than once on one page").into());
        }
        ids.push(id);
    }
    Ok(ids)
}

pub async fn wait_for_page(driver: &WebDriver, expected_page: u32) -> HarnessResult<()> {
    let expected_prefix = format!("Page {expected_page} ·");
    let deadline = Instant::now() + Duration::from_secs(30);

    loop {
        if let Some(error) = driver.find_all_css(ERROR).await?.into_iter().next() {
            let message = error.text().await?;
            return Err(format!(
                "Browse failed while navigating to page {expected_page}: {message}"
            )
            .into());
        }

        if let Some(empty) = driver.find_all_css(EMPTY).await?.into_iter().next() {
            let message = empty.text().await?;
            return Err(format!(
                "Browse unexpectedly returned no results on page {expected_page}: {message}"
            )
            .into());
        }

        if let Some(pagination) = driver.find_all_css(PAGE).await?.into_iter().next() {
            if pagination.text().await?.starts_with(&expected_prefix) {
                return Ok(());
            }
        }

        if Instant::now() >= deadline {
            let loading = !driver
                .find_all_css("[data-browse-loading]")
                .await?
                .is_empty();
            let current_page = match driver.find_all_css(PAGE).await?.into_iter().next() {
                Some(pagination) => pagination.text().await?,
                None => "not rendered".to_owned(),
            };
            return Err(format!(
                "Browse did not reach page {expected_page} within 30 seconds (loading={loading}, current page={current_page:?})"
            )
            .into());
        }

        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn wait_for_at_least_words(driver: &WebDriver, minimum: usize) -> HarnessResult<()> {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(error) = driver.find_all_css(ERROR).await?.into_iter().next() {
            return Err(format!(
                "Browse query failed while waiting for {minimum} results: {}",
                error.text().await?
            )
            .into());
        }
        let count = driver.find_all_css(WORD_CARD).await?.len();
        if count >= minimum {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "Browse returned {count} results rather than at least {minimum} within 15 seconds"
            )
            .into());
        }
        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn set_search_query(driver: &WebDriver, query: &str) -> HarnessResult<()> {
    let changed = driver
        .execute(
            crate::js_scripts::page_objects::BROWSE_SET_SEARCH_QUERY_01,
            vec![query.into()],
        )
        .await?
        .convert::<bool>()?;
    if !changed {
        return Err("Browse search input was not rendered".into());
    }
    Ok(())
}

pub async fn search_snapshot(driver: &WebDriver) -> HarnessResult<serde_json::Value> {
    Ok(driver
        .execute(
            crate::js_scripts::page_objects::BROWSE_SEARCH_SNAPSHOT_01,
            vec![],
        )
        .await?
        .convert::<serde_json::Value>()?)
}

pub async fn wait_for_search_query(
    driver: &WebDriver,
    query: &str,
    expected_headword: Option<&str>,
    minimum_results: usize,
) -> HarnessResult<serde_json::Value> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = search_snapshot(driver).await?;
        if let Some(error) = snapshot["error"].as_str() {
            return Err(format!("Browse search for {query:?} failed: {error}").into());
        }
        let value_matches = snapshot["query"].as_str() == Some(query);
        let ids = snapshot["ids"].as_array();
        let headword_matches = expected_headword.is_none_or(|expected| {
            snapshot["headwords"]
                .as_array()
                .is_some_and(|words| words.iter().any(|word| word.as_str() == Some(expected)))
        });
        if value_matches
            && !snapshot["loading"].as_bool().unwrap_or(false)
            && ids.is_some_and(|ids| ids.len() >= minimum_results)
            && headword_matches
        {
            return Ok(snapshot);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "Browse search for {query:?} did not settle within 30 seconds: {snapshot}"
            )
            .into());
        }
        sleep(Duration::from_millis(100)).await;
    }
}
