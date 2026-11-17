use js_sys::{Array, Function, Reflect};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use yew::Callback;

fn property(object: &JsValue, name: &str) -> Result<JsValue, String> {
    Reflect::get(object, &JsValue::from_str(name))
        .map_err(|_| format!("Speech recognition property {name} is unavailable"))
}

fn transcription(event: &JsValue) -> Result<String, String> {
    let results = property(event, "results")?;
    let first = Array::from(&results).get(0);
    let alternative = Array::from(&first).get(0);
    let transcript = property(&alternative, "transcript")?
        .as_string()
        .ok_or_else(|| "Speech recognition returned no text".to_owned())?;
    if transcript.trim().is_empty() {
        Err("No speech was recognized. Please try again.".to_owned())
    } else {
        Ok(transcript)
    }
}

pub fn test_mandarin(
    window: &web_sys::Window,
    completed: Callback<Result<String, String>>,
) -> Result<(), String> {
    let root: JsValue = window.clone().into();
    let constructor = property(&root, "SpeechRecognition")
        .ok()
        .filter(|value| !value.is_undefined() && !value.is_null())
        .or_else(|| {
            property(&root, "webkitSpeechRecognition")
                .ok()
                .filter(|value| !value.is_undefined() && !value.is_null())
        })
        .ok_or_else(|| "This browser does not provide Speech Recognition".to_owned())?;
    let constructor: Function = constructor
        .dyn_into()
        .map_err(|_| "Speech recognition constructor is unavailable".to_owned())?;
    let recognition = Reflect::construct(&constructor, &Array::new())
        .map_err(|_| "Could not create speech recognizer".to_owned())?;
    Reflect::set(
        &recognition,
        &JsValue::from_str("lang"),
        &JsValue::from_str("zh-CN"),
    )
    .map_err(|_| "Could not configure recognition language".to_owned())?;
    let _ = Reflect::set(
        &recognition,
        &JsValue::from_str("interimResults"),
        &JsValue::FALSE,
    );
    let _ = Reflect::set(
        &recognition,
        &JsValue::from_str("continuous"),
        &JsValue::FALSE,
    );
    let resolved = std::rc::Rc::new(std::cell::Cell::new(false));
    let outcome = completed.clone();
    let ended = resolved.clone();
    let on_result = Closure::<dyn FnMut(JsValue)>::once_assert_unwind_safe(move |event| {
        if !ended.replace(true) {
            outcome.emit(transcription(&event));
        }
    })
    .into_js_value();
    let outcome = completed.clone();
    let ended = resolved.clone();
    let on_error = Closure::<dyn FnMut(JsValue)>::once_assert_unwind_safe(move |event| {
        if !ended.replace(true) {
            let code = property(&event, "error")
                .ok()
                .and_then(|value| value.as_string())
                .unwrap_or_else(|| "unknown error".to_owned());
            outcome.emit(Err(format!("Speech recognition failed: {code}")));
        }
    })
    .into_js_value();
    let on_end = Closure::<dyn FnMut(JsValue)>::once_assert_unwind_safe(move |_| {
        if !resolved.replace(true) {
            completed.emit(Err(
                "No speech detected. Try speaking after allowing microphone access.".to_owned(),
            ));
        }
    })
    .into_js_value();
    for (name, callback) in [
        ("onresult", on_result),
        ("onerror", on_error),
        ("onend", on_end),
    ] {
        Reflect::set(&recognition, &JsValue::from_str(name), &callback)
            .map_err(|_| format!("Could not install speech event handler {name}"))?;
    }
    let start: Function = property(&recognition, "start")?
        .dyn_into()
        .map_err(|_| "Speech recognition start function is unavailable".to_owned())?;
    start.call0(&recognition).map_err(|error| {
        property(&error, "message")
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_else(|| "Unable to start microphone recognition".to_owned())
    })?;
    Ok(())
}
