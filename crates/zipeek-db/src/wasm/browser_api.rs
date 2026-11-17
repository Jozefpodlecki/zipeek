use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

pub(super) fn property(object: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    Reflect::get(object, &JsValue::from_str(key))
}

pub(super) fn method(object: &JsValue, name: &str) -> Result<Function, JsValue> {
    property(object, name)?
        .dyn_into::<Function>()
        .map_err(|_| JsValue::from_str(&format!("Browser method unavailable: {name}")))
}

pub(super) fn call(object: &JsValue, name: &str, args: &Array) -> Result<JsValue, JsValue> {
    method(object, name)?.apply(object, args)
}

pub(super) fn constructor(name: &str, args: &Array) -> Result<Object, JsValue> {
    let target = property(&js_sys::global(), name)?
        .dyn_into::<Function>()
        .map_err(|_| JsValue::from_str(&format!("Browser constructor unavailable: {name}")))?;
    Reflect::construct(&target, args)?
        .dyn_into::<Object>()
        .map_err(|_| {
            JsValue::from_str(&format!(
                "Browser constructor did not return an object: {name}"
            ))
        })
}

pub(super) fn args(values: &[JsValue]) -> Array {
    let result = Array::new();
    for value in values {
        result.push(value);
    }
    result
}
