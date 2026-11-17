use crate::contexts::service_management::ServiceHandle;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct NativeServicePanelProps {
    pub service: ServiceHandle,
}

#[function_component(NativeServicePanel)]
pub fn native_service_panel(props: &NativeServicePanelProps) -> Html {
    let service = &props.service;
    let on_start = {
        let action = service.start.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };
    let on_stop = {
        let action = service.stop.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };
    let service_label = if service.name == "ai" {
        "AI service"
    } else {
        "Image service"
    };
    let detail = if service.name == "ai" {
        html! {
            <p class="mt-2 text-xs text-muted">
                {"The bundled AI executable currently provides fixture mode only, not production inference."}
            </p>
        }
    } else {
        Html::default()
    };
    let error = if let Some(message) = &service.error {
        html! { <p class="mt-2 text-sm text-red-500" role="alert">{message}</p> }
    } else {
        Html::default()
    };
    let phase_label = service.availability.label();
    let status_message = service.availability.details().unwrap_or_else(|| {
        if service.managed {
            service
                .status
                .as_ref()
                .map(|status| status.message.as_str())
                .unwrap_or("Checking local service")
        } else {
            "Availability only: model readiness is not verified"
        }
    });
    let on_refresh = {
        let action = service.refresh.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };

    html! {
        <section class="mt-5 rounded-xl border border-line bg-surface p-4" data-native-service={service.name}>
            <div class="flex flex-wrap items-center justify-between gap-3">
                <div>
                    <div class="flex flex-wrap items-center gap-2">
                        <h3 class="text-sm font-semibold">{service_label}</h3>
                        <span class="rounded-md border border-line px-2 py-0.5 text-xs text-muted">{phase_label}</span>
                    </div>
                    <p class="mt-1 text-xs text-muted" role="status">{status_message}</p>
                </div>
                <div class="flex gap-2">
                    <button type="button" onclick={on_refresh} class="rounded-lg border border-line px-3 py-2 text-sm">{"Refresh"}</button>
                    if service.managed {<>
                    <button type="button" class="rounded-lg border border-line px-4 py-2 text-sm disabled:opacity-50"
                        disabled={!service.can_start()} onclick={on_start}>{"Start"}</button>
                    <button type="button" class="rounded-lg border border-line px-4 py-2 text-sm disabled:opacity-50"
                        disabled={!service.can_stop()} onclick={on_stop}>{"Stop"}</button>
                    </>}
                </div>
            </div>
            {detail}
            {error}
        </section>
    }
}
