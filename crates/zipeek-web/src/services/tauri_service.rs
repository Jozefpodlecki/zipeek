use super::tauri::{TauriService, commands};

pub(crate) use super::tauri::commands::ServiceStatus;

async fn run<C>(service: &str) -> Result<ServiceStatus, String>
where
    C: super::tauri::Command<Args = commands::Service, Output = ServiceStatus>,
{
    TauriService::shared()
        .map_err(|error| error.to_string())?
        .execute::<C>(&commands::Service {
            service: service.to_owned(),
        })
        .await
        .map_err(|error| error.to_string())
}

pub async fn start(service: &str) -> Result<ServiceStatus, String> {
    run::<commands::StartService>(service).await
}

pub async fn stop(service: &str) -> Result<ServiceStatus, String> {
    run::<commands::StopService>(service).await
}

pub async fn status(service: &str) -> Result<ServiceStatus, String> {
    run::<commands::ServiceStatusCommand>(service).await
}
