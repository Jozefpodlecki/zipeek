use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use js_sys::Reflect;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::services::tauri::{
    TauriEvents, TauriService,
    commands::{ShutdownForceExit, ShutdownPhase, ShutdownStatus},
};

impl Reducible for ShutdownPhase {
    type Action = Self;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        Rc::new((*self).max(action))
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    pub children: Children,
}

#[function_component(DesktopLifecycle)]
pub(crate) fn desktop_lifecycle(props: &Props) -> Html {
    let phase = use_reducer(ShutdownPhase::default);
    let panel = use_node_ref();

    {
        let phase = phase.clone();
        use_effect_with((), move |_| {
            let cancelled = Rc::new(Cell::new(false));
            let subscription = Rc::new(RefCell::new(None));
            let task_cancelled = cancelled.clone();
            let task_subscription = subscription.clone();
            spawn_local(async move {
                let Ok(service) = TauriService::shared() else {
                    return;
                };
                let result = async {
                    let events = TauriEvents::new(&service)?;
                    let event_phase = phase.clone();
                    let event_cancelled = task_cancelled.clone();
                    let listener = events
                        .listen("zipeek-shutdown", move |event| {
                            if event_cancelled.get() {
                                return;
                            }
                            let payload = Reflect::get(&event, &JsValue::from_str("payload"));
                            match payload.map_err(|error| format!("{error:?}")).and_then(
                                |payload| {
                                    serde_wasm_bindgen::from_value::<ShutdownPhase>(payload)
                                        .map_err(|error| error.to_string())
                                },
                            ) {
                                Ok(next) => event_phase.dispatch(next),
                                Err(error) => log::error!("Invalid shutdown event: {error}"),
                            }
                        })
                        .await?;
                    if task_cancelled.get() {
                        return Ok::<(), crate::services::tauri::TauriError>(());
                    }
                    *task_subscription.borrow_mut() = Some(listener);
                    let current = service.execute::<ShutdownStatus>(&()).await?;
                    if !task_cancelled.get() {
                        phase.dispatch(current);
                    }
                    Ok(())
                }
                .await;
                if let Err(error) = result {
                    log::error!("Unable to observe desktop shutdown: {error}");
                }
            });
            move || {
                cancelled.set(true);
                subscription.borrow_mut().take();
            }
        });
    }

    {
        let panel = panel.clone();
        use_effect_with(*phase, move |phase| {
            if *phase != ShutdownPhase::Idle
                && let Some(element) = panel.cast::<HtmlElement>()
            {
                let _ = element.focus();
            }
        });
    }

    let on_close_now = Callback::from(move |_| {
        spawn_local(async {
            let result = async {
                TauriService::shared()?
                    .execute::<ShutdownForceExit>(&())
                    .await
            }
            .await;
            if let Err(error) = result {
                log::error!("Unable to close Zipeek: {error}");
            }
        });
    });

    if *phase == ShutdownPhase::Idle {
        return html! { <>{for props.children.iter()}</> };
    }

    let failed = *phase == ShutdownPhase::Failed;
    html! {
        <div class="fixed inset-0 z-[200] grid min-h-dvh place-items-center bg-canvas p-6 text-ink" data-testid="desktop-shutdown">
            <div ref={panel} tabindex="-1" class="loading-panel focus:outline-none" role="status" aria-live="polite" aria-atomic="true">
                if !failed {
                    <span class="loading-spinner" aria-hidden="true"></span>
                }
                <p>{if failed { "Unable to finish closing" } else { "Closing Zipeek..." }}</p>
                <p class="loading-detail">
                    {if failed { "Some local services may still be running. You can close Zipeek now." } else { "Stopping local services" }}
                </p>
                if failed {
                    <button type="button" class="rounded-lg bg-brand-action px-4 py-2 text-white focus-visible:outline-2 focus-visible:outline-offset-2" onclick={on_close_now}>
                        {"Close now"}
                    </button>
                }
            </div>
        </div>
    }
}
