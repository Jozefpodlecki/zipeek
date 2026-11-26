use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const DRAWER: &str = "[data-component='navigation-drawer']";
pub const HEADER: &str = "[data-component='app-header']";
pub const SKIP: &str = "[data-skip-link]";

pub async fn visit(driver: &WebDriver, path: &str) -> crate::HarnessResult<()> {
    crate::page_objects::browser::navigate_hash(driver, path).await?;
    driver.query_named(HEADER, "Navigation - visit").await?;
    Ok(())
}

pub async fn current_hash(driver: &WebDriver) -> WebDriverResult<String> {
    driver
        .execute(
            crate::js_scripts::page_objects::NAVIGATION_CURRENT_HASH_01,
            vec![],
        )
        .await?
        .convert::<String>()
}

pub async fn skip_link_target_exists(driver: &WebDriver) -> WebDriverResult<bool> {
    driver
        .execute(
            crate::js_scripts::page_objects::NAVIGATION_SKIP_LINK_TARGET_EXISTS_01,
            vec![],
        )
        .await?
        .convert::<bool>()
}

pub async fn has_horizontal_overflow(driver: &WebDriver) -> WebDriverResult<bool> {
    driver
        .execute(
            crate::js_scripts::page_objects::NAVIGATION_HAS_HORIZONTAL_OVERFLOW_01,
            vec![],
        )
        .await?
        .convert::<bool>()
}

pub async fn wait_for_title(driver: &WebDriver, expected: &str) -> crate::HarnessResult<()> {
    let expected_title = format!("{expected} · Zipeek");
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let current = driver.title().await?;
        if current == expected_title {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(
                format!("Expected document title {expected_title:?}, found {current:?}").into(),
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
