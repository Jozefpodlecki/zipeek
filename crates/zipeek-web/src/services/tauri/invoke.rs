use std::{cell::RefCell, rc::Rc};

use js_sys::{Function, Promise, Reflect};
use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::Window;

use super::{TauriError, commands::Command, error::message};

thread_local! {
    static SHARED: RefCell<Option<Rc<TauriService>>> = const { RefCell::new(None) };
}

pub(crate) struct TauriService {
    runtime: JsValue,
    core: JsValue,
    invoke: Function,
}

impl TauriService {
    pub(crate) fn new(window: &Window) -> Result<Self, TauriError> {
        let runtime = Reflect::get(window.as_ref(), &JsValue::from_str("__TAURI__"))
            .map_err(|error| TauriError::JavaScript(message(error)))?;
        if !runtime.is_object() || runtime.is_null() {
            return Err(TauriError::BridgeUnavailable);
        }
        let core = Reflect::get(&runtime, &JsValue::from_str("core"))
            .map_err(|error| TauriError::JavaScript(message(error)))?;
        if !core.is_object() || core.is_null() {
            return Err(TauriError::ApiUnavailable("core"));
        }
        let invoke = Reflect::get(&core, &JsValue::from_str("invoke"))
            .map_err(|error| TauriError::JavaScript(message(error)))?
            .dyn_into::<Function>()
            .map_err(|_| TauriError::ApiUnavailable("core.invoke"))?;
        Ok(Self {
            runtime,
            core,
            invoke,
        })
    }

    pub(crate) fn shared() -> Result<Rc<Self>, TauriError> {
        SHARED.with(|slot| {
            if let Some(service) = slot.borrow().as_ref() {
                return Ok(service.clone());
            }
            let window = web_sys::window().ok_or(TauriError::BridgeUnavailable)?;
            let service = Rc::new(Self::new(&window)?);
            *slot.borrow_mut() = Some(service.clone());
            Ok(service)
        })
    }

    pub(crate) fn available(window: &Window) -> bool {
        Self::new(window).is_ok()
    }

    pub(crate) fn is_tauri(&self) -> Result<bool, TauriError> {
        let check = Reflect::get(&self.core, &JsValue::from_str("isTauri"))
            .map_err(|error| TauriError::JavaScript(message(error)))?
            .dyn_into::<Function>()
            .map_err(|_| TauriError::ApiUnavailable("core.isTauri"))?;
        check
            .call0(&self.core)
            .map_err(|error| TauriError::JavaScript(message(error)))?
            .as_bool()
            .ok_or(TauriError::InvalidResponse("core.isTauri"))
    }

    pub(crate) fn runtime(&self) -> &JsValue {
        &self.runtime
    }

    pub(crate) async fn invoke_raw<A: Serialize + ?Sized>(
        &self,
        command: &'static str,
        args: &A,
    ) -> Result<JsValue, TauriError> {
        let args = serde_wasm_bindgen::to_value(args)
            .map_err(|error| TauriError::Encode(error.to_string()))?;
        let promise = self
            .invoke
            .call2(&self.core, &JsValue::from_str(command), &args)
            .map_err(|error| TauriError::Invocation {
                command,
                details: message(error),
            })?
            .dyn_into::<Promise>()
            .map_err(|_| TauriError::InvalidResponse(command))?;
        JsFuture::from(promise)
            .await
            .map_err(|error| TauriError::Invocation {
                command,
                details: message(error),
            })
    }

    pub(crate) async fn invoke<A: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        command: &'static str,
        args: &A,
    ) -> Result<T, TauriError> {
        let value = self.invoke_raw(command, args).await?;
        serde_wasm_bindgen::from_value(value).map_err(|error| TauriError::Decode(error.to_string()))
    }

    pub(crate) async fn execute<C: Command>(
        &self,
        args: &C::Args,
    ) -> Result<C::Output, TauriError> {
        self.invoke(C::NAME, args).await
    }
}
