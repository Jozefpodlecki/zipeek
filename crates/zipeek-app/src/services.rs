use std::{
    collections::{HashMap, HashSet},
    env,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;

const HEALTH_TIMEOUT: Duration = Duration::from_millis(500);
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const STARTUP_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "details", rename_all = "snake_case")]
pub enum ServiceError {
    UnknownService(String),
    StateTransition(String),
    NotOwned(String),
    PortOccupied(String),
    InvalidExecutablePath(String),
    ExecutableMissing(String),
    StartupTimeout(String),
    ProcessFailure(String),
    RegistryUnavailable,
    TaskFailure(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownService(message)
            | Self::StateTransition(message)
            | Self::NotOwned(message)
            | Self::PortOccupied(message)
            | Self::InvalidExecutablePath(message)
            | Self::ExecutableMissing(message)
            | Self::StartupTimeout(message)
            | Self::ProcessFailure(message)
            | Self::TaskFailure(message) => formatter.write_str(message),
            Self::RegistryUnavailable => formatter.write_str("Service registry lock failed"),
        }
    }
}

impl std::error::Error for ServiceError {}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum ServiceKind {
    AiFixture,
    Image,
}

impl ServiceKind {
    fn parse(value: &str) -> Result<Self, ServiceError> {
        match value {
            "ai" => Ok(Self::AiFixture),
            "image" => Ok(Self::Image),
            _ => Err(ServiceError::UnknownService(format!(
                "Unknown service '{value}'. Expected 'ai' or 'image'."
            ))),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::AiFixture => "ai",
            Self::Image => "image",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::AiFixture => "AI fixture server (testing only)",
            Self::Image => "Image generation server",
        }
    }

