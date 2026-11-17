use std::{cell::RefCell, rc::Rc};

use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::{MessageEvent, Url, Worker, WorkerOptions, WorkerType};
use zipeek_pedagogy::{
    HealthMonitor, HealthPhase, HealthReply, HealthSnapshot, MonitorError, WireHealth,
};

const HEALTH_TIMEOUT_MS: i32 = 10_000;

fn field(object: &JsValue, name: &str) -> Result<JsValue, MonitorError> {
    let value =
        Reflect::get(object, &JsValue::from_str(name)).map_err(|_| MonitorError::InvalidHealth)?;
    if value.is_undefined() {
        return Err(MonitorError::InvalidHealth);
    }
    Ok(value)
}

fn parse_health(value: &JsValue) -> Result<WireHealth, MonitorError> {
    let phase = field(value, "phase")?
        .as_string()
        .ok_or(MonitorError::InvalidHealth)?;
    let pending_jobs = field(value, "pending_jobs")?
        .as_f64()
        .ok_or(MonitorError::InvalidHealth)?;
    let failed_jobs = field(value, "failed_jobs")?
        .as_f64()
        .ok_or(MonitorError::InvalidHealth)?;
    let time = field(value, "last_success_utc_ms")?;
    let last_success_utc_ms = if time.is_null() {
        None
    } else {
        Some(time.as_f64().ok_or(MonitorError::InvalidHealth)?)
    };
    let error = field(value, "last_error")?;
    let last_error = if error.is_null() {
        None
    } else {
        Some(error.as_string().ok_or(MonitorError::InvalidHealth)?)
    };
    Ok(WireHealth {
        phase,
        pending_jobs,
        failed_jobs,
        last_success_utc_ms,
        last_error,
    })
}

fn parse_reply(data: &JsValue) -> Option<HealthReply> {
    let correlation_id = field(data, "correlation_id").ok()?.as_string()?;
    let version = field(data, "version")
        .ok()
        .and_then(|value| value.as_f64())
        .filter(|value| value.is_finite() && value.fract() == 0.0)
        .and_then(|value| u16::try_from(value as i64).ok())
        .unwrap_or(0);
    let health = (|| {
        if let Ok(error) = field(data, "error") {
            if let Some(error) = error.as_string() {
                return Err(MonitorError::Remote(error));
            }
        }
        let result = field(data, "result")?;
        if let Ok(error) = field(&result, "error") {
            if let Some(error) = error.as_string() {
                return Err(MonitorError::Remote(error));
            }
        }
        parse_health(&field(&result, "health")?)
    })();
    Some(HealthReply {
        version,
        correlation_id,
        health,
    })
}

fn reject_with(reject: &Function, error: &MonitorError) {
    let _ = reject.call1(&JsValue::UNDEFINED, &JsValue::from_str(error.description()));
}

fn resolve_with(resolve: &Function, health: &HealthSnapshot) {
    let phase = match health.phase {
        HealthPhase::Disabled => "disabled",
        HealthPhase::Idle => "idle",
        HealthPhase::Running => "running",
        HealthPhase::Failed => "failed",
    };
    let summary = format!(
        "Worker available · state: {phase} · pending: {} · failed: {}",
        health.pending_jobs, health.failed_jobs
    );
    let _ = resolve.call1(&JsValue::UNDEFINED, &JsValue::from_str(&summary));
}

fn worker_url() -> Result<String, JsValue> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| JsValue::from_str("document unavailable"))?;
    let base = Reflect::get(document.as_ref(), &JsValue::from_str("baseURI"))?
        .as_string()
        .ok_or_else(|| JsValue::from_str("document base URI unavailable"))?;
    let directory = Url::new_with_base("pedagogy-worker/", &base)?;
    let url = Url::new_with_base("worker.mjs", &directory.href())?;
    Ok(url.href())
}

