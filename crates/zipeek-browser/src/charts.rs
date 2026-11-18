use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use web_sys::{HtmlCanvasElement, Url};

#[wasm_bindgen(inline_js = r#"
    export async function loadChartModule(url) {
        return import(url);
    }
"#)]
unsafe extern "C" {
    #[wasm_bindgen(catch, js_name = loadChartModule)]
    async fn load_chart_module(url: &str) -> Result<JsValue, JsValue>;
}

pub struct ChartInstance(JsValue);

impl ChartInstance {
    pub async fn create(canvas: &HtmlCanvasElement, configuration: &JsValue) -> Result<Self, JsValue> {
        let module_url = chart_module_url()?;
        let module = unsafe { load_chart_module(&module_url).await? };
        let create_chart = Reflect::get(&module, &JsValue::from_str("createChart"))?
            .dyn_into::<Function>()?;
        let chart = create_chart.call2(&JsValue::UNDEFINED, canvas.as_ref(), configuration)?;
        if chart.is_null() || chart.is_undefined() {
            return Err(JsValue::from_str("Chart.js did not return a chart instance"));
        }
        Ok(Self(chart))
    }

    pub fn destroy(&mut self) -> Result<(), JsValue> {
        if self.0.is_null() {
            return Ok(());
        }
        let destroy = Reflect::get(&self.0, &JsValue::from_str("destroy"))?
            .dyn_into::<Function>()?;
        destroy.call0(&self.0)?;
        self.0 = JsValue::NULL;
        Ok(())
    }
}

impl Drop for ChartInstance {
    fn drop(&mut self) {
        let _ = self.destroy();
    }
}

fn chart_module_url() -> Result<String, JsValue> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| JsValue::from_str("Chart.js requires a browser document"))?;
    let base_uri = Reflect::get(document.as_ref(), &JsValue::from_str("baseURI"))?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Document baseURI is unavailable"))?;
    Ok(Url::new_with_base("vendor/summary/summary.mjs", &base_uri)?.href())
}
