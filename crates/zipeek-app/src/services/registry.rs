use std::{collections::{HashMap, HashSet}, net::SocketAddr, process::Child, sync::{Arc, Mutex}, time::Duration};
use serde::Serialize;
use super::{ServiceError, health::health_ready};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const SPEECH_STARTUP_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub(super) struct ServiceKey(pub(super) &'static str);

pub(super) struct ServiceDefinition {
    pub(super) name: &'static str,
    label: &'static str,
    binary: &'static str,
    override_variable: &'static str,
    port: u16,
}

pub(super) static SERVICE_DEFINITIONS: [ServiceDefinition; 3] = [
    ServiceDefinition {
        name: "ai",
        label: "AI (local Llama.cpp)",
        binary: "zipeek-ai-server",
        override_variable: "ZIPEEK_TAURI_AI_SERVER_EXE",
        port: 1430,
    },
    ServiceDefinition {
        name: "image",
        label: "Image generation server",
        binary: "zipeek-image-server",
        override_variable: "ZIPEEK_TAURI_IMAGE_SERVER_EXE",
        port: 1431,
    },
    ServiceDefinition {
        name: "speech",
        label: "Speech (TTS, STT and Live STT)",
        binary: "zipeek-tts-stt",
        override_variable: "ZIPEEK_TAURI_SPEECH_SERVER_EXE",
        port: 8799,
    },
];

impl ServiceKey {
    pub(super) fn parse(value: &str) -> Result<Self, ServiceError> {
        SERVICE_DEFINITIONS
            .iter()
            .find(|definition| definition.name == value)
            .map(|definition| Self(definition.name))
            .ok_or_else(|| ServiceError::UnknownService(format!(
                "Unknown service '{value}'. Expected 'ai', 'image' or 'speech'."
            )))
    }

    fn definition(self) -> &'static ServiceDefinition {
        SERVICE_DEFINITIONS
            .iter()
            .find(|definition| definition.name == self.0)
            .expect("service key originates from known definition")
    }

    pub(super) fn name(self) -> &'static str { self.0 }
    pub(super) fn label(self) -> &'static str { self.definition().label }
    pub(super) fn binary(self) -> &'static str { self.definition().binary }
    pub(super) fn override_variable(self) -> &'static str { self.definition().override_variable }
    pub(super) fn address(self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.definition().port))
    }
    pub(super) fn startup_timeout(self) -> Duration {
        if self.name() == "speech" || self.name() == "ai" { SPEECH_STARTUP_TIMEOUT } else { STARTUP_TIMEOUT }
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

pub(super) struct ManagedProcess {
    pub(super) child: Child,
}

#[derive(Default)]
pub(super) struct Registry {
    pub(super) processes: HashMap<ServiceKey, ManagedProcess>,
    pub(super) launching: HashSet<ServiceKey>,
    pub(super) stopping: HashSet<ServiceKey>,
    pub(super) failures: HashMap<ServiceKey, String>,
    pub(super) health: HashMap<ServiceKey, bool>,
}

#[derive(Default)]
pub(super) struct ServicesInner {
    pub(super) registry: Mutex<Registry>,
}

#[derive(Clone)]
pub struct NativeServices(pub(super) Arc<ServicesInner>);

impl Default for NativeServices {
    fn default() -> Self {
        Self(Arc::new(ServicesInner { registry: Mutex::new(Registry::default()) }))
    }
}


impl NativeServices {
    pub(super) fn lock(&self) -> Result<std::sync::MutexGuard<'_, Registry>, ServiceError> {
        self.0.registry
            .lock()
            .map_err(|_| ServiceError::RegistryUnavailable)
    }

    pub(super) fn probe_health(&self) {
        for kind in SERVICE_DEFINITIONS.iter().map(|definition| ServiceKey(definition.name)) {
            let ready = health_ready(kind);
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

    pub(super) fn start_blocking(&self, service: &str) -> Result<ServiceStatus, ServiceError> {
        let kind = ServiceKey::parse(service)?;
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
                    return self.status_blocking(service);
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
        self.status_blocking(service)
    }

    pub(super) fn stop_blocking(&self, service: &str) -> Result<ServiceStatus, ServiceError> {
        let kind = ServiceKey::parse(service)?;
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
        self.status_blocking(service)
    }

    pub(super) fn status_blocking(&self, service: &str) -> Result<ServiceStatus, ServiceError> {
        let kind = ServiceKey::parse(service)?;
        let healthy = health_ready(kind);
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


}

impl Drop for ServicesInner {
    fn drop(&mut self) {
        if let Ok(mut registry) = self.registry.lock() {
            for (_, mut process) in registry.processes.drain() {
                let _ = process.child.kill();
                let _ = process.child.wait();
            }
        }
    }
}

