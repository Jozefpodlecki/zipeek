use crate::harness::{BrowserKind, ChromeDevTools};
use std::time::Duration;

use thirtyfour::prelude::*;

use super::HarnessResult;

fn visible_browser_requested(value: Option<&std::ffi::OsStr>) -> bool {
    value == Some(std::ffi::OsStr::new("1"))
}

async fn configure_timeouts(driver: &WebDriver) -> WebDriverResult<()> {
    driver.set_script_timeout(Duration::from_secs(60)).await?;
    driver.set_implicit_wait_timeout(Duration::ZERO).await?;
    driver
        .set_page_load_timeout(Duration::from_secs(60))
        .await?;
    Ok(())
}

pub async fn start(kind: BrowserKind) -> HarnessResult<WebDriver> {
    match kind {
        BrowserKind::Chrome => start_chrome().await,
        BrowserKind::Firefox => start_firefox().await,
    }
}

async fn start_chrome() -> HarnessResult<WebDriver> {
    let mut capabilities = DesiredCapabilities::chrome();
    if !visible_browser_requested(std::env::var_os("ZIPEEK_E2E_VISIBLE").as_deref()) {
        capabilities.set_headless()?;
    }
    capabilities.add_arg("--window-size=1280,1000")?;
    capabilities.add_arg("--disable-backgrounding-occluded-windows")?;
    capabilities.add_arg("--use-fake-device-for-media-stream")?;
    capabilities.add_arg("--use-fake-ui-for-media-stream")?;
    let driver = WebDriver::managed(capabilities).await?;

    let setup = async {
        configure_timeouts(&driver).await?;
        ChromeDevTools::new(&driver).network_enabled().await?;
        WebDriverResult::Ok(())
    }
    .await;
    if let Err(error) = setup {
        let _ = driver.quit().await;
        return Err(error.into());
    }
    Ok(driver)
}

async fn start_firefox() -> HarnessResult<WebDriver> {
    let mut capabilities = DesiredCapabilities::firefox();
    if !visible_browser_requested(std::env::var_os("ZIPEEK_E2E_VISIBLE").as_deref()) {
        capabilities.set_headless()?;
    }
    let driver = WebDriver::managed(capabilities).await?;
    let setup = async {
        configure_timeouts(&driver).await?;
        driver.set_window_rect(0, 0, 1280, 1000).await?;
        WebDriverResult::Ok(())
    }
    .await;
    if let Err(error) = setup {
        let _ = driver.quit().await;
        return Err(error.into());
    }
    Ok(driver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_browser_is_explicit_opt_in() {
        assert!(!visible_browser_requested(None));
        assert!(!visible_browser_requested(Some(std::ffi::OsStr::new(
            "true"
        ))));
        assert!(!visible_browser_requested(Some(std::ffi::OsStr::new("0"))));
        assert!(visible_browser_requested(Some(std::ffi::OsStr::new("1"))));
    }
}
