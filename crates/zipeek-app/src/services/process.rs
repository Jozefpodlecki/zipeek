use super::{
    NativeServices, ServiceError,
    health::{HEALTH_TIMEOUT, health_ready},
    registry::{ManagedProcess, ServiceKey},
};
use std::{
    env,
    net::TcpStream,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

const STARTUP_INTERVAL: Duration = Duration::from_millis(200);

impl NativeServices {
    pub(super) fn binary_path(kind: ServiceKey) -> Result<PathBuf, ServiceError> {
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

    pub(super) fn launch(&self, kind: ServiceKey) -> Result<(), ServiceError> {
        if self.0.closing.load(Ordering::Acquire) {
            return Err(ServiceError::StateTransition("Zipeek is closing".into()));
        }
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
            ServiceKey("ai") => {
                command.args(["--bind", "127.0.0.1:1430"]);
            }
            ServiceKey("image") => {
                command.env("ZIPEEK_IMAGE_BIND", "127.0.0.1:1431");
            }
            ServiceKey("speech") => {
                command.env("ZIPEEK_SPEECH_PORT", "8799");
            }
            _ => unreachable!("validated service key"),
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let mut registry = self.lock()?;
        if self.0.closing.load(Ordering::Acquire) {
            return Err(ServiceError::StateTransition("Zipeek is closing".into()));
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
        registry.processes.insert(kind, ManagedProcess { child });
        drop(registry);

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
                        "{} exited before readiness: {exit}{}",
                        kind.binary(),
                        if kind.name() == "ai" {
                            " (configure a GGUF model in models/ai or ZIPEEK_AI_MODEL_PATH)"
                        } else if kind.name() == "image" {
                            " (SDXL requires model files in models/sdxl or ZIPEEK_IMAGE_MODEL_DIR and a working CUDA device; check image-server logs)"
                        } else {
                            ""
                        }
                    )));
                }
            }
            if health_ready(kind) {
                return Ok(());
            }
            if start.elapsed() >= kind.startup_timeout() {
                self.terminate(kind)?;
                return Err(ServiceError::StartupTimeout(format!(
                    "{} did not become healthy at http://{}/health within {} seconds",
                    kind.binary(),
                    kind.address(),
                    kind.startup_timeout().as_secs()
                )));
            }
            thread::sleep(STARTUP_INTERVAL);
        }
    }

    pub(super) fn terminate(&self, kind: ServiceKey) -> Result<(), ServiceError> {
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

    pub fn begin_shutdown(&self) {
        self.0.closing.store(true, Ordering::Release);
    }

    pub fn shutdown(&self) -> Result<(), ServiceError> {
        self.begin_shutdown();
        let mut registry = self.lock()?;
        let mut failures = Vec::new();
        for process in registry.processes.values_mut() {
            match process.child.try_wait() {
                Ok(Some(_)) => {}
                Ok(None) => {
                    if let Err(error) = process.child.kill() {
                        failures.push(error.to_string());
                    }
                }
                Err(error) => failures.push(error.to_string()),
            }
        }
        for process in registry.processes.values_mut() {
            if let Err(error) = process.child.wait() {
                failures.push(error.to_string());
            }
        }
        registry
            .processes
            .retain(|_, process| !matches!(process.child.try_wait(), Ok(Some(_))));
        registry.launching.clear();
        registry.stopping.clear();
        if failures.is_empty() {
            Ok(())
        } else {
            Err(ServiceError::ProcessFailure(failures.join("; ")))
        }
    }
}
