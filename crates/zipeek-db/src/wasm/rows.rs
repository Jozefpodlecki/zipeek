use js_sys::{Array, Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};

pub fn table_rows(table: &JsValue) -> Result<JsValue, JsValue> {
    let to_array = Reflect::get(table, &JsValue::from_str("toArray"))?.dyn_into::<Function>()?;
    let rows = to_array.call0(table)?;
    let array = Array::from(&rows);
    let converted = Array::new();
    for row in array.iter() {
        let to_json = Reflect::get(&row, &JsValue::from_str("toJSON"))?.dyn_into::<Function>()?;
        converted.push(&to_json.call0(&row)?);
    }
    Ok(converted.into())
}
