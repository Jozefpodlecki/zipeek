use std::{
    fs::File,
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[cfg(test)]
use std::io::{Read, Write};

use super::ai_fixture::AiFixtureServer;
use super::{Harness, HarnessResult};

#[derive(Clone, Copy)]
pub enum SuiteKind {
    Root,
    Pages,
}

impl SuiteKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Pages => "pages",
        }
    }
}

pub(super) struct OwnedProcess {
    child: Child,
    log: PathBuf,
}

pub(super) enum OwnedServer {
    Process(OwnedProcess),
    Ai(AiFixtureServer),
}

pub(super) struct BuiltWeb {
    pub servers: Vec<OwnedServer>,
    pub dist: Option<PathBuf>,
    pub adapter_url: String,
    pub ai_url: String,
}

impl OwnedServer {
    pub(super) fn stop(&mut self) -> HarnessResult<()> {
        match self {
            Self::Process(process) => process.stop(),
            Self::Ai(server) => server.stop(),
        }
    }
}

fn hidden(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
}

impl OwnedProcess {
    fn spawn(command: &mut Command, log: &Path) -> HarnessResult<Self> {
        hidden(command);
        let output = File::create(log)?;
        command
            .stdout(Stdio::from(output.try_clone()?))
            .stderr(Stdio::from(output))
            .stdin(Stdio::null());
        let child = command
            .spawn()
            .map_err(|error| format!("Cannot spawn {command:?}: {error}; log {}", log.display()))?;
        let process = Self {
            child,
            log: log.to_owned(),
        };
        std::fs::write(log.with_extension("pid"), process.child.id().to_string())?;
        Ok(process)
    }

    pub(super) fn stop(&mut self) -> HarnessResult<()> {
        if self.child.try_wait()?.is_some() {
            return Ok(());
        }
        #[cfg(windows)]
        {
            let mut command = Command::new("taskkill");
            command.args(["/PID", &self.child.id().to_string(), "/T", "/F"]);
            hidden(&mut command);
            let result = command.output()?;
            if !result.status.success() && self.child.try_wait()?.is_none() {
                self.child.kill()?;
                self.child.wait()?;
                return Err(format!(
                    "Tree cleanup denied for owned PID {}; direct child reaped: {}",
                    self.child.id(),
                    String::from_utf8_lossy(&result.stderr)
                )
                .into());
            }
        }
        #[cfg(unix)]
        {
            let result = Command::new("kill")
                .args(["-KILL", "--", &format!("-{}", self.child.id())])
                .status()?;
            if !result.success() && self.child.try_wait()?.is_none() {
                return Err(format!("Failed to stop process group {}", self.child.id()).into());
            }
        }
        self.child.wait()?;
        std::fs::write(
            self.log.with_extension("reaped"),
            self.child.id().to_string(),
        )?;
        Ok(())
    }

    fn ready(&mut self, port: u16, timeout: Duration) -> HarnessResult<()> {
        self.wait_ready(port, timeout, |_, _| true)
    }

    #[cfg(test)]
    fn http_ready(&mut self, port: u16, timeout: Duration) -> HarnessResult<()> {
        self.wait_ready(port, timeout, |mut stream, remaining| {
            if remaining.is_zero() {
                return false;
            }
            let wait = remaining.min(Duration::from_millis(200));
            let mut status = [0u8; 12];
            stream.set_read_timeout(Some(wait)).is_ok()
                && stream.set_write_timeout(Some(wait)).is_ok()
                && stream
                    .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
                    .is_ok()
                && stream.read_exact(&mut status).is_ok()
                && matches!(&status, b"HTTP/1.1 200" | b"HTTP/1.0 200")
        })
    }