    fn binary(self) -> &'static str {
        match self {
            Self::AiFixture => "zipeek-ai-server",
            Self::Image => "zipeek-image-server",
        }
    }

    fn override_variable(self) -> &'static str {
        match self {
            Self::AiFixture => "ZIPEEK_TAURI_AI_SERVER_EXE",
            Self::Image => "ZIPEEK_TAURI_IMAGE_SERVER_EXE",
        }
    }

    fn address(self) -> SocketAddr {
        match self {
            Self::AiFixture => SocketAddr::from(([127, 0, 0, 1], 1430)),
            Self::Image => SocketAddr::from(([127, 0, 0, 1], 1431)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServicePhase {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
    External,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatus {
    pub service: &'static str,
    pub label: &'static str,
    pub phase: ServicePhase,
    pub running: bool,
    pub healthy: bool,
    pub owned: bool,
    pub pid: Option<u32>,
    pub message: String,
}

struct ManagedProcess {
    child: Child,
}

#[derive(Default)]
struct Registry {
    processes: HashMap<ServiceKind, ManagedProcess>,
    launching: HashSet<ServiceKind>,
    stopping: HashSet<ServiceKind>,
    failures: HashMap<ServiceKind, String>,
    health: HashMap<ServiceKind, bool>,
}

#[derive(Default)]
pub struct NativeServices {
    registry: Mutex<Registry>,
}

fn health_ready(address: SocketAddr) -> bool {
    let Ok(mut stream) = TcpStream::connect_timeout(&address, HEALTH_TIMEOUT) else {
        return false;
    };
    if stream.set_read_timeout(Some(HEALTH_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(HEALTH_TIMEOUT)).is_err()
    {
        return false;
    }
    let request = b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    if stream.write_all(request).is_err() {
        return false;
    }
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 1024];
    while bytes.len() < 16_384 {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
            Err(_) => break,
        }
    }
    let Ok(response) = std::str::from_utf8(&bytes) else {
        return false;
    };
    let Some((headers, body)) = response.split_once("\r\n\r\n") else {
        return false;
    };
    if !headers
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("HTTP/1.1 200 ") || line.starts_with("HTTP/1.0 200 "))
    {
        return false;
    }
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|json| {
            json.get("status")
                .and_then(|status| status.as_str())
                .map(|s| s == "ok")
        })
        .unwrap_or(false)
}

impl NativeServices {
    pub fn new() -> Arc<Self> {
        let services = Arc::new(Self::default());
        let weak = Arc::downgrade(&services);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(5));
                let Some(registry) = weak.upgrade() else {
                    break;
                };
                registry.probe_health();
            }
        });
        services
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Registry>, ServiceError> {
        self.registry
            .lock()
            .map_err(|_| ServiceError::RegistryUnavailable)
    }

    fn probe_health(&self) {
        for kind in [ServiceKind::AiFixture, ServiceKind::Image] {
            let ready = health_ready(kind.address());
            if let Ok(mut registry) = self.lock() {
                registry.health.insert(kind, ready);
                let exited = registry
                    .processes
                    .get_mut(&kind)
                    .and_then(|process| process.child.try_wait().ok().flatten());
                if let Some(exit) = exited {
                    registry.processes.remove(&kind);
                    if !registry.stopping.contains(&kind) {
                        registry
                            .failures
                            .insert(kind, format!("Service exited unexpectedly: {exit}"));
                    }
                }
            }
        }
    }

    fn binary_path(kind: ServiceKind) -> Result<PathBuf, ServiceError> {
        if let Some(path) = env::var_os(kind.override_variable()) {
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err(ServiceError::InvalidExecutablePath(format!(
                    "{} must be an absolute path",
                    kind.override_variable()
                )));
            }
            return Ok(path);
        }
        let mut path = env::current_exe().map_err(|error| {
            ServiceError::ProcessFailure(format!("Cannot locate application executable: {error}"))
        })?;
        path.set_file_name(format!("{}{}", kind.binary(), env::consts::EXE_SUFFIX));
        Ok(path)
    }

    fn launch(&self, kind: ServiceKind) -> Result<(), ServiceError> {
        if TcpStream::connect_timeout(&kind.address(), HEALTH_TIMEOUT).is_ok() {
            return Err(ServiceError::PortOccupied(format!(
                "Port {} is already occupied by an external process. Zipeek will not claim it.",
                kind.address()
            )));
        }
        let path = Self::binary_path(kind)?;
        if !path.is_file() {
            return Err(ServiceError::ExecutableMissing(format!(
                "Executable missing: {}. Build desktop sidecars or set {}.",
                path.display(),
                kind.override_variable()
            )));
        }
        let mut command = Command::new(&path);
        match kind {
            ServiceKind::AiFixture => {
                command.args(["--fixture", "--bind", "127.0.0.1:1430"]);
            }
            ServiceKind::Image => {
                command.env("ZIPEEK_IMAGE_BIND", "127.0.0.1:1431");
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                ServiceError::ProcessFailure(format!(
                    "Cannot launch {} ({}): {error}",
                    kind.binary(),
                    path.display()
                ))
            })?;
        self.lock()?
            .processes
            .insert(kind, ManagedProcess { child });

        let start = Instant::now();
        loop {
            {
                let mut registry = self.lock()?;
                let Some(process) = registry.processes.get_mut(&kind) else {
                    return Err(ServiceError::ProcessFailure(
                        "Managed process disappeared during startup".into(),
                    ));
                };
                if let Some(exit) = process
                    .child
                    .try_wait()
                    .map_err(|error| ServiceError::ProcessFailure(error.to_string()))?
                {
                    registry.processes.remove(&kind);
                    return Err(ServiceError::ProcessFailure(format!(
                        "{} exited before readiness: {exit}",
                        kind.binary()
                    )));
                }
            }
            if health_ready(kind.address()) {
                return Ok(());
            }
            if start.elapsed() >= STARTUP_TIMEOUT {
                self.terminate(kind)?;
                return Err(ServiceError::StartupTimeout(format!(
                    "{} did not become healthy at http://{}/health within {} seconds",
                    kind.binary(),
                    kind.address(),
                    STARTUP_TIMEOUT.as_secs()
                )));
            }
            thread::sleep(STARTUP_INTERVAL);
        }
    }

    fn terminate(&self, kind: ServiceKind) -> Result<(), ServiceError> {
        let mut registry = self.lock()?;
        if let Some(process) = registry.processes.get_mut(&kind) {
            if process
                .child
                .try_wait()
                .map_err(|error| ServiceError::ProcessFailure(error.to_string()))?
                .is_none()
            {
                process.child.kill().map_err(|error| {
                    ServiceError::ProcessFailure(format!(
                        "Unable to terminate owned process: {error}"
                    ))
                })?;
            }
            process.child.wait().map_err(|error| {
                ServiceError::ProcessFailure(format!("Unable to reap owned process: {error}"))
            })?;
            registry.processes.remove(&kind);
        }
        Ok(())
    }

    pub fn start(&self, service: &str) -> Result<ServiceStatus, ServiceError> {
        let kind = ServiceKind::parse(service)?;
        {
            let mut registry = self.lock()?;
            if registry.launching.contains(&kind) || registry.stopping.contains(&kind) {
                return Err(ServiceError::StateTransition(format!(
                    "{} is already changing state",
                    kind.label()
                )));
            }
            if let Some(process) = registry.processes.get_mut(&kind) {
                if process
                    .child
                    .try_wait()
                    .map_err(|error| ServiceError::ProcessFailure(error.to_string()))?
                    .is_none()
                {
                    drop(registry);
                    return self.status(service);
                }
                registry.processes.remove(&kind);
            }
            registry.failures.remove(&kind);
            registry.launching.insert(kind);
        }
        let result = self.launch(kind);
        {
            let mut registry = self.lock()?;
            registry.launching.remove(&kind);
            if let Err(message) = &result {
                registry.failures.insert(kind, message.to_string());
            }
        }
        result?;
        self.status(service)
    }

    pub fn stop(&self, service: &str) -> Result<ServiceStatus, ServiceError> {
        let kind = ServiceKind::parse(service)?;
        {
            let mut registry = self.lock()?;
            if registry.launching.contains(&kind) || registry.stopping.contains(&kind) {
                return Err(ServiceError::StateTransition(format!(
                    "{} is already changing state",
                    kind.label()
                )));
            }
            if !registry.processes.contains_key(&kind) {
                return Err(ServiceError::NotOwned(format!(
                    "{} is not owned by Zipeek; an external process cannot be stopped",
                    kind.label()
                )));
            }
            registry.stopping.insert(kind);
        }
        let result = self.terminate(kind);
        {
            let mut registry = self.lock()?;
            registry.stopping.remove(&kind);
            if let Err(message) = &result {
                registry.failures.insert(kind, message.to_string());
            } else {
                registry.failures.remove(&kind);
            }
        }
        result?;
        self.status(service)
    }

    pub fn status(&self, service: &str) -> Result<ServiceStatus, ServiceError> {
        let kind = ServiceKind::parse(service)?;
        let healthy = health_ready(kind.address());
        let mut registry = self.lock()?;
        registry.health.insert(kind, healthy);
        let mut exit = None;
        let pid = if let Some(process) = registry.processes.get_mut(&kind) {
            match process
                .child
                .try_wait()
                .map_err(|error| ServiceError::ProcessFailure(error.to_string()))?
            {
                None => Some(process.child.id()),
                Some(code) => {
                    exit = Some(code);
                    None
                }
            }
        } else {
            None
        };
        if let Some(exit) = exit {
            registry.processes.remove(&kind);
            if !registry.stopping.contains(&kind) {
                registry
                    .failures
                    .insert(kind, format!("Service exited unexpectedly: {exit}"));
            }
        }
        let phase = if registry.stopping.contains(&kind) {
            ServicePhase::Stopping
        } else if registry.launching.contains(&kind) {
            ServicePhase::Starting
        } else if pid.is_some() && healthy {
            ServicePhase::Running
        } else if pid.is_some() {
            ServicePhase::Starting
        } else if healthy {
            ServicePhase::External
        } else if registry.failures.contains_key(&kind) {
            ServicePhase::Failed
        } else {
            ServicePhase::Stopped
        };
        let message = match phase {
            ServicePhase::Running => "Running and healthy".into(),
            ServicePhase::Starting => "Starting; waiting for the health endpoint".into(),
            ServicePhase::Stopping => "Stopping service".into(),
            ServicePhase::External => {
                "An external service is healthy; Zipeek does not own it".into()
            }
            ServicePhase::Stopped => "Stopped".into(),
            ServicePhase::Failed => registry.failures.get(&kind).cloned().unwrap_or_default(),
        };
        Ok(ServiceStatus {
            service: kind.name(),
            label: kind.label(),
            phase,
            running: pid.is_some(),
            healthy,
            owned: pid.is_some(),
            pid,
            message,
        })
    }

    pub fn shutdown(&self) {
        if let Ok(mut registry) = self.lock() {
            for (_, mut process) in registry.processes.drain() {
                let _ = process.child.kill();
                let _ = process.child.wait();
            }
            registry.launching.clear();
            registry.stopping.clear();
        }
    }
}

