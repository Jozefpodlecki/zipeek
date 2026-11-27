use js_sys::{Array, ArrayBuffer, Function, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub(super) fn property(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    Reflect::get(target, &JsValue::from_str(name))
}

pub(super) fn assign(target: &JsValue, name: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(name), value).map(|_| ())
}

pub(super) fn invoke(
    target: &JsValue,
    method: &str,
    values: &[JsValue],
) -> Result<JsValue, JsValue> {
    let function = property(target, method)?
        .dyn_into::<Function>()
        .map_err(|_| JsValue::from_str(&format!("Missing browser method: {method}")))?;
    let arguments = Array::new();
    for value in values {
        arguments.push(value);
    }
    function.apply(target, &arguments)
}

pub(super) fn new_object(name: &str, values: &[JsValue]) -> Result<JsValue, JsValue> {
    let constructor = property(&js_sys::global(), name)?
        .dyn_into::<Function>()
        .map_err(|_| JsValue::from_str(&format!("Missing browser constructor: {name}")))?;
    let arguments = Array::new();
    for value in values {
        arguments.push(value);
    }
    Reflect::construct(&constructor, &arguments).map(JsValue::from)
}

pub(super) async fn wait(value: JsValue) -> Result<JsValue, JsValue> {
    JsFuture::from(Promise::resolve(&value)).await
}

pub(super) async fn fetch(
    path: &str,
    method: &str,
    kind: Option<&str>,
    data: Option<&JsValue>,
    controller: Option<&JsValue>,
) -> Result<JsValue, JsValue> {
    if tauri_available() {
        return native_speech_response(path, method, data).await;
    }
    let options = Object::new();
    assign(&options, "method", &JsValue::from_str(method))?;
    assign(&options, "mode", &JsValue::from_str("cors"))?;
    assign(&options, "cache", &JsValue::from_str("no-store"))?;
    assign(&options, "credentials", &JsValue::from_str("omit"))?;
    if let Some(data) = data {
        assign(&options, "body", data)?;
    }
    if let Some(kind) = kind {
        let headers = Object::new();
        assign(&headers, "Content-Type", &JsValue::from_str(kind))?;
        assign(&options, "headers", &headers)?;
    }
    if let Some(controller) = controller {
        assign(&options, "signal", &property(controller, "signal")?)?;
    }
    let response = wait(invoke(
        &js_sys::global(),
        "fetch",
        &[
            JsValue::from_str(&format!("http://127.0.0.1:8799{path}")),
            options.into(),
        ],
    )?)
    .await?;
    if !property(&response, "ok")?.as_bool().unwrap_or(false) {
        let status = property(&response, "status")?.as_f64().unwrap_or(0.0) as u16;
        let detail = wait(invoke(&response, "text", &[])?)
            .await?
            .as_string()
            .unwrap_or_default();
        return Err(JsValue::from_str(&format!(
            "Local speech service {status}: {}",
            detail.chars().take(300).collect::<String>()
        )));
    }
    Ok(response)
}

pub(super) async fn decode_audio(data: ArrayBuffer) -> Result<Vec<u8>, JsValue> {
    let context = new_object("AudioContext", &[])?;
    let result = async {
        let buffer = wait(invoke(&context, "decodeAudioData", &[data.into()])?).await?;
        let sample_rate = property(&buffer, "sampleRate")?.as_f64().unwrap_or(0.0) as u32;
        let duration = property(&buffer, "duration")?
            .as_f64()
            .unwrap_or(f64::INFINITY);
        let length = property(&buffer, "length")?.as_f64().unwrap_or(0.0) as usize;
        let channels = property(&buffer, "numberOfChannels")?
            .as_f64()
            .unwrap_or(0.0) as usize;
        if duration <= 0.0
            || duration > 30.0
            || !(1..=2).contains(&channels)
            || !(8000..=192000).contains(&sample_rate)
            || length == 0
            || length > sample_rate as usize * 30
        {
            return Err(JsValue::from_str(
                "Recording has invalid duration or audio channels",
            ));
        }
        let mut mono = vec![0.0_f32; length];
        for channel in 0..channels {
            let data = invoke(
                &buffer,
                "getChannelData",
                &[JsValue::from_f64(channel as f64)],
            )?;
            let samples = js_sys::Float32Array::new(&data);
            if samples.length() as usize != length {
                return Err(JsValue::from_str("Malformed decoded microphone samples"));
            }
            for (index, sample) in mono.iter_mut().enumerate() {
                *sample += samples.get_index(index as u32) / channels as f32;
            }
        }
        crate::speech_wav::encode_mono_pcm16(&mono, sample_rate).map_err(JsValue::from_str)
    }
    .await;
    if let Ok(promise) = invoke(&context, "close", &[]) {
        let _ = wait(promise).await;
    }
    result
}

pub(super) fn tauri_available() -> bool {
    property(&js_sys::global(), "__TAURI__")
        .ok()
        .is_some_and(|value| value.is_object() && !value.is_null())
}

async fn native_speech_response(
    path: &str,
    method: &str,
    data: Option<&JsValue>,
) -> Result<JsValue, JsValue> {
    let operation = match (method, path) {
        ("GET", "/v1/capabilities") => "capabilities",
        ("POST", "/v1/tts") => "tts",
        ("POST", "/v1/stt") => "stt",
        _ => return Err(JsValue::from_str("Unsupported native speech operation")),
    };
    let bytes = match (operation, data) {
        ("capabilities", _) => Array::new(),
        ("tts", Some(value)) => {
            let text = value
                .as_string()
                .ok_or_else(|| JsValue::from_str("TTS text expected"))?;
            {
                let result = Array::new();
                for byte in text.as_bytes() {
                    result.push(&JsValue::from_f64(f64::from(*byte)));
                }
                result
            }
        }
        ("stt", Some(value)) => Array::from(&Uint8Array::new(value)),
        _ => return Err(JsValue::from_str("Speech body missing")),
    };
    let args = Object::new();
    assign(&args, "operation", &JsValue::from_str(operation))?;
    assign(&args, "bytes", &bytes.into())?;
    let tauri = property(&js_sys::global(), "__TAURI__")?;
    let core = property(&tauri, "core")?;
    let reply = wait(invoke(
        &core,
        "invoke",
        &[JsValue::from_str("speech_request"), args.into()],
    )?)
    .await?;
    let response_bytes = property(&reply, "bytes")?;
    let body = Uint8Array::new(&response_bytes);
    let headers = Object::new();
    assign(&headers, "Content-Type", &property(&reply, "content_type")?)?;
    let options = Object::new();
    assign(&options, "status", &JsValue::from_f64(200.0))?;
    assign(&options, "headers", &headers.into())?;
    new_object("Response", &[body.into(), options.into()])
}
