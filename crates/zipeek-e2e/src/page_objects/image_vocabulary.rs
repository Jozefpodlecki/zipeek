use std::time::{Duration, Instant};
use thirtyfour::prelude::*;

use crate::HarnessResult;

use super::{WebDriverActions, selectors::image_review};

pub const PAGE: &str = "[data-image-vocabulary]";
pub const ANSWERS: &str = "[data-image-answer-option]";
pub const CHECK: &str = "[data-image-check-answer]";
pub const REGENERATE: &str = "[data-image-regenerate]";
pub const VERDICT: &str = "[data-image-vocabulary] p[role='status']";

pub async fn wait_for_review(driver: &WebDriver) -> HarnessResult<()> {
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        let approvals = driver.find_all_css(image_review::APPROVE).await?;
        if approvals.len() == 1 {
            if approvals[0].is_enabled().await? {
                return Ok(());
            }
        }
        let alerts = driver
            .find_all_css("[data-image-vocabulary] [role='alert']")
            .await?;
        if let Some(alert) = alerts.first() {
            let message = alert.text().await?;
            return Err(format!("Picture Vocabulary generation failed: {message}").into());
        }
        let image_errors = driver.find_all_css(image_review::ERROR).await?;
        if !image_errors.is_empty() {
            return Err("Picture Vocabulary image failed to load in Chrome".into());
        }
        if Instant::now() >= deadline {
            let progress = driver
                .find_all_css("[data-image-generation-progress]")
                .await?;
            let phase = if let Some(progress) = progress.first() {
                progress.text().await?
            } else {
                "No generation progress indicator".to_owned()
            };
            let review = driver.find_all_css(image_review::ROOT).await?.len();
            let loading = driver.find_all_css(image_review::LOADING).await?.len();
            let failed = driver.find_all_css(image_review::ERROR).await?.len();
            return Err(format!(
                "Picture Vocabulary review unavailable after 45s: phase={phase:?}, review={review}, image_loading={loading}, image_failed={failed}"
            ).into());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

pub async fn approve(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .query_single(format!("{}:not([disabled])", image_review::APPROVE))
        .await?
        .click()
        .await
}

pub async fn select_options(driver: &WebDriver, count: usize) -> WebDriverResult<usize> {
    let choices = driver.find_all_css(ANSWERS).await?;
    let available = choices.len();
    for choice in choices.iter().take(count) {
        choice.click().await?;
    }
    Ok(available)
}

pub async fn check(driver: &WebDriver) -> WebDriverResult<String> {
    driver
        .query_single(format!("{CHECK}:not([disabled])"))
        .await?
        .click()
        .await?;
    driver.query_single(VERDICT).await?.text().await
}

pub async fn regenerate(driver: &WebDriver) -> WebDriverResult<()> {
    driver.click(REGENERATE).await?;
    driver
        .query_single(format!("{}:not([disabled])", image_review::APPROVE))
        .await?;
    Ok(())
}

pub async fn all_options_disabled(driver: &WebDriver) -> WebDriverResult<bool> {
    let choices = driver.find_all_css(ANSWERS).await?;
    if choices.is_empty() {
        return Ok(false);
    }
    for choice in choices {
        if choice.is_enabled().await? {
            return Ok(false);
        }
    }
    Ok(true)
}