impl Drop for NativeServices {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn service_error_serialization_keeps_category() {
        let missing = ServiceError::ExecutableMissing("missing binary".into());
        assert_eq!(
            serde_json::to_value(missing).expect("serialize"),
            serde_json::json!({"kind": "executable_missing", "details": "missing binary"})
        );
        assert_eq!(
            serde_json::to_value(ServiceError::RegistryUnavailable).expect("serialize"),
            serde_json::json!({"kind": "registry_unavailable"})
        );
    }

    #[test]
    fn known_service_names_and_loopback_addresses() {
        for kind in [ServiceKind::AiFixture, ServiceKind::Image] {
            assert_eq!(ServiceKind::parse(kind.name()), Ok(kind));
            assert!(kind.address().ip().is_loopback());
        }
        assert!(ServiceKind::parse("../server").is_err());
    }

    #[test]
    fn empty_registry_stops_neither_service() {
        let services = NativeServices::default();
        assert!(services.stop("ai").is_err());
        assert!(services.stop("image").is_err());
    }

    #[test]
    fn occupied_external_port_is_not_owned_or_stopped() {
        let address = ServiceKind::Image.address();
        let Ok(listener) = TcpListener::bind(address) else {
            return;
        };
        let services = NativeServices::default();
        assert!(services.start("image").is_err());
        assert!(services.stop("image").is_err());
        assert!(TcpStream::connect_timeout(&address, HEALTH_TIMEOUT).is_ok());
        drop(listener);
    }

