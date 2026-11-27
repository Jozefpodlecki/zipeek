use tauri::{State, command};

use super::error::CommandError;
use crate::services::{NativeServices, ServiceStatus};

#[command]
pub async fn start_service(
    service: String,
    services: State<'_, NativeServices>,
) -> Result<ServiceStatus, CommandError> {
    Ok(services.start(service).await?)
}

#[command]
pub async fn stop_service(
    service: String,
    services: State<'_, NativeServices>,
) -> Result<ServiceStatus, CommandError> {
    Ok(services.stop(service).await?)
}

#[command]
pub async fn service_status(
    service: String,
    services: State<'_, NativeServices>,
) -> Result<ServiceStatus, CommandError> {
    Ok(services.status(service).await?)
}
