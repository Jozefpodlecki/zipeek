mod error;
mod health;
mod process;
mod registry;

#[cfg(test)]
mod tests;

pub use error::ServiceError;
pub use registry::{NativeServices, ServicePhase, ServiceStatus};

use std::{sync::Arc, thread, time::Duration};

impl NativeServices {
    pub fn new() -> Self {
        let services = Self::default();
        let weak = Arc::downgrade(&services.0);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(5));
                let Some(registry) = weak.upgrade() else {
                    break;
                };
                NativeServices(registry).probe_health();
            }
        });
        services
    }

    pub async fn start(&self, service: String) -> Result<ServiceStatus, ServiceError> {
        let services = self.clone();
        tauri::async_runtime::spawn_blocking(move || services.start_blocking(&service))
            .await
            .map_err(|error| ServiceError::TaskFailure(error.to_string()))?
    }

    pub async fn stop(&self, service: String) -> Result<ServiceStatus, ServiceError> {
        let services = self.clone();
        tauri::async_runtime::spawn_blocking(move || services.stop_blocking(&service))
            .await
            .map_err(|error| ServiceError::TaskFailure(error.to_string()))?
    }

    pub async fn status(&self, service: String) -> Result<ServiceStatus, ServiceError> {
        let services = self.clone();
        tauri::async_runtime::spawn_blocking(move || services.status_blocking(&service))
            .await
            .map_err(|error| ServiceError::TaskFailure(error.to_string()))?
    }
}

