use serde::Serialize;
use thirtyfour::{WebDriver, error::WebDriverResult};

pub struct ChromeDevTools<'a> {
    driver: &'a WebDriver,
}

impl<'a> ChromeDevTools<'a> {
    pub fn new(driver: &'a WebDriver) -> Self {
        Self { driver }
    }

    pub async fn viewport(&self, width: u32, height: u32, mobile: bool) -> WebDriverResult<()> {
        self.driver
            .cdp()
            .emulation()
            .set_device_metrics_override(width as _, height as _, 1.0, mobile)
            .await?;
        Ok(())
    }

    pub async fn media<T: Serialize>(&self, settings: T) -> WebDriverResult<()> {
        self.driver
            .cdp()
            .send_raw("Emulation.setEmulatedMedia", settings)
            .await?;
        Ok(())
    }

    pub async fn reduced_motion(&self, enabled: bool) -> WebDriverResult<()> {
        let value = if enabled { "reduce" } else { "no-preference" };
        self.media(serde_json::json!({
            "features": [{"name": "prefers-reduced-motion", "value": value}]
        }))
        .await
    }

    pub async fn block_patterns(&self, patterns: &[&str]) -> WebDriverResult<()> {
        self.blocked_urls(serde_json::json!({ "urls": patterns }))
            .await
    }

    pub async fn network_enabled(&self) -> WebDriverResult<()> {
        self.driver
            .cdp()
            .send_raw(
                "Network.enable",
                std::collections::BTreeMap::<String, String>::new(),
            )
            .await?;
        Ok(())
    }

    pub async fn blocked_urls<T: Serialize>(&self, settings: T) -> WebDriverResult<()> {
        self.driver
            .cdp()
            .send_raw("Network.setBlockedURLs", settings)
            .await?;
        Ok(())
    }

    pub async fn init_script<T: Serialize>(
        &self,
        settings: T,
    ) -> WebDriverResult<serde_json::Value> {
        self.driver
            .cdp()
            .send_raw("Page.addScriptToEvaluateOnNewDocument", settings)
            .await
    }

    pub async fn remove_init_script<T: Serialize>(&self, settings: T) -> WebDriverResult<()> {
        self.driver
            .cdp()
            .send_raw("Page.removeScriptToEvaluateOnNewDocument", settings)
            .await?;
        Ok(())
    }
}