pub async fn inspect_worker() -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
    let options = WorkerOptions::new();
    options.set_type(WorkerType::Module);
    let worker = Worker::new_with_options(&worker_url()?, &options)?;
    let monitor = Rc::new(RefCell::new(
        HealthMonitor::new(HEALTH_TIMEOUT_MS as u64)
            .map_err(|error| JsValue::from_str(error.description()))?,
    ));
    let request = monitor
        .borrow_mut()
        .get_health(0)
        .map_err(|error| JsValue::from_str(error.description()))?;

    let resolve_cell = Rc::new(RefCell::new(None::<Function>));
    let reject_cell = Rc::new(RefCell::new(None::<Function>));
    let promise = Promise::new(&mut {
        let resolve_cell = Rc::clone(&resolve_cell);
        let reject_cell = Rc::clone(&reject_cell);
        move |resolve, reject| {
            *resolve_cell.borrow_mut() = Some(resolve);
            *reject_cell.borrow_mut() = Some(reject);
        }
    });

    let on_message = {
        let monitor = Rc::clone(&monitor);
        let resolve = Rc::clone(&resolve_cell);
        let reject = Rc::clone(&reject_cell);
        Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            let Some(reply) = parse_reply(&event.data()) else {
                return;
            };
            let resolution = monitor.borrow_mut().receive(reply);
            if let Some(resolution) = resolution {
                match resolution.result {
                    Ok(health) => {
                        if let Some(resolve) = resolve.borrow().as_ref() {
                            resolve_with(resolve, &health);
                        }
                    }
                    Err(error) => {
                        if let Some(reject) = reject.borrow().as_ref() {
                            reject_with(reject, &error);
                        }
                    }
                }
            }
        })
    };
    worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

    let on_error = {
        let monitor = Rc::clone(&monitor);
        let reject = Rc::clone(&reject_cell);
        Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            if !monitor
                .borrow_mut()
                .shutdown(MonitorError::WorkerFailed)
                .is_empty()
            {
                if let Some(reject) = reject.borrow().as_ref() {
                    reject_with(reject, &MonitorError::WorkerFailed);
                }
            }
        })
    };
    worker.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    let on_message_error = {
        let monitor = Rc::clone(&monitor);
        let reject = Rc::clone(&reject_cell);
        Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            if !monitor
                .borrow_mut()
                .shutdown(MonitorError::UnreadableMessage)
                .is_empty()
            {
                if let Some(reject) = reject.borrow().as_ref() {
                    reject_with(reject, &MonitorError::UnreadableMessage);
                }
            }
        })
    };
    worker.set_onmessageerror(Some(on_message_error.as_ref().unchecked_ref()));

    let on_timeout = {
        let monitor = Rc::clone(&monitor);
        let reject = Rc::clone(&reject_cell);
        Closure::<dyn FnMut()>::new(move || {
            if !monitor.borrow_mut().expire(u64::MAX).is_empty() {
                if let Some(reject) = reject.borrow().as_ref() {
                    reject_with(reject, &MonitorError::TimedOut);
                }
            }
        })
    };
    let timer = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        on_timeout.as_ref().unchecked_ref(),
        HEALTH_TIMEOUT_MS,
    );
    let outcome = if let Ok(timer) = timer {
        let message = Object::new();
        let command = Object::new();
        let _ = Reflect::set(
            &command,
            &JsValue::from_str("kind"),
            &JsValue::from_str("get_health"),
        );
        let _ = Reflect::set(
            &message,
            &JsValue::from_str("version"),
            &JsValue::from_f64(f64::from(request.version)),
        );
        let _ = Reflect::set(
            &message,
            &JsValue::from_str("correlation_id"),
            &JsValue::from_str(&request.correlation_id),
        );
        let _ = Reflect::set(&message, &JsValue::from_str("command"), &command);
        if worker.post_message(&message).is_err() {
            if let Some(reject) = reject_cell.borrow().as_ref() {
                reject_with(reject, &MonitorError::SendFailed);
            }
        }
        let outcome = JsFuture::from(promise).await;
        window.clear_timeout_with_handle(timer);
        outcome
    } else {
        Err(JsValue::from_str(
            "unable to schedule worker health timeout",
        ))
    };

    worker.set_onmessage(None);
    worker.set_onerror(None);
    worker.set_onmessageerror(None);
    worker.terminate();
    monitor.borrow_mut().shutdown(MonitorError::Closed);
    outcome.and_then(|value| {
        value
            .as_string()
            .ok_or_else(|| JsValue::from_str("invalid worker health summary"))
    })
}
