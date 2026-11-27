use std::{cell::Cell, rc::Rc};

use futures::future::{AbortHandle, Abortable};

use gloo::{events::EventListener, timers::callback::Interval};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    hooks::use_app_environment,
    services::{
        image_generation,
        tauri_service::{self, ServiceStatus},
    },
};

mod probe;
pub(crate) use probe::with_timeout;

const POLL_TICK_MS: u32 = 5_000;
const HEALTHY_POLL_TICKS: u32 = 6;
const CONFIGURATION_CHANGED: &str = "zipeek-service-configuration-changed";

fn next_probe_ticks(succeeded: bool, prior_failures: u32) -> (u32, u32) {
    if succeeded {
        (0, HEALTHY_POLL_TICKS)
    } else {
        let failures = prior_failures.saturating_add(1);
        (failures, failures.clamp(1, HEALTHY_POLL_TICKS))
    }
}

fn schedule_next_probe(
    succeeded: bool,
    failures: &Cell<u32>,
    next_poll_tick: &Cell<u32>,
    ticks: &Cell<u32>,
) {
    let (count, delay) = next_probe_ticks(succeeded, failures.get());
    failures.set(count);
    next_poll_tick.set(ticks.get().saturating_add(delay));
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceAvailability {
    Checking,
    Available,
    Unavailable,
    Failed(String),
}

impl ServiceAvailability {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Checking => "Checking",
            Self::Available => "Available",
            Self::Unavailable => "Unavailable",
            Self::Failed(_) => "Check failed",
        }
    }

    pub fn details(&self) -> Option<&str> {
        match self {
            Self::Failed(message) => Some(message),
            Self::Unavailable => Some("Service not reachable at the configured address"),
            _ => None,
        }
    }
}

pub(crate) fn notify_configuration_changed() {
    if let Some(window) = web_sys::window() {
        if let Ok(event) = web_sys::Event::new(CONFIGURATION_CHANGED) {
            let _ = window.dispatch_event(&event);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceOperation {
    Starting,
    Stopping,
}

#[derive(Clone, PartialEq)]
pub struct ServiceHandle {
    pub name: &'static str,
    pub status: Option<ServiceStatus>,
    pub availability: ServiceAvailability,
    pub busy: bool,
    pub operation: Option<ServiceOperation>,
    pub error: Option<String>,
    pub managed: bool,
    pub start: Callback<()>,
    pub stop: Callback<()>,
    pub refresh: Callback<()>,
}

impl ServiceHandle {
    pub fn supports_start_stop(&self) -> bool {
        self.managed
    }

    pub fn can_start(&self) -> bool {
        self.managed
            && !self.busy
            && self.status.as_ref().is_some_and(|status| {
                matches!(status.phase.as_str(), "stopped" | "failed")
                    && !(status.healthy && !status.owned)
            })
    }

    pub fn can_stop(&self) -> bool {
        self.managed
            && !self.busy
            && self
                .status
                .as_ref()
                .is_some_and(|status| status.owned && status.phase == "running")
    }
}

#[derive(Clone, PartialEq)]
pub struct ServiceContext {
    pub ai: ServiceHandle,
    pub image: ServiceHandle,
    pub speech: ServiceHandle,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ServiceProviderProps {
    pub children: Children,
}

#[derive(Clone)]
struct MonitorState {
    status: Option<ServiceStatus>,
    availability: ServiceAvailability,
    error: Option<String>,
    busy: bool,
    operation: Option<ServiceOperation>,
    refresh_epoch: u64,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            status: None,
            availability: ServiceAvailability::Checking,
            error: None,
            busy: false,
            operation: None,
            refresh_epoch: 0,
        }
    }
}

enum MonitorAction {
    Refresh,
    Checking,
    NativeStatus(Result<ServiceStatus, String>),
    BrowserStatus(Result<(), probe::ProbeError>),
    BeginManagement(ServiceOperation),
    FinishManagement(Result<ServiceStatus, String>),
}

impl Reducible for MonitorState {
    type Action = MonitorAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            MonitorAction::Refresh => {
                next.availability = ServiceAvailability::Checking;
                next.error = None;
                next.refresh_epoch = next.refresh_epoch.wrapping_add(1);
            }
            MonitorAction::Checking => {
                next.availability = ServiceAvailability::Checking;
                next.error = None;
            }
            MonitorAction::NativeStatus(result) => match result {
                Ok(status) => {
                    next.availability = if status.healthy {
                        ServiceAvailability::Available
                    } else {
                        ServiceAvailability::Unavailable
                    };
                    next.status = Some(status);
                    next.error = None;
                }
                Err(message) => {
                    next.availability = ServiceAvailability::Failed(message.clone());
                    next.error = Some(message);
                }
            },
            MonitorAction::BrowserStatus(result) => {
                next.status = None;
                next.error = None;
                next.availability = match result {
                    Ok(()) => ServiceAvailability::Available,
                    Err(probe::ProbeError::Unreachable) => ServiceAvailability::Unavailable,
                    Err(probe::ProbeError::Failed(message)) => ServiceAvailability::Failed(message),
                };
            }
            MonitorAction::BeginManagement(operation) => {
                next.busy = true;
                next.operation = Some(operation);
                next.error = None;
            }
            MonitorAction::FinishManagement(result) => {
                next.busy = false;
                next.operation = None;
                match result {
                    Ok(status) => {
                        next.status = Some(status);
                        next.error = None;
                    }
                    Err(message) => next.error = Some(message),
                }
                next.refresh_epoch = next.refresh_epoch.wrapping_add(1);
            }
        }
        Rc::new(next)
    }
}

