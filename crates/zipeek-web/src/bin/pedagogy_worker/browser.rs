use std::{cell::RefCell, rc::Rc};

use js_sys::Array;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

use super::{
    error::{WorkerErrorCode, WorkerRuntimeError},
    runtime::WorkerRuntime,
    wire::ResponseDto,
};

fn respond(scope: &DedicatedWorkerGlobalScope, runtime: &mut WorkerRuntime, data: JsValue) {
    let correlation_id = js_sys::Reflect::get(&data, &JsValue::from_str("correlation_id"))
        .ok()
        .and_then(|value| value.as_string());
    let reply = match serde_wasm_bindgen::from_value::<serde_json::Value>(data) {
        Ok(request) => runtime.handle_json(&request.to_string()),
        Err(error) => ResponseDto::failure(
            correlation_id,
            WorkerRuntimeError::new(
                WorkerErrorCode::InvalidRequest,
                format!("invalid worker request: {error}"),
            ),
        ),
    };
    match serde_wasm_bindgen::to_value(&reply) {
        Ok(value) => {
            if let Err(error) = scope.post_message(&value) {
                web_sys::console::error_1(&error);
            }
        }
        Err(error) => web_sys::console::error_1(&JsValue::from_str(&format!(
            "could not encode worker response: {error}"
        ))),
    }
}

#[wasm_bindgen]
pub fn start_worker(pending: Array) -> Result<(), JsValue> {
    let scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into()?;
    let runtime = Rc::new(RefCell::new(WorkerRuntime::default()));
    let event_scope = scope.clone();
    let event_runtime = Rc::clone(&runtime);
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        respond(&event_scope, &mut event_runtime.borrow_mut(), event.data());
    });
    scope.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
    for message in pending.iter() {
        respond(&scope, &mut runtime.borrow_mut(), message);
    }
    Ok(())
}
