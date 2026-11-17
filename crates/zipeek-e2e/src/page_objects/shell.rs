use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

const DATABASE_READY: &str = "[data-database-state='ready'][data-database-rows]:not([data-database-rows='0'])[data-database-engine]";
const BACKGROUND: &str = "#app-background";
const HEADER: &str = "[data-component='app-header']";
const DRAWER: &str = "[data-component='navigation-drawer']";
const CONTENT: &str = "[data-component='main-content']";

pub async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query_named(DATABASE_READY, "Shell - ready").await
}

pub async fn background_source(driver: &WebDriver) -> crate::HarnessResult<String> {
    let image = driver
        .query_named(BACKGROUND, "Shell - background source")
        .await?;
    image
        .attr("src")
        .await?
        .filter(|source| !source.is_empty())
        .ok_or_else(|| "App background image is missing its src attribute".into())
}

pub async fn assert_navigation_ready(driver: &WebDriver) -> crate::HarnessResult<()> {
    driver.query_named(HEADER, "Shell - header ready").await?;
    let drawer = driver.query_named(DRAWER, "Shell - drawer ready").await?;
    if !drawer.is_displayed().await? {
        return Err("App shell navigation is unexpectedly hidden".into());
    }
    let main = driver
        .query_named(CONTENT, "Shell - main content ready")
        .await?;
    if !main.is_displayed().await? {
        return Err("App shell main content is unexpectedly hidden".into());
    }
    Ok(())
}
