mod ai_fixture;
mod artifacts;
mod assertions;
mod browser;
mod browser_kind;
pub mod browser_scripts;
pub mod chrome;
mod error;
mod fixture_client;
mod recorder;
pub mod routes;
mod scenario;
mod server;
mod viewport;

use std::path::PathBuf;

use thirtyfour::WebDriver;

pub use assertions::{AssertionFailure, AssertionKind};
pub use browser_kind::BrowserKind;
pub use chrome::ChromeDevTools;
pub use error::HarnessError;
pub use fixture_client::{FixtureClient, FixtureScenario};
pub use routes::AppRoute;
pub use scenario::ScenarioContext;
pub use server::SuiteKind;
pub use viewport::BrowserViewport;

pub type HarnessResult<T> = Result<T, HarnessError>;

pub struct Harness {
    pub browser: BrowserKind,
    pub base_url: String,
    pub adapter_url: String,
    pub ai_url: String,
    pub image_url: String,
    image_server: Option<zipeek_image_server::ImageServer>,
    static_server: Option<zipeek_static_server::StaticServer>,
    pub artifacts: PathBuf,
    servers: Vec<server::OwnedServer>,
    failures: Vec<String>,
}

impl Harness {
    pub async fn start(kind: SuiteKind) -> HarnessResult<Self> {
        let browser = BrowserKind::from_environment()?;
        let artifacts = artifacts::run_directory(kind.name())?;
        std::fs::write(artifacts.join("browser.txt"), browser.as_str())?;
        println!("e2e artifacts: {}", artifacts.display());
        let output = artifacts.clone();
        let setup = tokio::task::spawn_blocking(move || server::start(kind, &output)).await?;
        let built = match setup {
            Ok(setup) => setup,
            Err(error) => {
                artifacts::setup_failure(&artifacts, "setup-failure.txt", &error);
                if let Err(report_error) = artifacts::suite_start_failure(&artifacts, &error) {
                    eprintln!("Suite startup manifest unavailable: {report_error}");
                }
                return Err(error);
            }
        };
        let fixture = FixtureClient::from_ai_url(&built.ai_url)?;
        fixture.health().await?;
        let static_server = if let Some(dist) = built.dist {
            let config = zipeek_static_server::ServerConfig {
                root: dist,
                address: std::net::SocketAddr::from(([127, 0, 0, 1], 0)),
                base_path: "/".to_owned(),
            };
            Some(
                zipeek_static_server::StaticServer::run(config)
                    .await
                    .map_err(|error| format!("Cannot start owned static server: {error}"))?,
            )
        } else {
            None
        };
        let base_url = static_server.as_ref().map_or_else(
            || built.adapter_url.clone(),
            zipeek_static_server::StaticServer::url,
        );
        let mut options = zipeek_image_server::ImageServerOptions::placeholder(
            std::net::SocketAddr::from(([127, 0, 0, 1], 0)),
            artifacts.join("image-server-data"),
        );
        options
            .allowed_origins
            .push(origin_of(&base_url)?.to_owned());
        let image_server = zipeek_image_server::ImageServer::run(options)
            .await
            .map_err(|error| format!("Cannot start owned image fixture: {error}"))?;
        let image_url = image_server.url();
        Ok(Self {
            browser,
            base_url,
            adapter_url: built.adapter_url,
            ai_url: built.ai_url,
            image_url,
            image_server: Some(image_server),
            static_server,
            artifacts,
            servers: built.servers,
            failures: Vec::new(),
        })
    }

