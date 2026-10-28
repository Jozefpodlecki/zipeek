mod artifacts;
mod browser;
mod error;
mod scenario;
mod server;

use std::path::PathBuf;

use thirtyfour::{WebDriver, prelude::ElementQueryable};

pub use error::HarnessError;
pub use scenario::ScenarioContext;
pub use server::SuiteKind;

pub type HarnessResult<T> = Result<T, HarnessError>;

pub struct Harness {
    pub driver: WebDriver,
    pub base_url: String,
    pub adapter_url: String,
    pub artifacts: PathBuf,
    scripts: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    servers: Vec<server::OwnedProcess>,
    failures: Vec<String>,
}

impl Harness {
    pub async fn start(kind: SuiteKind) -> HarnessResult<Self> {
        let artifacts = artifacts::run_directory(kind.name())?;
        println!("e2e artifacts: {}", artifacts.display());
        let output = artifacts.clone();
        let setup = tokio::task::spawn_blocking(move || server::start(kind, &output)).await?;
        let (servers, base_url, adapter_url) = match setup {
            Ok(setup) => setup,
            Err(error) => {
                artifacts::setup_failure(&artifacts, "setup-failure.txt", &error);
                return Err(error);
            }
        };
        let driver = match browser::start().await {
            Ok(driver) => driver,
            Err(error) => {
                artifacts::setup_failure(&artifacts, "browser-failure.txt", &error);
                return Err(error);
            }
        };
        Ok(Self {
            driver,
            base_url,
            adapter_url,
            artifacts,
            scripts: Default::default(),
            servers,
            failures: Vec::new(),
        })
    }

    pub async fn run<F, Fut>(&mut self, name: &str, scenario: F) -> HarnessResult<()>
    where
        F: FnOnce(ScenarioContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = HarnessResult<()>> + Send + 'static,
    {
        let result = self.run_case(name, scenario).await;
        if let Err(error) = result {
            self.failures.push(format!("{name}: {error}"));
            if self.context("session-recovery")?.reset().await.is_err() {
                return Err(
                    format!("Browser session unusable; {}", self.failures.join("\n")).into(),
                );
            }
        }
        Ok(())
    }

    async fn run_case<F, Fut>(&self, name: &str, scenario: F) -> HarnessResult<()>
    where
        F: FnOnce(ScenarioContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = HarnessResult<()>> + Send + 'static,
    {
        println!("SCENARIO {name}");
        let context = self.context(name)?;
        if let Err(error) = context.reset().await {
            artifacts::failure(&context, &format!("Scenario setup: {error}")).await;
            return Err(error);
        }
        let task_context = context.clone();
        let outcome = tokio::spawn(async move { scenario(task_context).await }).await;
        let primary = match outcome {
            Ok(result) => result,
            Err(error) => Err(format!("Scenario task failed: {error}").into()),
        };
        artifacts::diagnostics(&context).await;
        if let Err(error) = &primary {
            artifacts::failure(&context, &error.to_string()).await;
        }
        let cleanup = context.reset().await;
        if let Err(error) = &cleanup {
            artifacts::failure(&context, &format!("Scenario cleanup: {error}")).await;
        }
        match (primary, cleanup) {
            (Err(primary), _) => Err(primary),
            (Ok(()), Err(cleanup)) => Err(cleanup),
            (Ok(()), Ok(())) => {
                std::fs::write(context.artifacts.join("passed.txt"), name)?;
                println!("PASS {name}");
                Ok(())
            }
        }
    }

    fn context(&self, name: &str) -> HarnessResult<ScenarioContext> {
        let artifacts = self.artifacts.join(name);
        std::fs::create_dir_all(&artifacts)?;
        Ok(ScenarioContext {
            driver: self.driver.clone(),
            base_url: self.base_url.clone(),
            adapter_url: self.adapter_url.clone(),
            artifacts,
            scripts: self.scripts.clone(),
        })
    }

    pub async fn prove_failure_isolation(&self) -> HarnessResult<()> {
        for panic in [false, true] {
            let name = if panic {
                "expected-panic"
            } else {
                "expected-failure"
            };
            let result = self.run_case(name, move |context| async move {
                context.add_init_script("window.e2ePoisoned = true;").await?;
                let driver = &context.driver;
                driver.execute("localStorage.setItem('e2e-poison', 'true'); document.cookie = 'e2e-poison=true';", vec![]).await?;
                context.block_urls(&["*hanzi.duckdb"]).await?;
                driver.cdp().emulation().set_device_metrics_override(400, 500, 1.0, true).await?;
                let reduced = driver.execute("return {features: [{name: 'prefers-reduced-motion', value: 'reduce'}]};", vec![]).await?;
                driver.cdp().send_raw("Emulation.setEmulatedMedia", reduced.json()).await?;
                assert!(!panic, "deliberate isolation panic");
                Err("deliberate isolation failure".into())
            }).await;
            if result.is_ok() {
                return Err("Deliberate failure was not reported".into());
            }
            let context = self.context(&format!("{name}-recovery"))?;
            let clean = context.driver.execute("return !localStorage.getItem('e2e-poison') && !document.cookie.includes('e2e-poison') && !window.e2ePoisoned && innerWidth === 1280 && !matchMedia('(prefers-reduced-motion: reduce)').matches;", vec![]).await?.convert::<bool>()?;
            if !clean {
                return Err(format!("State leaked after {name}").into());
            }
            context
                .driver
                .query(thirtyfour::By::Css("[data-database-state='ready']"))
                .single()
                .await?;
            std::fs::write(
                context.artifacts.join("passed.txt"),
                "Failure cleaned; same session recovered",
            )?;
        }
        Ok(())
    }

    pub async fn finish(mut self) -> HarnessResult<()> {
        if let Err(error) = self.driver.clone().quit().await {
            self.failures.push(format!("Browser shutdown: {error}"));
        }
        for server in &mut self.servers {
            if let Err(error) = server.stop() {
                self.failures.push(format!("Server shutdown: {error}"));
            }
        }
        let summary = if self.failures.is_empty() {
            "All scenarios passed; browser quit and owned servers reaped".to_owned()
        } else {
            self.failures.join("\n")
        };
        let report = std::fs::write(self.artifacts.join("summary.txt"), &summary);
        if self.failures.is_empty() {
            report?;
            Ok(())
        } else {
            if let Err(error) = report {
                eprintln!("Summary write failed: {error}");
            }
            Err(summary.into())
        }
    }

    pub fn workspace() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
    }
}
