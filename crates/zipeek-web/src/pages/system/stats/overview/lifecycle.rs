use std::{cell::Cell, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;

use crate::{AppEnvironment, appearance::Appearance, services::charts};

use super::activity::WEEKS;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ChartStatus {
    Loading,
    Ready,
    Unavailable,
}

#[hook]
pub(super) fn use_activity_chart(
    canvas: NodeRef,
    buckets: [u32; WEEKS],
) -> UseStateHandle<ChartStatus> {
    let environment = use_context::<AppEnvironment>().expect("Statistics requires AppEnvironment");
    let appearance = use_context::<Appearance>().expect("Statistics requires Appearance");
    let instance = use_mut_ref(|| None::<zipeek_browser::charts::ChartInstance>);
    let status = use_state(|| ChartStatus::Loading);
    {
        let window = environment.window.clone();
        let instance = instance.clone();
        let status = status.clone();
        use_effect_with((buckets, appearance.preset), move |(values, _)| {
            status.set(ChartStatus::Loading);
            let active = Rc::new(Cell::new(true));
            let previous = instance.borrow_mut().take();
            drop(previous);
            if let Some(element) = canvas.cast::<HtmlCanvasElement>() {
                let accent = window
                    .get_computed_style(&element)
                    .ok()
                    .flatten()
                    .and_then(|style| style.get_property_value("--accent").ok())
                    .filter(|color| !color.trim().is_empty())
                    .unwrap_or_else(|| "#8062d0".to_owned());
                let active_for_task = active.clone();
                let instance_for_task = instance.clone();
                let status_for_task = status.clone();
                let values = *values;
                spawn_local(async move {
                    let result = charts::stats_activity(&element, &values, &accent).await;
                    if !active_for_task.get() {
                        return;
                    }
                    match result {
                        Ok(chart) => {
                            *instance_for_task.borrow_mut() = Some(chart);
                            status_for_task.set(ChartStatus::Ready);
                        }
                        Err(reason) => {
                            log::warn!("Statistics activity chart failed: {reason:?}");
                            status_for_task.set(ChartStatus::Unavailable);
                        }
                    }
                });
            } else {
                status.set(ChartStatus::Unavailable);
            }
            move || {
                active.set(false);
                instance.borrow_mut().take();
            }
        });
    }
    status
}
