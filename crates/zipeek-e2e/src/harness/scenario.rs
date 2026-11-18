use crate::harness::{
    AppRoute, BrowserKind, BrowserViewport, ChromeDevTools, FixtureClient, FixtureScenario,
};
use crate::page_objects::WebDriverActions;
use std::{collections::BTreeMap, path::PathBuf};

use thirtyfour::prelude::*;

use super::HarnessResult;

#[derive(Clone)]
pub struct ScenarioContext {
    pub browser: BrowserKind,
    pub driver: WebDriver,
    pub base_url: String,
    pub adapter_url: String,
    pub ai_url: String,
    pub image_url: String,
    pub artifacts: PathBuf,
}

impl ScenarioContext {
    pub fn viewport(&self) -> BrowserViewport<'_> {
        BrowserViewport::for_browser(&self.driver, self.browser)
    }

    pub fn url(&self, suffix: &str) -> String {
        format!("{}{suffix}", self.base_url)
    }

    pub async fn open(&self, route: AppRoute) -> HarnessResult<()> {
        self.driver.goto(route.url(&self.base_url)).await?;
        self.driver
            .query_single("[data-component='app-shell']")
            .await?;
        Ok(())
    }

    pub async fn open_route(&self, path: &str) -> HarnessResult<()> {
        let route = AppRoute::from_deep_link(path)?;
        self.driver.goto(route.url(&self.base_url)).await?;
        let route_path = path.split_once('?').map_or(path, |(path, _)| path);
        let marker = crate::page_objects::routes::ROUTES
            .iter()
            .find(|case| case.path == route_path)
            .map_or("[data-component='app-shell']", |case| case.marker);
        self.driver.query_single(marker).await?;
        Ok(())
    }

    pub async fn navigate_in_app(&self, route: AppRoute) -> HarnessResult<()> {
        crate::page_objects::browser::navigate_hash(&self.driver, &route.hash_path()).await
    }

    pub async fn checkpoint_visible(&self, label: &str, selector: &str) -> HarnessResult<()> {
        let element = self.driver.query_single(selector).await?;
        if !element.is_displayed().await? {
            return Err(format!("Checkpoint {label} is not visible: {selector}").into());
        }
        self.checkpoint(label).await
    }

    pub async fn checkpoint(&self, label: &str) -> HarnessResult<()> {
        if std::env::var_os("ZIPEEK_E2E_RECORD").as_deref() == Some(std::ffi::OsStr::new("0")) {
            return Ok(());
        }
        if !valid_checkpoint_label(label) {
            return Err(format!("Invalid evidence checkpoint label: {label}").into());
        }
        let directory = self.artifacts.join("timeline");
        std::fs::create_dir_all(&directory)?;
        let index = match std::fs::read_to_string(directory.join("steps.jsonl")) {
            Ok(content) => content.lines().count(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(format!("Cannot read previous E2E timeline: {error}").into()),
        };
        if index >= 100 {
            return Err("E2E evidence checkpoint limit exceeded".into());
        }
        let screenshot = if std::env::var("ZIPEEK_E2E_SCREENSHOTS").as_deref() != Ok("0") {
            let filename = format!("{index:03}-{label}.png");
            self.driver.screenshot(&directory.join(&filename)).await?;
            Some(filename)
        } else {
            None
        };
        let record = serde_json::json!({
            "index": index,
            "label": label,
            "screenshot": screenshot,
            "unix_ms": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?.as_millis(),
        });
        use std::io::Write;
        writeln!(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(directory.join("steps.jsonl"))?,
            "{record}"
        )?;
        Ok(())
    }

    pub async fn configure_image_fixture(&self) -> HarnessResult<()> {
        let applied = self
            .driver
            .execute(
                crate::js_scripts::harness::CONFIGURE_IMAGE_FIXTURE,
                vec![self.image_url.clone().into()],
            )
            .await?
            .convert::<bool>()?;
        if !applied {
            return Err("Could not configure browser image-server preferences".into());
        }
        Ok(())
    }

    pub async fn configure_image_exercise_fixtures(&self) -> HarnessResult<()> {
        self.configure_ai_fixture("success").await?;
        let applied = self
            .driver
            .execute(
                crate::js_scripts::harness::CONFIGURE_IMAGE_EXERCISE_FIXTURES,
                vec![self.image_url.clone().into(), self.ai_url.clone().into()],
            )
            .await?
            .convert::<bool>()?;
        if !applied {
            return Err("Could not configure Picture Vocabulary AI and image fixtures".into());
        }
        Ok(())
    }

    pub async fn assert_ai_request_kind(&self, kind: &str) -> HarnessResult<()> {
        let requests_url = self.ai_url.replace("/v1/", "/__e2e/requests");
        let observed = self
            .driver
            .execute_async(
                crate::js_scripts::harness::ASSERT_AI_FIXTURE_REQUEST_KIND,
                vec![requests_url.into(), kind.to_owned().into()],
            )
            .await?
            .convert::<bool>()?;
        if !observed {
            return Err(format!("AI fixture did not record a {kind} request").into());
        }
        Ok(())
    }

    pub async fn assert_image_fixture_reachable(&self) -> HarnessResult<()> {
        let reachable = self
            .driver
            .execute_async(
                crate::js_scripts::harness::ASSERT_IMAGE_FIXTURE_HEALTH,
                vec![self.image_url.clone().into()],
            )
            .await?
            .convert::<bool>()?;
        if !reachable {
            return Err(format!(
                "Image fixture is inaccessible from browser origin {}",
                self.base_url
            )
            .into());
        }
        Ok(())
    }

    pub async fn assert_image_fixture_generates_image(&self) -> HarnessResult<()> {
        let generated = self
            .driver
            .execute_async(
                crate::js_scripts::harness::ASSERT_IMAGE_FIXTURE_GENERATES_IMAGE,
                vec![self.image_url.clone().into()],
            )
            .await?
            .convert::<bool>()?;
        if !generated {
            return Err(
                "Image fixture did not generate and serve an image through browser CORS".into(),
            );
        }
        Ok(())
    }

    pub async fn configure_ai_fixture(&self, scenario: &str) -> HarnessResult<()> {
        FixtureClient::from_ai_url(&self.ai_url)?
            .reset(FixtureScenario::parse(scenario)?)
            .await
    }

    pub async fn block_urls(&self, urls: &[&str]) -> HarnessResult<()> {
        if !self.browser.supports_cdp() {
            return Err(
                "Network blocking requires Chrome CDP; this scenario is not Firefox-compatible"
                    .into(),
            );
        }
        ChromeDevTools::new(&self.driver)
            .block_patterns(urls)
            .await?;
        Ok(())
    }

    pub async fn add_init_script(&self, source: &str) -> HarnessResult<()> {
        if !self.browser.supports_cdp() {
            return Err("Pre-navigation script injection requires Chrome CDP; this scenario is not Firefox-compatible".into());
        }
        let result = ChromeDevTools::new(&self.driver)
            .init_script(BTreeMap::from([("source", source)]))
            .await?;
        result
            .get("identifier")
            .and_then(|value| value.as_str())
            .ok_or("CDP did not return a script identifier")?;
        Ok(())
    }

    async fn enter_application(&self) -> HarnessResult<()> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if !self
                .driver
                .find_all_css("[data-database-state]")
                .await?
                .is_empty()
            {
                return Ok(());
            }
            let entry = self.driver.find_all_css("[data-welcome-enter]").await?;
            if let Some(button) = entry.into_iter().next() {
                button.click().await?;
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                let address = self
                    .driver
                    .current_url()
                    .await
                    .map(|value| value.to_string())
                    .unwrap_or_else(|_| "unknown".into());
                return Err(format!("Neither landing entry nor database boundary appeared within 15 seconds at {address}").into());
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    async fn wait_for_database(&self) -> HarnessResult<()> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(45);
        loop {
            let state = self.driver.find_all_css("[data-database-state]").await?;
            if state.len() > 1 {
                return Err(
                    format!("Ambiguous database boundary: {} state markers", state.len()).into(),
                );
            }
            for node in state {
                match node.attr("data-database-state").await?.as_deref() {
                    Some("ready") => return Ok(()),
                    Some("failed") => {
                        let message = node
                            .text()
                            .await
                            .unwrap_or_else(|_| "Database initialization failed".into());
                        return Err(format!("Dictionary initialization failed: {message}").into());
                    }
                    _ => {}
                }
            }
            if tokio::time::Instant::now() >= deadline {
                let url = self
                    .driver
                    .current_url()
                    .await
                    .map(|url| url.to_string())
                    .unwrap_or_else(|_| "unknown".into());
                let states = self
                    .driver
                    .execute(
                        crate::js_scripts::harness::SCENARIO_WAIT_FOR_DATABASE_01,
                        vec![],
                    )
                    .await
                    .map(|result| result.json().to_string())
                    .unwrap_or_default();
                return Err(format!(
                    "Dictionary readiness timed out after 45 seconds at {url}; observed: {states}"
                )
                .into());
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }

    async fn prepare_firefox(&self) -> HarnessResult<()> {
        let driver = &self.driver;
        driver.goto(&self.base_url).await?;
        let preference = include_str!("../../fixtures/preferences-default.json");
        driver
            .execute(
                crate::js_scripts::harness::FIREFOX_BOOTSTRAP_STORAGE,
                vec![preference.into()],
            )
            .await?;
        driver.refresh().await?;
        driver
            .execute(
                include_str!("../../fixtures/browser_probes/diagnostics.js"),
                vec![],
            )
            .await?;
        self.enter_application().await?;
        let mode = driver
            .execute(
                crate::js_scripts::harness::SCENARIO_WAIT_FOR_DATABASE_03,
                vec![],
            )
            .await?
            .convert::<Option<String>>()?;
        if mode.as_deref() != Some("silent") {
            return Err(format!("Firefox E2E speech fixture failed; mode: {mode:?}").into());
        }
        self.wait_for_database().await?;
        self.configure_ai_fixture("success").await?;
        Ok(())
    }

    pub(super) async fn prepare(&self) -> HarnessResult<()> {
        if self.browser == BrowserKind::Firefox {
            return self.prepare_firefox().await;
        }
        let driver = &self.driver;
        driver.goto("about:blank").await?;
        ChromeDevTools::new(&driver)
            .viewport(1280, 1000, false)
            .await?;
        let media = driver
            .execute(
                crate::js_scripts::harness::SCENARIO_WAIT_FOR_DATABASE_02,
                vec![],
            )
            .await?;
        ChromeDevTools::new(&driver).media(media.json()).await?;
        self.add_init_script(include_str!("../../fixtures/browser_probes/diagnostics.js"))
            .await?;
        self.add_init_script(
            "try { if (location.protocol.startsWith('http')) sessionStorage.setItem('zipeek.e2e.settings.v1', JSON.stringify({speech:'silent'})); } catch (_) {}",
        )
        .await?;
        let onboarding = ChromeDevTools::new(&driver).init_script(BTreeMap::from([(
                    "source",
                    format!(
                        "try {{ const key = 'zipeek.preferences.v1'; if (location.protocol.startsWith('http') && localStorage.getItem(key) === null) localStorage.setItem(key, {}); }} catch (_) {{}}",
                        serde_json::to_string(include_str!("../../fixtures/preferences-default.json"))
                            .map_err(|error| format!("Could not serialize E2E preferences fixture: {error}"))?,
                    ),
                )]))
            .await?;
        let script_id = onboarding
            .get("identifier")
            .and_then(|identifier| identifier.as_str())
            .ok_or("CDP did not return the returning-user fixture script identifier")?;
        let first_navigation = driver.goto(&self.base_url).await;
        ChromeDevTools::new(&driver)
            .remove_init_script(BTreeMap::from([("identifier", script_id)]))
            .await?;
        first_navigation?;
        self.enter_application().await?;
        let mode = driver
            .execute(
                crate::js_scripts::harness::SCENARIO_WAIT_FOR_DATABASE_03,
                vec![],
            )
            .await?
            .convert::<Option<String>>()?;
        if mode.as_deref() != Some("silent") {
            return Err(format!("E2E speech activation failed; resolved mode: {mode:?}").into());
        }
        self.wait_for_database().await?;
        self.configure_ai_fixture("success").await?;
        Ok(())
    }
}

fn valid_checkpoint_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 64
        && label
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod checkpoint_tests {
    use super::valid_checkpoint_label;

    #[test]
    fn evidence_paths_reject_traversal_and_control_characters() {
        for label in [
            "../data",
            "a/b",
            "a\\b",
            "",
            "With Spaces",
            "a\n",
            "a.json",
            "é",
        ] {
            assert!(
                !valid_checkpoint_label(label),
                "accepted invalid label {label:?}"
            );
        }
        assert!(!valid_checkpoint_label(&"a".repeat(65)));
        assert!(valid_checkpoint_label("ai-answer-after-refresh-1"));
    }
}
