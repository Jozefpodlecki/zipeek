use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use thirtyfour::prelude::*;

use super::HarnessResult;

#[derive(Clone)]
pub struct ScenarioContext {
    pub driver: WebDriver,
    pub base_url: String,
    pub adapter_url: String,
    pub artifacts: PathBuf,
    pub(super) scripts: Arc<Mutex<Vec<String>>>,
}

impl ScenarioContext {
    pub fn url(&self, suffix: &str) -> String {
        format!("{}{suffix}", self.base_url)
    }

    pub async fn block_urls(&self, urls: &[&str]) -> HarnessResult<()> {
        self.driver
            .cdp()
            .send_raw("Network.setBlockedURLs", BTreeMap::from([("urls", urls)]))
            .await?;
        Ok(())
    }

    pub async fn add_init_script(&self, source: &str) -> HarnessResult<()> {
        let result = self
            .driver
            .cdp()
            .send_raw(
                "Page.addScriptToEvaluateOnNewDocument",
                BTreeMap::from([("source", source)]),
            )
            .await?;
        let identifier = result
            .get("identifier")
            .and_then(|value| value.as_str())
            .ok_or("CDP did not return a script identifier")?;
        self.scripts
            .lock()
            .map_err(|_| "Script registry poisoned")?
            .push(identifier.to_owned());
        Ok(())
    }

    pub(super) async fn reset(&self) -> HarnessResult<()> {
        let driver = &self.driver;
        driver.goto("about:blank").await?;
        let scripts = std::mem::take(
            &mut *self
                .scripts
                .lock()
                .map_err(|_| "Script registry poisoned")?,
        );
        for identifier in scripts {
            driver
                .cdp()
                .send_raw(
                    "Page.removeScriptToEvaluateOnNewDocument",
                    BTreeMap::from([("identifier", identifier)]),
                )
                .await?;
        }
        self.block_urls(&[]).await?;
        driver
            .cdp()
            .send_raw(
                "Network.setCacheDisabled",
                BTreeMap::from([("cacheDisabled", false)]),
            )
            .await?;
        driver
            .cdp()
            .send_raw(
                "Network.clearBrowserCookies",
                BTreeMap::<String, String>::new(),
            )
            .await?;
        for origin in [&self.base_url, &self.adapter_url] {
            let origin = origin.split('/').take(3).collect::<Vec<_>>().join("/");
            driver
                .cdp()
                .send_raw(
                    "Storage.clearDataForOrigin",
                    BTreeMap::from([("origin", origin.as_str()), ("storageTypes", "all")]),
                )
                .await?;
        }
        driver
            .cdp()
            .send_raw(
                "Emulation.clearDeviceMetricsOverride",
                BTreeMap::<String, String>::new(),
            )
            .await?;
        driver
            .cdp()
            .emulation()
            .set_device_metrics_override(1280, 1000, 1.0, false)
            .await?;
        let media = driver
            .execute(
                "return {features: [{name: 'prefers-reduced-motion', value: 'no-preference'}]};",
                vec![],
            )
            .await?;
        driver
            .cdp()
            .send_raw("Emulation.setEmulatedMedia", media.json())
            .await?;
        self.add_init_script(include_str!("../../fixtures/browser_probes/diagnostics.js"))
            .await?;
        driver.goto(self.url("/")).await?;
        driver
            .query(By::Css("[data-database-state='ready']"))
            .single()
            .await?;
        Ok(())
    }
}
