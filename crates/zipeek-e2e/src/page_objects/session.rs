use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const REMINDER: &str = "[data-unfinished-session]";
pub const RESUME: &str = "[data-unfinished-session] a[href*='/practice/session/']";
pub const END: &str = "[data-unfinished-session] button";
pub const DISMISS: &str = "[data-unfinished-session] button[aria-label='Dismiss session reminder']";

pub async fn reminder(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query_named(REMINDER, "Session - reminder").await
}

pub async fn dismiss(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .query_named(DISMISS, "Session - dismiss")
        .await?
        .click()
        .await?;
    Ok(())
}

pub async fn resume(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .query_named(RESUME, "Session - resume")
        .await?
        .click()
        .await?;
    driver
        .query_named("[data-practice-local-session]", "Session - resumed session")
        .await?;
    Ok(())
}

pub async fn end(driver: &WebDriver) -> WebDriverResult<()> {
    let panel = reminder(driver).await?;
    panel
        .query(By::XPath(".//button[normalize-space()='End session']"))
        .desc("Session - end")
        .single()
        .await?
        .click()
        .await?;
    Ok(())
}
