use serde::Serialize;

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

