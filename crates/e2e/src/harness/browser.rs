use std::time::Duration;

use thirtyfour::prelude::*;

use super::HarnessResult;

pub async fn start() -> HarnessResult<WebDriver> {
    let mut capabilities = DesiredCapabilities::chrome();
    capabilities.set_headless()?;
    capabilities.add_arg("--window-size=1280,1000")?;
    let driver = WebDriver::managed(capabilities).await?;
    let setup = async {
        driver.set_script_timeout(Duration::from_secs(60)).await?;
        driver
            .set_page_load_timeout(Duration::from_secs(60))
            .await?;
        driver
            .cdp()
            .send_raw(
                "Network.enable",
                std::collections::BTreeMap::<String, String>::new(),
            )
            .await?;
        WebDriverResult::Ok(())
    }
    .await;
    if let Err(error) = setup {
        let _ = driver.quit().await;
        return Err(error.into());
    }
    Ok(driver)
}