    fn wait_ready(
        &mut self,
        port: u16,
        timeout: Duration,
        mut probe: impl FnMut(TcpStream, Duration) -> bool,
    ) -> HarnessResult<()> {
        let deadline = Instant::now() + timeout;
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let mut backoff = Duration::from_millis(100);
        loop {
            if let Some(status) = self.child.try_wait()? {
                return Err(format!(
                    "Server exited with {status}; {}\n{}",
                    self.log.display(),
                    std::fs::read_to_string(&self.log)?
                )
                .into());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(format!(
                    "Server readiness timed out on {address}; {}\n{}",
                    self.log.display(),
                    std::fs::read_to_string(&self.log)?
                )
                .into());
            }
            if TcpStream::connect_timeout(&address, remaining.min(Duration::from_millis(200)))
                .is_ok_and(|stream| {
                    probe(stream, deadline.saturating_duration_since(Instant::now()))
                })
            {
                return Ok(());
            }
            std::thread::sleep(backoff.min(deadline.saturating_duration_since(Instant::now())));
            backoff = (backoff * 2).min(Duration::from_secs(2));
        }
    }

    fn complete(&mut self, timeout: Duration) -> HarnessResult<()> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.child.try_wait()? {
                return if status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "Build exited with {status}; {}\n{}",
                        self.log.display(),
                        std::fs::read_to_string(&self.log)?
                    )
                    .into())
                };
            }
            if Instant::now() >= deadline {
                return Err(format!("Build timed out; see {}", self.log.display()).into());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for OwnedProcess {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!("Owned process cleanup failed: {error}");
        }
    }
}

fn available_port() -> HarnessResult<u16> {
    Ok(TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?
        .local_addr()?
        .port())
}

fn fixture(dist: &Path, prefix: &str, output: &Path) -> HarnessResult<(OwnedProcess, String)> {
    let port = available_port()?;
    let root = Harness::workspace();
    let mut command = Command::new("node");
    command
        .arg(root.join("zipeek-e2e/fixtures/pages-server.cjs"))
        .arg(dist)
        .arg(port.to_string())
        .arg(prefix)
        .arg(&root);
    let mut process = OwnedProcess::spawn(&mut command, &output.join("fixture-server.log"))?;
    process.ready(port, Duration::from_secs(30))?;
    Ok((process, format!("http://127.0.0.1:{port}{prefix}")))
}

fn dictionary_asset(root: &Path) -> HarnessResult<(String, u64)> {
    let static_dir = root.join("zipeek-web/static");
    let manifest_path = static_dir.join("hanzi.manifest.json");
    let manifest_text = std::fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "E2E dictionary preflight: cannot read {}: {error}. Run `cargo make db-generate`, `cargo make db-compress`, and `cargo make db-install`.",
            manifest_path.display()
        )
    })?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_text)
        .map_err(|error| format!("E2E dictionary preflight: invalid {}: {error}", manifest_path.display()))?;
    let files = manifest["files"]
        .as_array()
        .ok_or("E2E dictionary preflight: manifest.files must be an array")?;
    let compressed = files.iter().filter(|entry| {
        entry["path"].as_str().is_some_and(|path| path.ends_with(".duckdb.gz"))
    }).collect::<Vec<_>>();
    if compressed.len() != 1 {
        return Err(format!(
            "E2E dictionary preflight: expected one compressed DuckDB entry in {}, found {}",
            manifest_path.display(), compressed.len()
        ).into());
    }
    let entry = compressed[0];
    let filename = entry["path"]
        .as_str()
        .ok_or("E2E dictionary preflight: compressed entry has no path")?;
    if !filename.starts_with("hanzi-")
        || !filename.ends_with(".duckdb.gz")
        || !filename.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.'))
    {
        return Err(format!("E2E dictionary preflight: invalid dictionary asset name {filename:?}").into());
    }
    let expected_size = entry["size_bytes"]
        .as_u64()
        .filter(|size| *size > 0)
        .ok_or("E2E dictionary preflight: compressed database size must be positive")?;
    verify_dictionary_file(&static_dir, filename, expected_size)?;
    Ok((filename.to_owned(), expected_size))
}

fn verify_dictionary_file(directory: &Path, filename: &str, expected_size: u64) -> HarnessResult<()> {
    let path = directory.join(filename);
    let size = std::fs::metadata(&path).map_err(|error| {
        format!(
            "E2E dictionary preflight: {} unavailable: {error}. Run `cargo make db-generate`, `cargo make db-compress`, and `cargo make db-install`.",
            path.display()
        )
    })?.len();
    if size != expected_size {
        return Err(format!(
            "E2E dictionary preflight: {} has {size} bytes, manifest requires {expected_size}. Run `cargo make db-compress` and `cargo make db-install`.",
            path.display()
        ).into());
    }
    Ok(())
}