    #[test]
    fn non_health_http_response_is_not_readiness() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept connection");
            let mut request = [0; 256];
            let _ = stream.read(&mut request);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 17\r\n\r\n{\"status\":\"bad\"}")
                .expect("write response");
        });
        assert!(!health_ready(address));
        worker.join().expect("join worker");
    }

    #[test]
    fn shutdown_reaps_only_owned_children() {
        #[cfg(windows)]
        let mut command = {
            let mut command = Command::new("cmd");
            command.args(["/C", "ping -n 30 127.0.0.1 >NUL"]);
            command
        };
        #[cfg(not(windows))]
        let mut command = {
            let mut command = Command::new("sleep");
            command.arg("30");
            command
        };
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn owned test child");
        let services = NativeServices::default();
        services
            .lock()
            .expect("registry")
            .processes
            .insert(ServiceKind::Image, ManagedProcess { child });
        services.shutdown();
        assert!(services.lock().expect("registry").processes.is_empty());
    }

    #[test]
    fn valid_health_response_is_readiness() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept connection");
            let mut request = [0; 256];
            let _ = stream.read(&mut request);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\n\r\n{\"status\":\"ok\"}")
                .expect("write response");
        });
        assert!(health_ready(address));
        worker.join().expect("join worker");
    }
}
