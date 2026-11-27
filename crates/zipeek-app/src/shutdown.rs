use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, WindowEvent};

use crate::services::{NativeServices, ServiceError};

const SHUTDOWN_EVENT: &str = "zipeek-shutdown";
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

fn wait_for_cleanup(
    receiver: mpsc::Receiver<Result<(), ServiceError>>,
    timeout: Duration,
) -> Result<(), String> {
    receiver
        .recv_timeout(timeout)
        .map_err(|error| format!("Service shutdown did not complete: {error}"))?
        .map_err(|error| format!("Service shutdown failed: {error}"))
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShutdownPhase {
    #[default]
    Idle,
    Closing,
    Failed,
}

#[derive(Default)]
pub struct Shutdown {
    phase: Mutex<ShutdownPhase>,
    exit_allowed: AtomicBool,
}

impl Shutdown {
    fn phase(&self) -> ShutdownPhase {
        *self.phase.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn begin(&self) -> bool {
        let mut phase = self.phase.lock().unwrap_or_else(|error| error.into_inner());
        if *phase != ShutdownPhase::Idle {
            return false;
        }
        *phase = ShutdownPhase::Closing;
        true
    }
}

#[tauri::command]
pub fn shutdown_status(shutdown: State<'_, Shutdown>) -> ShutdownPhase {
    shutdown.phase()
}

#[tauri::command]
pub fn shutdown_force_exit(app: AppHandle, shutdown: State<'_, Shutdown>) -> Result<(), String> {
    if shutdown.phase() != ShutdownPhase::Failed {
        return Err("Close now is available only after shutdown fails".into());
    }
    shutdown.exit_allowed.store(true, Ordering::Release);
    app.exit(0);
    Ok(())
}

fn request(app: &AppHandle) {
    if app.state::<Shutdown>().phase() == ShutdownPhase::Failed {
        app.state::<Shutdown>()
            .exit_allowed
            .store(true, Ordering::Release);
        app.exit(0);
        return;
    }
    if !app.state::<Shutdown>().begin() {
        return;
    }
    let services = app.state::<NativeServices>().inner().clone();
    services.begin_shutdown();
    if let Err(error) = app.emit(SHUTDOWN_EVENT, ShutdownPhase::Closing) {
        eprintln!("Unable to show shutdown screen: {error}");
    }
    let app = app.clone();
    thread::spawn(move || {
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let _ = sender.send(services.shutdown());
        });
        match wait_for_cleanup(receiver, SHUTDOWN_TIMEOUT) {
            Ok(()) => {
                app.state::<Shutdown>()
                    .exit_allowed
                    .store(true, Ordering::Release);
                app.exit(0);
            }
            Err(error) => {
                eprintln!("{error}");
                *app.state::<Shutdown>()
                    .phase
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = ShutdownPhase::Failed;
                if let Err(error) = app.emit(SHUTDOWN_EVENT, ShutdownPhase::Failed) {
                    eprintln!("Unable to show shutdown failure: {error}");
                    app.state::<Shutdown>()
                        .exit_allowed
                        .store(true, Ordering::Release);
                    app.exit(1);
                }
            }
        }
    });
}

pub fn handle_event(app: &AppHandle, event: RunEvent) {
    if app.state::<Shutdown>().exit_allowed.load(Ordering::Acquire) {
        return;
    }
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } if label == "main" => {
            api.prevent_close();
            request(app);
        }
        RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
            request(app);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_close_requests_do_not_restart_shutdown() {
        let shutdown = Shutdown::default();
        assert!(shutdown.begin());
        assert!(!shutdown.begin());
        assert!(shutdown.phase() == ShutdownPhase::Closing);
        *shutdown.phase.lock().unwrap() = ShutdownPhase::Failed;
        assert!(!shutdown.begin());
        assert!(!shutdown.exit_allowed.load(Ordering::Acquire));
    }

    #[test]
    fn cleanup_timeout_does_not_wait_for_a_stalled_worker() {
        let (_sender, receiver) = mpsc::sync_channel(1);
        assert!(wait_for_cleanup(receiver, Duration::ZERO).is_err());
    }

    #[test]
    fn cleanup_reports_worker_failures_and_success() {
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(Err(ServiceError::RegistryUnavailable)).unwrap();
        assert!(
            wait_for_cleanup(receiver, Duration::ZERO)
                .unwrap_err()
                .contains("Service registry lock failed")
        );
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(Ok(())).unwrap();
        assert!(wait_for_cleanup(receiver, Duration::ZERO).is_ok());
        let (sender, receiver) = mpsc::sync_channel(1);
        drop(sender);
        assert!(wait_for_cleanup(receiver, Duration::ZERO).is_err());
    }
}
