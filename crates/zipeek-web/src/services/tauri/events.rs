use js_sys::{Function, Promise, Reflect};
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;

use super::{TauriError, TauriService, error::message};

pub(crate) struct TauriEvents {
    object: JsValue,
    emit: Function,
    listen: Function,
}

pub(crate) struct EventSubscription {
    unlisten: Function,
    listener: Closure<dyn FnMut(JsValue)>,
}

impl EventSubscription {
    pub(crate) fn unsubscribe(self) {
        drop(self);
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        let _ = self.unlisten.call0(&JsValue::UNDEFINED);
    }
}

impl TauriEvents {
    pub(crate) fn new(service: &TauriService) -> Result<Self, TauriError> {
        let object = Reflect::get(service.runtime(), &JsValue::from_str("event"))
            .map_err(|error| TauriError::JavaScript(message(error)))?;
        if !object.is_object() || object.is_null() {
            return Err(TauriError::ApiUnavailable("event"));
        }
        let emit = Self::function(&object, "emit")?;
        let listen = Self::function(&object, "listen")?;
        Ok(Self {
            object,
            emit,
            listen,
        })
    }

    fn function(object: &JsValue, name: &'static str) -> Result<Function, TauriError> {
        Reflect::get(object, &JsValue::from_str(name))
            .map_err(|error| TauriError::JavaScript(message(error)))?
            .dyn_into::<Function>()
            .map_err(|_| TauriError::ApiUnavailable(name))
    }

    pub(crate) async fn emit<A: Serialize + ?Sized>(
        &self,
        name: &str,
        payload: &A,
    ) -> Result<(), TauriError> {
        let payload = serde_wasm_bindgen::to_value(payload)
            .map_err(|error| TauriError::Encode(error.to_string()))?;
        let pending = self
            .emit
            .call2(&self.object, &JsValue::from_str(name), &payload)
            .map_err(|error| TauriError::JavaScript(message(error)))?
            .dyn_into::<Promise>()
            .map_err(|_| TauriError::InvalidResponse("event.emit"))?;
        JsFuture::from(pending)
            .await
            .map_err(|error| TauriError::JavaScript(message(error)))?;
        Ok(())
    }

    pub(crate) async fn listen<F>(
        &self,
        name: &str,
        handler: F,
    ) -> Result<EventSubscription, TauriError>
    where
        F: FnMut(JsValue) + 'static,
    {
        let listener = Closure::wrap(Box::new(handler) as Box<dyn FnMut(JsValue)>);
        let pending = self
            .listen
            .call2(&self.object, &JsValue::from_str(name), listener.as_ref())
            .map_err(|error| TauriError::JavaScript(message(error)))?
            .dyn_into::<Promise>()
            .map_err(|_| TauriError::InvalidResponse("event.listen"))?;
        let unlisten = JsFuture::from(pending)
            .await
            .map_err(|error| TauriError::JavaScript(message(error)))?
            .dyn_into::<Function>()
            .map_err(|_| TauriError::InvalidResponse("event.listen"))?;
        Ok(EventSubscription { unlisten, listener })
    }
}