    pub async fn run<F, Fut>(&mut self, name: &str, scenario: F) -> HarnessResult<()>
    where
        F: FnOnce(ScenarioContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = HarnessResult<()>> + Send + 'static,
    {
        if let Err(error) = self.run_case(name, scenario).await {
            self.failures.push(format!("{name}: {error}"));
        }
        Ok(())
    }

    async fn run_case<F, Fut>(&self, name: &str, scenario: F) -> HarnessResult<()>
    where
        F: FnOnce(ScenarioContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = HarnessResult<()>> + Send + 'static,
    {
        if !artifacts::valid_artifact_name(name) {
            return Err(format!("Invalid E2E scenario name: {name}").into());
        }
        if self.artifacts.join(name).exists() {
            return Err(format!("Duplicate E2E scenario artifact name: {name}").into());
        }
        println!("SCENARIO {name}");
        let directory = self.artifacts.join(name);
        std::fs::create_dir_all(&directory)?;
        let driver = match browser::start(self.browser).await {
            Ok(driver) => driver,
            Err(error) => {
                artifacts::setup_failure(&directory, "browser-failure.txt", &error);
                artifacts::case_result(&directory, "failed", "browser", Some(&error.to_string()))?;
                return Err(error);
            }
        };
        let context = self.context(name, driver.clone())?;
        std::fs::write(directory.join("browser.txt"), self.browser.as_str())?;
        let mut recording_error = None;
        let recorder =
            match recorder::VisualRecorder::start(&driver, &directory, self.browser).await {
                Ok(recorder) => recorder,
                Err(error) => {
                    eprintln!("Video capture unavailable: {error}");
                    artifacts::setup_failure(&directory, "video-failure.txt", &error);
                    recording_error = Some(error);
                    None
                }
            };

        let mut checkpoint_failure = None;
        let setup = context.prepare().await;
        let (phase, primary) = match setup {
            Ok(()) => {
                if let Err(error) = context.checkpoint("scenario-ready").await {
                    eprintln!("Evidence checkpoint failed: {error}");
                    checkpoint_failure = Some(format!("scenario-ready: {error}"));
                }
                let task_context = context.clone();
                let outcome = tokio::spawn(async move { scenario(task_context).await }).await;
                let result = match outcome {
                    Ok(result) => result,
                    Err(error) => Err(format!("Scenario task failed: {error}").into()),
                };
                ("scenario", result)
            }
            Err(error) => ("setup", Err(error)),
        };

        if let Err(error) = context.checkpoint("scenario-finished").await {
            eprintln!("Evidence checkpoint failed: {error}");
            let message = format!("scenario-finished: {error}");
            checkpoint_failure = Some(match checkpoint_failure {
                Some(previous) => format!("{previous}; {message}"),
                None => message,
            });
        }
        if let Some(error) = &checkpoint_failure {
            std::fs::write(directory.join("checkpoint-failure.txt"), error)?;
        }
        artifacts::diagnostics(&context).await;
        if let Err(error) = &primary {
            artifacts::failure(&context, &format!("{phase}: {error}")).await;
        }
        if let Some(recorder) = recorder {
            if let Err(error) = recorder.stop().await {
                artifacts::setup_failure(&directory, "video-failure.txt", &error);
                recording_error = Some(error);
            }
        }
        let shutdown_error = driver
            .quit()
            .await
            .err()
            .map(|error| format!("Browser shutdown failed after {name}: {error}"));
        if let Some(error) = &shutdown_error {
            eprintln!("{error}");
        }
        let primary_error = primary.as_ref().err().map(ToString::to_string);
        let evidence_strict = std::env::var("ZIPEEK_E2E_EVIDENCE_STRICT").as_deref() == Ok("1");
        let strict_recording_error = if primary.is_ok() && (recorder::VisualRecorder::strict() || evidence_strict) {
            recording_error.as_ref().map(ToString::to_string).or_else(|| {
                if evidence_strict && !directory.join("video/status.json").is_file() {
                    Some("Strict E2E evidence requires a completed video capture status".to_owned())
                } else {
                    None
                }
            }).or_else(|| {
                if evidence_strict {
                    let video_status = std::fs::read(directory.join("video/status.json"))
                        .ok()
                        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
                    if video_status.as_ref().is_none_or(|value| value["validation"]["status"] != "valid" || value["truncated"] == true || value["captured"].as_u64() == Some(0)) {
                        Some("Strict E2E evidence requires a valid, nontruncated recording".to_owned())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        } else {
            None
        };
        let strict_checkpoint_error = if evidence_strict && primary.is_ok() {
            checkpoint_failure.clone().or_else(|| {
                if !directory.join("timeline/steps.jsonl").is_file() {
                    Some("Strict E2E evidence requires a checkpoint timeline".to_owned())
                } else {
                    None
                }
            })
        } else {
            None
        };
        let case_error = primary_error
            .as_deref()
            .or(strict_recording_error.as_deref())
            .or(strict_checkpoint_error.as_deref())
            .or(shutdown_error.as_deref());
        let case_phase = if primary_error.is_some() {
            phase
        } else if strict_recording_error.is_some() {
            "recording"
        } else if strict_checkpoint_error.is_some() {
            "recording"
        } else if shutdown_error.is_some() {
            "cleanup"
        } else {
            phase
        };
        let case_status = if case_error.is_some() {
            "failed"
        } else {
            "passed"
        };
        let assertion_status = if phase == "scenario" {
            if primary.is_ok() { "passed" } else { "failed" }
        } else { "not_run" };
        std::fs::write(directory.join("assertion-status.txt"), assertion_status)?;
        artifacts::case_result(&directory, case_status, case_phase, case_error)?;
        if let Some(error) = primary_error {
            return Err(error.into());
        }
        if let Some(error) = shutdown_error {
            return Err(error.into());
        }
        if recorder::VisualRecorder::strict() {
            if let Some(error) = recording_error {
                return Err(error);
            }
        }
        if let Some(error) = strict_recording_error {
            return Err(error.into());
        }
        if let Some(error) = strict_checkpoint_error {
            return Err(error.into());
        }
        std::fs::write(directory.join("passed.txt"), name)?;
        println!("PASS {name}");
        Ok(())
    }

    fn context(&self, name: &str, driver: WebDriver) -> HarnessResult<ScenarioContext> {
        let artifacts = self.artifacts.join(name);
        std::fs::create_dir_all(&artifacts)?;
        Ok(ScenarioContext {
            browser: self.browser,
            driver,
            base_url: self.base_url.clone(),
            adapter_url: self.adapter_url.clone(),
            ai_url: self.ai_url.clone(),
            image_url: self.image_url.clone(),
            artifacts,
        })
    }

    pub async fn prove_failure_isolation(&self) -> HarnessResult<()> {
        for panic in [false, true] {
            let name = if panic {
                "expected-panic"
            } else {
                "expected-failure"
            };
            let result = self
                .run_case(name, move |context| async move {
                    context
                        .add_init_script("window.e2ePoisoned = true;")
                        .await?;
                    let driver = &context.driver;
                    driver
                        .execute(
                            crate::js_scripts::harness::MOD_PROVE_FAILURE_ISOLATION_01,
                            vec![],
                        )
                        .await?;
                    context.block_urls(&["*hanzi.duckdb"]).await?;
                    assert!(!panic, "deliberate isolation panic");
                    Err("deliberate isolation failure".into())
                })
                .await;
            if result.is_ok() {
                return Err("Deliberate failure was not reported".into());
            }

            artifacts::case_result(
                &self.artifacts.join(name),
                "passed",
                "expected_failure",
                None,
            )?;
            let driver = browser::start(self.browser).await?;
            let context = self.context(&format!("{name}-recovery"), driver.clone())?;
            let clean = context.prepare().await;
            let state = match clean {
                Ok(()) => driver
                    .execute(
                        crate::js_scripts::harness::MOD_PROVE_FAILURE_ISOLATION_02,
                        vec![],
                    )
                    .await?
                    .convert::<bool>()?,
                Err(error) => {
                    let _ = driver.quit().await;
                    return Err(error);
                }
            };
            driver.quit().await?;
            if !state {
                return Err(format!("State leaked into a fresh browser after {name}").into());
            }
            std::fs::write(
                context.artifacts.join("passed.txt"),
                "Fresh browser has independent state",
            )?;
            artifacts::case_result(&context.artifacts, "passed", "isolation", None)?;
        }
        Ok(())
    }

    pub async fn finish(mut self) -> HarnessResult<()> {
        if let Some(image_server) = self.image_server.take() {
            if let Err(error) = image_server.stop().await {
                self.failures
                    .push(format!("Server shutdown: image fixture: {error}"));
            }
        }
        if let Some(static_server) = self.static_server.take() {
            if let Err(error) = static_server.stop().await {
                self.failures
                    .push(format!("Server shutdown: static server: {error}"));
            }
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
        let mut cases = std::fs::read_dir(&self.artifacts)?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| std::fs::read(entry.path().join("scenario.json")).ok())
            .filter_map(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .collect::<Vec<_>>();
        cases.sort_by(|left, right| left["scenario"].as_str().cmp(&right["scenario"].as_str()));
        let suite = serde_json::json!({
            "schema": "zipeek-e2e-suite-v1",
            "browser": self.browser.as_str(),
            "passed": cases.iter().filter(|case| case["status"] == "passed").count(),
            "failed": cases.iter().filter(|case| case["status"] == "failed").count(),
            "scenarios": cases,
            "infrastructure_failures": self.failures.iter().filter(|error| error.starts_with("Server shutdown:")).collect::<Vec<_>>(),
            "reported_failures": &self.failures,
        });
        std::fs::write(
            self.artifacts.join("suite.json"),
            serde_json::to_vec_pretty(&suite).map_err(|error| error.to_string())?,
        )?;
        let review = format!(
            "# E2E run review\n\nBrowser: {}\n\nScenarios: {}\n\nSee `suite.json` for separate execution, assertion and recording states. Each scenario has an `agent-feedback.md` report.\n\n{}\n",
            self.browser.as_str(), cases.len(),
            cases.iter().map(|case| format!("- {}: {} (video: {})", case["scenario"].as_str().unwrap_or("unknown"), case["status"].as_str().unwrap_or("unknown"), case["recording_status"].as_str().unwrap_or("unavailable"))).collect::<Vec<_>>().join("\n")
        );
        std::fs::write(self.artifacts.join("review.md"), review)?;
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

    pub fn repository_root() -> PathBuf {
        Self::workspace()
            .parent()
            .expect("E2E workspace must be in repository")
            .to_path_buf()
    }

    pub fn workspace() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("E2E crate must reside inside the workspace crates directory")
            .to_path_buf()
    }
}

fn origin_of(url: &str) -> HarnessResult<&str> {
    let authority = url
        .strip_prefix("http://")
        .and_then(|value| value.split('/').next())
        .ok_or("E2E image fixture requires a loopback HTTP origin")?;
    if !authority.starts_with("127.0.0.1:") && !authority.starts_with("localhost:") {
        return Err("E2E image fixture requires a loopback application origin".into());
    }
    Ok(&url[.."http://".len() + authority.len()])
}

#[cfg(test)]
mod image_origin_tests {
    use super::origin_of;

    #[test]
    fn extracts_browser_origin_without_route_prefix() {
        assert_eq!(
            origin_of("http://127.0.0.1:3100/zipeek/").unwrap(),
            "http://127.0.0.1:3100"
        );
        assert!(origin_of("https://example.com/").is_err());
    }
}
