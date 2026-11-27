use std::{cell::Cell, rc::Rc};

use futures::future::{AbortHandle, Abortable};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    contexts::service_management::{ServiceAvailability, with_timeout},
    hooks::use_image_service,
    services::{image_generation, tauri_service::ServiceStatus},
};

#[derive(Clone, PartialEq)]
struct ProbeKey {
    needed: bool,
    endpoint: String,
    availability: ServiceAvailability,
    status: Option<ServiceStatus>,
    busy: bool,
    revision: u64,
}

#[hook]
pub(super) fn use_image_readiness(
    needed: bool,
    endpoint: String,
) -> (ServiceAvailability, Callback<MouseEvent>) {
    let service = use_image_service();
    let revision = use_state(|| 0_u64);
    let snapshot = use_state(|| None::<(ProbeKey, ServiceAvailability)>);
    let key = ProbeKey {
        needed,
        endpoint,
        availability: service.availability,
        status: service.status,
        busy: service.busy,
        revision: *revision,
    };
    {
        let snapshot = snapshot.clone();
        use_effect_with(key.clone(), move |key| {
            let active = Rc::new(Cell::new(true));
            let (abort, registration) = AbortHandle::new_pair();
            if key.needed {
                let key = key.clone();
                let active = active.clone();
                spawn_local(async move {
                    let _ = Abortable::new(
                        async {
                            let result =
                                with_timeout(image_generation::check_health(&key.endpoint)).await;
                            if active.get() {
                                let availability = match result {
                                    Ok(_) => ServiceAvailability::Available,
                                    Err(error) => ServiceAvailability::Failed(error),
                                };
                                snapshot.set(Some((key, availability)));
                            }
                        },
                        registration,
                    )
                    .await;
                });
            } else {
                snapshot.set(None);
            }
            move || {
                active.set(false);
                abort.abort();
            }
        });
    }
    let availability = if !needed {
        ServiceAvailability::Available
    } else {
        snapshot
            .as_ref()
            .filter(|(checked, _)| checked == &key)
            .map(|(_, availability)| availability.clone())
            .unwrap_or(ServiceAvailability::Checking)
    };
    let on_retry = Callback::from(move |_: MouseEvent| revision.set(revision.wrapping_add(1)));
    (availability, on_retry)
}
