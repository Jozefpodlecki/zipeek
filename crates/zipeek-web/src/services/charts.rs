use serde::Serialize;
use serde_json::json;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;
use zipeek_browser::charts::ChartInstance;

pub(crate) async fn summary(
    canvas: &HtmlCanvasElement,
    correct: u32,
    total: u32,
    reduced_motion: bool,
) -> Result<ChartInstance, JsValue> {
    let correct = correct.min(total);
    create(
        canvas,
        json!({
            "type": "doughnut",
            "data": {"datasets": [{
                "data": [correct, total - correct],
                "backgroundColor": ["#2563eb", "#94a3b8"],
                "borderWidth": 0
            }]},
            "options": {
                "responsive": true,
                "maintainAspectRatio": false,
                "cutout": "72%",
                "events": [],
                "animation": if reduced_motion { json!(false) } else { json!({"duration": 300, "easing": "linear"}) },
                "plugins": {"legend": false, "tooltip": false}
            }
        }),
    )
    .await
}

pub(crate) async fn course_progress(
    canvas: &HtmlCanvasElement,
    completed: u32,
    total: u32,
    accent: &str,
    reduced_motion: bool,
) -> Result<ChartInstance, JsValue> {
    let done = completed.min(total);
    let data = if total == 0 {
        vec![0, 1]
    } else {
        vec![done, total - done]
    };
    create(
        canvas,
        json!({
            "type": "doughnut",
            "data": {"datasets": [{
                "data": data,
                "backgroundColor": [accent, "#64748b55"],
                "borderWidth": 0
            }]},
            "options": {
                "responsive": true,
                "maintainAspectRatio": false,
                "cutout": "79%",
                "events": [],
                "animation": if reduced_motion { json!(false) } else { json!({"duration": 300}) },
                "plugins": {"legend": false, "tooltip": false}
            }
        }),
    )
    .await
}

pub(crate) async fn stats_activity(
    canvas: &HtmlCanvasElement,
    counts: &[u32; 5],
    accent: &str,
) -> Result<ChartInstance, JsValue> {
    create(
        canvas,
        json!({
            "type": "bar",
            "data": {
                "labels": ["Oct 15", "Oct 22", "Oct 29", "Nov 5", "Nov 12"],
                "datasets": [{
                    "label": "Practice sessions",
                    "data": counts,
                    "backgroundColor": accent,
                    "borderRadius": 5
                }]
            },
            "options": {
                "responsive": true,
                "maintainAspectRatio": false,
                "animation": false,
                "plugins": {"legend": {"display": false}},
                "scales": {
                    "y": {"beginAtZero": true, "ticks": {"precision": 0}},
                    "x": {"grid": {"display": false}}
                }
            }
        }),
    )
    .await
}

async fn create(
    canvas: &HtmlCanvasElement,
    configuration: serde_json::Value,
) -> Result<ChartInstance, JsValue> {
    let configuration = configuration
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|error| JsValue::from_str(&format!("Invalid chart config: {error}")))?;
    ChartInstance::create(canvas, &configuration).await
}