fn verify_web_prerequisites(root: &Path) -> HarnessResult<()> {
    let web = root.join("zipeek-web");
    if !web.join("Trunk.toml").is_file() {
        return Err(format!(
            "Missing Trunk.toml in {}; check the workspace layout",
            web.display()
        )
        .into());
    }
    for relative in ["../zipeek-core", "../../Cargo.toml", "../../Cargo.lock"] {
        let target = web.join(relative);
        if !target.exists() {
            return Err(format!(
                "Trunk watch prerequisite missing: {}; check crates/zipeek-web/Trunk.toml",
                target.display()
            )
            .into());
        }
    }
    let output = Command::new("node")
        .current_dir(&web)
        .args([
            "-e",
            "require.resolve('esbuild'); require.resolve('@duckdb/duckdb-wasm'); require.resolve('chart.js')",
        ])
        .output()
        .map_err(|error| format!("Node.js is required for Trunk pre-build hooks: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Web build dependencies unavailable in {}. Run `cd crates/zipeek-web; npm ci --ignore-scripts` before E2E tests. Node said: {}",
            web.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ).into());
    }
    Ok(())
}

pub(super) fn start(kind: SuiteKind, output: &Path) -> HarnessResult<BuiltWeb> {
    let root = Harness::workspace();
    let (dictionary_name, dictionary_size) = dictionary_asset(&root)?;
    verify_web_prerequisites(&root)?;
    let dist = output.join("dist");
    let mut command = Command::new("trunk");
    command
        .current_dir(root.join("zipeek-web"))
        .args(["build", "--dist"])
        .arg(&dist);
    if matches!(kind, SuiteKind::Pages) {
        command.args(["--public-url", "/zipeek/"]);
    }
    command.env("ZIPEEK_E2E_SPEECH", "1");
    let mut build = OwnedProcess::spawn(&mut command, &output.join("build.log"))?;
    build.complete(Duration::from_secs(300))?;
    verify_dictionary_file(&dist, &dictionary_name, dictionary_size)?;
    let prefix = if matches!(kind, SuiteKind::Pages) {
        "/zipeek"
    } else {
        ""
    };
    let (fixture, adapter_url) = fixture(&dist, prefix, output)?;
    let ai = AiFixtureServer::start()?;
    let ai_url = ai.url();

    Ok(BuiltWeb {
        servers: vec![OwnedServer::Process(fixture), OwnedServer::Ai(ai)],
        dist: matches!(kind, SuiteKind::Root).then_some(dist),
        adapter_url,
        ai_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_preflight_rejects_missing_and_incorrect_assets() -> HarnessResult<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-e2e-db-preflight-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
        ));
        let static_dir = root.join("zipeek-web/static");
        std::fs::create_dir_all(&static_dir)?;
        let missing = dictionary_asset(&root).unwrap_err().to_string();
        assert!(missing.contains("db-install"), "{missing}");
        std::fs::write(static_dir.join("hanzi.manifest.json"), "not json")?;
        assert!(dictionary_asset(&root).unwrap_err().to_string().contains("invalid"));
        std::fs::write(
            static_dir.join("hanzi.manifest.json"),
            r#"{"files":[{"path":"hanzi-test.duckdb.gz","size_bytes":4}]}"#,
        )?;
        let missing = dictionary_asset(&root).unwrap_err().to_string();
        assert!(missing.contains("hanzi-test.duckdb.gz"), "{missing}");
        std::fs::write(static_dir.join("hanzi-test.duckdb.gz"), b"abc")?;
        assert!(dictionary_asset(&root).unwrap_err().to_string().contains("3 bytes"));
        std::fs::write(static_dir.join("hanzi-test.duckdb.gz"), b"abcd")?;
        assert_eq!(dictionary_asset(&root)?, ("hanzi-test.duckdb.gz".into(), 4));
        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn dictionary_preflight_rejects_unsafe_manifest_paths() -> HarnessResult<()> {
        let root = std::env::temp_dir().join(format!(
            "zipeek-e2e-db-unsafe-{}",
            std::process::id()
        ));
        let static_dir = root.join("zipeek-web/static");
        std::fs::create_dir_all(&static_dir)?;
        std::fs::write(
            static_dir.join("hanzi.manifest.json"),
            r#"{"files":[{"path":"../hanzi-test.duckdb.gz","size_bytes":4}]}"#,
        )?;
        assert!(dictionary_asset(&root).unwrap_err().to_string().contains("invalid dictionary asset name"));
        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn missing_web_layout_fails_before_launching_trunk() {
        let root = std::env::temp_dir().join("zipeek-e2e-nonexistent-web-layout");
        assert!(
            verify_web_prerequisites(&root)
                .unwrap_err()
                .to_string()
                .contains("Trunk.toml")
        );
    }

    #[test]
    fn listening_http_server_is_not_ready_until_it_serves_the_app() -> HarnessResult<()> {
        let output = super::super::artifacts::run_directory("http-readiness")?;
        let port = available_port()?;
        let mut command = Command::new("node");
        command
            .arg(Harness::workspace().join("zipeek-e2e/fixtures/lifecycle.cjs"))
            .args(["http", &port.to_string()]);
        let mut process = OwnedProcess::spawn(&mut command, &output.join("http.log"))?;
        process.http_ready(port, Duration::from_secs(3))?;
        process.stop()?;
        let log = std::fs::read_to_string(output.join("http.log"))?;
        assert!(log.contains("HTTP readiness 404"), "{log}");
        assert!(log.contains("HTTP readiness 200"), "{log}");
        Ok(())
    }

    fn sleeper(output: &Path, mode: &str) -> HarnessResult<OwnedProcess> {
        let mut command = Command::new("node");
        command
            .arg(Harness::workspace().join("zipeek-e2e/fixtures/lifecycle.cjs"))
            .arg(mode);
        OwnedProcess::spawn(&mut command, &output.join(format!("{mode}.log")))
    }

    #[test]
    fn readiness_failure_reaps_owned_process_and_preserves_unrelated_listener() -> HarnessResult<()>
    {
        let output = super::super::artifacts::run_directory("startup-failure")?;
        let unrelated = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let unused = available_port()?;
        let mut process = sleeper(&output, "hang")?;
        let start = Instant::now();
        assert!(process.ready(unused, Duration::from_millis(180)).is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
        process.stop()?;
        assert!(process.child.try_wait()?.is_some());
        assert!(TcpStream::connect(unrelated.local_addr()?).is_ok());
        Ok(())
    }

    #[test]
    fn early_server_exit_reports_original_status_and_output() -> HarnessResult<()> {
        let output = super::super::artifacts::run_directory("early-exit")?;
        let mut process = sleeper(&output, "exit")?;
        let error = process
            .ready(available_port()?, Duration::from_secs(3))
            .expect_err("fixture must fail")
            .to_string();
        assert!(error.contains("intentional startup failure"), "{error}");
        assert!(error.contains("17"), "{error}");
        process.stop()?;
        Ok(())
    }

    #[test]
    fn dropping_guard_reaps_process_even_during_unwind() -> HarnessResult<()> {
        let output = super::super::artifacts::run_directory("unwind")?;
        let process = sleeper(&output, "hang")?;
        let pid = process.child.id();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _guard = process;
            panic!("intentional unwind");
        }));
        assert!(result.is_err());
        assert_eq!(
            std::fs::read_to_string(output.join("hang.reaped"))?,
            pid.to_string()
        );
        Ok(())
    }

    #[test]
    fn dropping_server_guard_stops_descendant_listener() -> HarnessResult<()> {
        let output = super::super::artifacts::run_directory("descendant")?;
        let port = available_port()?;
        let mut command = Command::new("node");
        command
            .arg(Harness::workspace().join("zipeek-e2e/fixtures/lifecycle.cjs"))
            .args(["tree", &port.to_string()]);
        let mut process = OwnedProcess::spawn(&mut command, &output.join("tree.log"))?;
        process.ready(port, Duration::from_secs(3))?;
        drop(process);
        assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, port)).is_err());
        assert!(output.join("tree.reaped").is_file());
        Ok(())
    }
}
