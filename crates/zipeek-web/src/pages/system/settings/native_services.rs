use crate::contexts::service_management::{ServiceAvailability, ServiceHandle, ServiceOperation};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

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
    let on_refresh = {
        let action = service.refresh.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };
    let (label, description, icon) = match service.name {
        "ai" => ("Local AI", "Llama.cpp GGUF inference", IconData::LUCIDE_CPU),
        "image" => (
            "Image generation",
            "Local image backend",
            IconData::LUCIDE_IMAGE,
        ),
        "speech" => ("Speech", "TTS, STT and Live STT", IconData::LUCIDE_MIC),
        _ => (
            "Local service",
            "Managed local process",
            IconData::LUCIDE_SERVER,
        ),
    };
    let checking = matches!(&service.availability, ServiceAvailability::Checking);
    let starting = service.operation == Some(ServiceOperation::Starting);
    let stopping = service.operation == Some(ServiceOperation::Stopping);
    let phase = service.status.as_ref().map(|status| status.phase.as_str());
    let running =
        phase == Some("running") && service.status.as_ref().is_some_and(|status| status.healthy);
    let external = phase == Some("external");
    let failed =
        matches!(&service.availability, ServiceAvailability::Failed(_)) || phase == Some("failed");
    let (phase_label, phase_color, phase_icon) = if starting || phase == Some("starting") {
        ("Starting", "text-amber-500", IconData::LUCIDE_REFRESH_CW)
    } else if stopping || phase == Some("stopping") {
        ("Stopping", "text-amber-500", IconData::LUCIDE_REFRESH_CW)
    } else if running {
        ("Healthy", "text-emerald-500", IconData::LUCIDE_CHECK)
    } else if external {
        (
            "External",
            "text-amber-500",
            IconData::LUCIDE_ALERT_TRIANGLE,
        )
    } else if failed {
        (
            "Issue detected",
            "text-red-500",
            IconData::LUCIDE_ALERT_TRIANGLE,
        )
    } else {
        ("Stopped", "text-muted", IconData::LUCIDE_SQUARE)
    };
    let message = if checking {
        "Checking health..."
    } else if let Some(details) = service.availability.details() {
        details
    } else {
        match service.status.as_ref() {
            Some(status)
                if (running && status.message == "Running and healthy")
                    || (phase == Some("stopped") && status.message == "Stopped") =>
            {
                ""
            }
            Some(status) => status.message.as_str(),
            None => "Waiting for service status",
        }
    };
    let error = service.error.as_deref();
    let button_classes = "inline-flex min-h-10 items-center justify-center gap-2 rounded-lg border border-line bg-raised px-3 py-2 text-sm font-medium transition-colors hover:border-accent/50 hover:bg-accent-soft focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40";

    html! {
        <section class="rounded-2xl border border-line bg-surface p-4 sm:p-5" data-native-service={service.name} aria-busy={service.busy.to_string()}>
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div class="flex min-w-0 items-start gap-3">
                    <span class="grid size-11 shrink-0 place-items-center rounded-xl border border-line bg-raised text-accent" aria-hidden="true">
                        <Icon data={icon} width="21px" height="21px" />
                    </span>
                    <div class="min-w-0 space-y-1">
                        <h3 class="font-semibold text-ink">{label}</h3>
                        <p class="text-xs text-muted">{description}</p>
                        <div class="flex flex-wrap items-center gap-2 pt-1" role="status" aria-live="polite">
                            <span class={classes!("inline-flex", "items-center", "gap-1.5", "text-xs", "font-medium", phase_color)}>
                                <Icon data={phase_icon} width="14px" height="14px" class={if starting || stopping || phase == Some("starting") || phase == Some("stopping") {"animate-spin motion-reduce:animate-none"} else {""}} />
                                {phase_label}
                            </span>
                            if !message.is_empty() {
                                <span class="text-xs text-muted">{"·"}</span>
                                <span class="text-xs text-muted">{message}</span>
                            }
                        </div>
                    </div>
                </div>
                <div class="flex shrink-0 flex-wrap items-center gap-2 sm:justify-end">
                    <button type="button" onclick={on_refresh} disabled={checking || service.busy}
                        data-service-health={service.name} aria-label={format!("Check {label} health")}
                        class={button_classes}>
                        <Icon data={IconData::LUCIDE_REFRESH_CW} width="16px" height="16px" class={if checking {"animate-spin motion-reduce:animate-none"} else {""}} />
                        {if checking {"Checking..."} else {"Check health"}}
                    </button>
                    if service.managed {
                        <button type="button" onclick={on_start} disabled={!service.can_start()}
                            data-service-start={service.name} class={button_classes}>
                            <Icon data={if starting {IconData::LUCIDE_REFRESH_CW} else {IconData::LUCIDE_PLAY}} width="16px" height="16px" class={if starting {"animate-spin motion-reduce:animate-none"} else {""}} />
                            {if starting {"Starting..."} else {"Start"}}
                        </button>
                        <button type="button" onclick={on_stop} disabled={!service.can_stop()}
                            data-service-stop={service.name} class={button_classes}>
                            <Icon data={if stopping {IconData::LUCIDE_REFRESH_CW} else {IconData::LUCIDE_SQUARE}} width="16px" height="16px" class={if stopping {"animate-spin motion-reduce:animate-none"} else {""}} />
                            {if stopping {"Stopping..."} else {"Stop"}}
                        </button>
                    }
                </div>
            </div>
            if let Some(message) = error {
                <p class="mt-3 rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-500" role="alert">{message}</p>
            }
        </section>
    }
}