#[hook]
fn use_service_handle(
    name: &'static str,
    managed: bool,
    endpoint: String,
    config_epoch: u64,
) -> ServiceHandle {
    let state = use_reducer(MonitorState::default);
    let refresh = {
        let state = state.clone();
        Callback::from(move |(): ()| state.dispatch(MonitorAction::Refresh))
    };

    {
        let state = state.clone();
        use_effect_with(
            (managed, endpoint, config_epoch, state.refresh_epoch),
            move |(managed, endpoint, _, _)| {
                let active = Rc::new(Cell::new(true));
                let in_flight = Rc::new(Cell::new(false));
                let pending = Rc::new(std::cell::RefCell::new(None::<AbortHandle>));
                let ticks = Rc::new(Cell::new(0_u32));
                let next_poll_tick = Rc::new(Cell::new(0_u32));
                let failures = Rc::new(Cell::new(0_u32));
                let managed = *managed;
                let endpoint = endpoint.clone();
                state.dispatch(MonitorAction::Checking);
                let poll = {
                    let active = active.clone();
                    let in_flight = in_flight.clone();
                    let pending = pending.clone();
                    let ticks = ticks.clone();
                    let next_poll_tick = next_poll_tick.clone();
                    let failures = failures.clone();
                    move || {
                        if !active.get()
                            || ticks.get() < next_poll_tick.get()
                            || in_flight.replace(true)
                        {
                            return;
                        }
                        let active = active.clone();
                        let in_flight = in_flight.clone();
                        let pending = pending.clone();
                        let ticks = ticks.clone();
                        let next_poll_tick = next_poll_tick.clone();
                        let failures = failures.clone();
                        let endpoint = endpoint.clone();
                        let state = state.clone();
                        let (abort, registration) = AbortHandle::new_pair();
                        *pending.borrow_mut() = Some(abort);
                        spawn_local(async move {
                            let _ = Abortable::new(
                                async {
                                    if managed {
                                        let result =
                                            probe::with_timeout(tauri_service::status(name)).await;
                                        if active.get() {
                                            let succeeded = result.is_ok();
                                            state.dispatch(MonitorAction::NativeStatus(result));
                                            schedule_next_probe(
                                                succeeded,
                                                &failures,
                                                &next_poll_tick,
                                                &ticks,
                                            );
                                        }
                                    } else {
                                        let result = probe::check(name, &endpoint).await;
                                        if active.get() {
                                            let succeeded = result.is_ok();
                                            state.dispatch(MonitorAction::BrowserStatus(result));
                                            schedule_next_probe(
                                                succeeded,
                                                &failures,
                                                &next_poll_tick,
                                                &ticks,
                                            );
                                        }
                                    }
                                },
                                registration,
                            )
                            .await;
                            pending.borrow_mut().take();
                            in_flight.set(false);
                        });
                    }
                };
                poll();
                let interval = Interval::new(POLL_TICK_MS, move || {
                    ticks.set(ticks.get().saturating_add(1));
                    poll();
                });
                move || {
                    active.set(false);
                    if let Some(abort) = pending.borrow_mut().take() {
                        abort.abort();
                    }
                    drop(interval);
                }
            },
        );
    }

    let action = |start: bool| {
        let state = state.clone();
        Callback::from(move |(): ()| {
            if !managed || state.busy {
                return;
            }
            state.dispatch(MonitorAction::BeginManagement(if start {
                ServiceOperation::Starting
            } else {
                ServiceOperation::Stopping
            }));
            let state = state.clone();
            spawn_local(async move {
                let result = if start {
                    tauri_service::start(name).await
                } else {
                    tauri_service::stop(name).await
                };
                state.dispatch(MonitorAction::FinishManagement(result));
            });
        })
    };

    ServiceHandle {
        name,
        status: state.status.clone(),
        availability: state.availability.clone(),
        busy: state.busy,
        operation: state.operation,
        error: state.error.clone(),
        managed,
        start: action(true),
        stop: action(false),
        refresh,
    }
}

