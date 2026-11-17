use std::sync::Arc;

use tauri::State;

use crate::services::{NativeServices, ServiceError, ServiceStatus};

#[tauri::command]
pub async fn start_service(
    service: String,
    state: State<'_, Arc<NativeServices>>,
) -> Result<ServiceStatus, ServiceError> {
    let services = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || services.start(&service))
        .await
        .map_err(|error| ServiceError::TaskFailure(error.to_string()))?
}

#[tauri::command]
pub async fn stop_service(
    service: String,
    state: State<'_, Arc<NativeServices>>,
) -> Result<ServiceStatus, ServiceError> {
    let services = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || services.stop(&service))
        .await
        .map_err(|error| ServiceError::TaskFailure(error.to_string()))?
}

#[tauri::command]
pub async fn service_status(
    service: String,
    state: State<'_, Arc<NativeServices>>,
) -> Result<ServiceStatus, ServiceError> {
    let services = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || services.status(&service))
        .await
        .map_err(|error| ServiceError::TaskFailure(error.to_string()))?
}