#[function_component(ServiceProvider)]
pub fn service_provider(props: &ServiceProviderProps) -> Html {
    let environment = use_app_environment();
    let config_epoch = use_state(|| 0_u64);
    {
        let config_epoch = config_epoch.clone();
        use_effect_with((), move |_| {
            let listener = web_sys::window().map(|window| {
                EventListener::new(&window, CONFIGURATION_CHANGED, move |_| {
                    config_epoch.set((*config_epoch).wrapping_add(1));
                })
            });
            move || drop(listener)
        });
    }
    let preferences = crate::preferences::PreferencesStore::new(&environment.local_storage).read();
    let ai = use_service_handle(
        "ai",
        environment.is_tauri,
        preferences.ai_url,
        *config_epoch,
    );
    let image = use_service_handle(
        "image",
        environment.is_tauri,
        image_generation::read_api_url(&environment.local_storage),
        *config_epoch,
    );
    let speech = use_service_handle(
        "speech",
        environment.is_tauri,
        "http://127.0.0.1:8799".to_owned(),
        *config_epoch,
    );
    let context = ServiceContext { ai, image, speech };

    html! {
        <ContextProvider<ServiceContext> context={context}>
            {props.children.clone()}
        </ContextProvider<ServiceContext>>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle(managed: bool, running: bool, healthy: bool, owned: bool) -> ServiceHandle {
        ServiceHandle {
            name: "ai",
            status: Some(ServiceStatus {
                service: "ai".to_owned(),
                label: "AI".to_owned(),
                phase: if running { "running" } else { "stopped" }.to_owned(),
                running,
                healthy,
                owned,
                pid: None,
                message: String::new(),
            }),
            availability: if healthy {
                ServiceAvailability::Available
            } else {
                ServiceAvailability::Unavailable
            },
            busy: false,
            operation: None,
            error: None,
            managed,
            start: Callback::from(|()| {}),
            stop: Callback::from(|()| {}),
            refresh: Callback::from(|()| {}),
        }
    }

    #[test]
    fn monitor_state_tracks_refresh_and_management_atomically() {
        let state = Rc::new(MonitorState::default());
        let state = state.reduce(MonitorAction::Refresh);
        assert_eq!(state.refresh_epoch, 1);
        assert_eq!(state.availability, ServiceAvailability::Checking);
        let state = state.reduce(MonitorAction::BeginManagement(ServiceOperation::Starting));
        assert!(state.busy);
        assert_eq!(state.operation, Some(ServiceOperation::Starting));
        let state = state.reduce(MonitorAction::FinishManagement(Err(
            "start failed".to_owned()
        )));
        assert!(!state.busy);
        assert_eq!(state.operation, None);
        assert_eq!(state.refresh_epoch, 2);
        assert_eq!(state.error.as_deref(), Some("start failed"));
        let state = state.reduce(MonitorAction::Checking);
        assert_eq!(state.availability, ServiceAvailability::Checking);
        assert_eq!(state.error, None);
    }

    #[test]
    fn monitor_state_distinguishes_probe_outcomes() {
        let state = Rc::new(MonitorState::default());
        let state = state.reduce(MonitorAction::BrowserStatus(Err(
            probe::ProbeError::Unreachable,
        )));
        assert_eq!(state.availability, ServiceAvailability::Unavailable);
        let state = state.reduce(MonitorAction::BrowserStatus(Err(
            probe::ProbeError::Failed("HTTP 503".to_owned()),
        )));
        assert_eq!(
            state.availability,
            ServiceAvailability::Failed("HTTP 503".to_owned())
        );
        let state = state.reduce(MonitorAction::BrowserStatus(Ok(())));
        assert_eq!(state.availability, ServiceAvailability::Available);
        assert_eq!(state.error, None);
    }

    #[test]
    fn browser_cannot_manage_services() {
        let service = handle(false, false, false, false);
        assert!(!service.supports_start_stop());
        assert!(!service.can_start());
        assert!(!service.can_stop());
    }

    #[test]
    fn externally_owned_service_cannot_be_started_or_stopped() {
        let service = handle(true, false, true, false);
        assert!(service.supports_start_stop());
        assert!(!service.can_start());
        assert!(!service.can_stop());
    }

    #[test]
    fn owned_service_can_be_stopped_and_stopped_service_started() {
        assert!(handle(true, false, false, false).can_start());
        assert!(handle(true, true, true, true).can_stop());
        let mut pending = handle(true, true, true, true);
        pending.busy = true;
        assert!(!pending.can_start());
        assert!(!pending.can_stop());
    }

    #[test]
    fn availability_is_independent_from_process_ownership() {
        let external = handle(true, true, true, false);
        assert_eq!(external.availability, ServiceAvailability::Available);
        assert!(!external.can_stop());
        assert_eq!(ServiceAvailability::Checking.label(), "Checking");
    }
}

#[cfg(test)]
mod retry_tests {
    use super::next_probe_ticks;

    #[test]
    fn probes_back_off_linearly_and_reset_on_success() {
        assert_eq!(next_probe_ticks(false, 0), (1, 1));
        assert_eq!(next_probe_ticks(false, 1), (2, 2));
        assert_eq!(next_probe_ticks(false, 4), (5, 5));
        assert_eq!(next_probe_ticks(false, 8), (9, 6));
        assert_eq!(next_probe_ticks(true, 8), (0, 6));
        assert_eq!(next_probe_ticks(false, 0), (1, 1));
    }
}
