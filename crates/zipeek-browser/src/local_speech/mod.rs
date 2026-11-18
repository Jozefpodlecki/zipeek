mod browser_api;

use std::{cell::{Cell, RefCell}, rc::Rc};

use js_sys::{Array, ArrayBuffer, Function, Promise, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;

use browser_api::{assign, decode_audio, fetch, invoke, new_object, property, wait};

#[derive(Default)]
struct LocalState {
    enabled: bool,
    generation: u64,
    recording: bool,
    abort: Option<JsValue>,
    audio: Option<JsValue>,
    audio_url: Option<String>,
    microphone: Option<JsValue>,
}

thread_local! {
    static LOCAL: RefCell<LocalState> = RefCell::new(LocalState::default());
}

fn rejected(message: &str) -> JsValue {
    JsValue::from_str(message)
}

fn valid_origin(origin: &str) -> bool {
    matches!(origin, "http://localhost:1420" | "http://127.0.0.1:1420")
}

pub fn allowed() -> bool {
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .is_some_and(|origin| valid_origin(&origin))
}

pub fn enabled() -> bool {
    allowed() && LOCAL.with(|state| state.borrow().enabled)
}

fn require_enabled() -> Result<(), JsValue> {
    if enabled() {
        Ok(())
    } else {
        Err(rejected("Enable local speech explicitly in Speech settings first"))
    }
}

fn stop_tracks(stream: &JsValue) {
    if let Ok(tracks) = invoke(stream, "getTracks", &[]) {
        for track in Array::from(&tracks).iter() {
            let _ = invoke(&track, "stop", &[]);
        }
    }
}

fn revoke(url: &str) {
    if let Ok(api) = property(&js_sys::global(), "URL") {
        let _ = invoke(&api, "revokeObjectURL", &[JsValue::from_str(url)]);
    }
}

pub fn cancel() {
    let (abort, audio, url, microphone) = LOCAL.with(|cell| {
        let mut state = cell.borrow_mut();
        state.generation = state.generation.wrapping_add(1);
        state.recording = false;
        (
            state.abort.take(),
            state.audio.take(),
            state.audio_url.take(),
            state.microphone.take(),
        )
    });
    if let Some(abort) = abort {
        let _ = invoke(&abort, "abort", &[]);
    }
    if let Some(audio) = audio {
        let _ = invoke(&audio, "pause", &[]);
        let _ = invoke(&audio, "removeAttribute", &[JsValue::from_str("src")]);
        let _ = invoke(&audio, "load", &[]);
    }
    if let Some(url) = url {
        revoke(&url);
    }
    if let Some(microphone) = microphone {
        stop_tracks(&microphone);
    }
}

pub fn select(local: bool) -> Result<(), JsValue> {
    if local && !allowed() {
        return Err(rejected("Local speech requires the trusted localhost:1420 development app"));
    }
    cancel();
    LOCAL.with(|state| state.borrow_mut().enabled = local);
    Ok(())
}

fn active(generation: u64) -> bool {
    enabled() && LOCAL.with(|state| state.borrow().generation == generation)
}

fn begin(recording: bool) -> Result<(u64, JsValue), JsValue> {
    require_enabled()?;
    if recording && LOCAL.with(|state| state.borrow().recording) {
        return Err(rejected("A local transcription is already running"));
    }
    if recording {
        cancel();
    }
    let abort = new_object("AbortController", &[])?;
    let generation = LOCAL.with(|state| {
        let mut state = state.borrow_mut();
        state.generation = state.generation.wrapping_add(1);
        state.abort = Some(abort.clone());
        state.recording = recording;
        state.generation
    });
    Ok((generation, abort))
}

pub async fn capabilities() -> Result<String, JsValue> {
    require_enabled()?;
    let response = fetch("/v1/capabilities", "GET", None, None, None).await?;
    let body = wait(invoke(&response, "text", &[])?).await?
        .as_string().unwrap_or_default();
    let synthesis = body.lines().any(|line| line == "synthesis=true");
    let recognition = body.lines().any(|line| line == "recognition=true");
    Ok(format!(
        "Local service: voice {}, microphone {}",
        if synthesis { "available" } else { "unavailable" },
        if recognition { "available" } else { "unavailable" }
    ))
}

pub async fn play(text: &str) -> Result<String, JsValue> {
    require_enabled()?;
    if text.trim().is_empty() || text.len() > 4096 {
        return Err(rejected("Enter a short Mandarin phrase"));
    }
    cancel();
    let (generation, abort) = begin(false)?;
    let result = async {
        let response = fetch(
            "/v1/tts", "POST", Some("text/plain; charset=utf-8"),
            Some(&JsValue::from_str(text)), Some(&abort),
        ).await?;
        let blob = wait(invoke(&response, "blob", &[])?).await?;
        let size = property(&blob, "size")?.as_f64().unwrap_or(0.0);
        if !(44.0..=8_000_000.0).contains(&size) {
            return Err(rejected("Invalid WAV response size"));
        }
        if !active(generation) {
            return Err(rejected("Playback cancelled"));
        }
        let api = property(&js_sys::global(), "URL")?;
        let url = invoke(&api, "createObjectURL", &[blob])?
            .as_string().ok_or_else(|| rejected("Could not create local audio URL"))?;
        let audio = match new_object("Audio", &[JsValue::from_str(&url)]) {
            Ok(audio) => audio,
            Err(error) => {
                revoke(&url);
                return Err(error);
            }
        };
        let owned = LOCAL.with(|cell| {
            let mut state = cell.borrow_mut();
            if state.generation != generation {
                return false;
            }
            state.audio = Some(audio.clone());
            state.audio_url = Some(url.clone());
            true
        });
        if !owned {
            revoke(&url);
            return Err(rejected("Playback cancelled"));
        }
        let resolver = Rc::new(RefCell::new(None::<Function>));
        let finished = Promise::new(&mut {
            let resolver = resolver.clone();
            move |resolve, _reject| {
                *resolver.borrow_mut() = Some(resolve);
            }
        });
        let failed = Rc::new(Cell::new(false));
        let on_end = {
            let resolver = resolver.clone();
            Closure::<dyn FnMut()>::new(move || {
                if let Some(resolve) = resolver.borrow().as_ref() {
                    let _ = resolve.call0(&JsValue::NULL);
                }
            })
        };
        let on_error = {
            let resolver = resolver.clone();
            let failed = failed.clone();
            Closure::<dyn FnMut()>::new(move || {
                failed.set(true);
                if let Some(resolve) = resolver.borrow().as_ref() {
                    let _ = resolve.call0(&JsValue::NULL);
                }
            })
        };
        let on_abort = {
            let resolver = resolver.clone();
            Closure::<dyn FnMut()>::new(move || {
                if let Some(resolve) = resolver.borrow().as_ref() {
                    let _ = resolve.call0(&JsValue::NULL);
                }
            })
        };
        assign(&audio, "onended", on_end.as_ref())?;
        assign(&audio, "onerror", on_error.as_ref())?;
        let signal = property(&abort, "signal")?;
        invoke(&signal, "addEventListener", &[
            JsValue::from_str("abort"), on_abort.as_ref().clone(),
        ])?;
        let started = wait(invoke(&audio, "play", &[])?).await;
        if started.is_ok() {
            let _ = JsFuture::from(finished).await;
        }
        let _ = assign(&audio, "onended", &JsValue::NULL);
        let _ = assign(&audio, "onerror", &JsValue::NULL);
        let _ = invoke(&signal, "removeEventListener", &[
            JsValue::from_str("abort"), on_abort.as_ref().clone(),
        ]);
        started?;
        if !active(generation) {
            return Err(rejected("Playback cancelled"));
        }
        if failed.get() {
            return Err(rejected("Browser could not play the WAV audio"));
        }
        Ok("Local Mandarin WAV playback completed".to_owned())
    }.await;
    if active(generation) {
        cancel();
    }
    result
}

async fn send_audio(wav: &[u8], generation: u64, abort: &JsValue) -> Result<String, JsValue> {
    if !active(generation) {
        return Err(rejected("Transcription cancelled"));
    }
    let bytes = Uint8Array::from(wav);
    let response = fetch("/v1/stt", "POST", Some("audio/wav"), Some(&bytes.into()), Some(abort)).await?;
    let transcript = wait(invoke(&response, "text", &[])?).await?
        .as_string().unwrap_or_default().trim().to_owned();
    if !active(generation) {
        return Err(rejected("Transcription cancelled"));
    }
    if transcript.is_empty() {
        return Err(rejected("No Mandarin speech detected"));
    }
    Ok(transcript)
}

pub async fn transcribe_recording(url: &str) -> Result<String, JsValue> {
    require_enabled()?;
    if !url.starts_with("blob:") {
        return Err(rejected("Only an in-memory browser recording may be transcribed"));
    }
    let (generation, abort) = begin(true)?;
    let result = async {
        let response = wait(invoke(&js_sys::global(), "fetch", &[JsValue::from_str(url)])?).await?;
        let data = wait(invoke(&response, "arrayBuffer", &[])?).await?
            .dyn_into::<ArrayBuffer>()?;
        if data.byte_length() == 0 || data.byte_length() > 8_000_000 {
            return Err(rejected("Recording exceeds local transcription size limit"));
        }
        let wav = decode_audio(data).await?;
        send_audio(&wav, generation, &abort).await
    }.await;
    if active(generation) {
        cancel();
    }
    result
}

pub async fn transcribe_microphone() -> Result<String, JsValue> {
    let (generation, abort) = begin(true)?;
    let result = async {
        let wav = capture(generation).await?;
        send_audio(&wav, generation, &abort).await
    }.await;
    if active(generation) {
        cancel();
    }
    result
}

async fn capture(generation: u64) -> Result<Vec<u8>, JsValue> {
    let navigator = property(&js_sys::global(), "navigator")?;
    let devices = property(&navigator, "mediaDevices")?;
    if devices.is_null() || devices.is_undefined() {
        return Err(rejected("Microphone recording is not supported by this browser"));
    }
    let constraints = js_sys::Object::new();
    assign(&constraints, "audio", &JsValue::TRUE)?;
    assign(&constraints, "video", &JsValue::FALSE)?;
    let stream = wait(invoke(&devices, "getUserMedia", &[constraints.into()])?).await?;
    LOCAL.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.generation == generation {
            state.microphone = Some(stream.clone());
        }
    });
    if !active(generation) {
        stop_tracks(&stream);
        return Err(rejected("Recording cancelled"));
    }
    let recorder = new_object("MediaRecorder", &[stream.clone()])?;
    let chunks = Rc::new(RefCell::new(Vec::<JsValue>::new()));
    let on_data = {
        let chunks = chunks.clone();
        Closure::<dyn FnMut(JsValue)>::new(move |event| {
            if let Ok(blob) = property(&event, "data") {
                if property(&blob, "size").ok().and_then(|value| value.as_f64()).unwrap_or(0.0) > 0.0 {
                    chunks.borrow_mut().push(blob);
                }
            }
        })
    };
    assign(&recorder, "ondataavailable", on_data.as_ref())?;
    invoke(&recorder, "start", &[])?;
    let window = web_sys::window().ok_or_else(|| rejected("Browser window unavailable"))?;
    let timer = Promise::new(&mut |resolve, _reject| {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 5000);
    });
    let _ = JsFuture::from(timer).await;
    if !active(generation) {
        let _ = invoke(&recorder, "stop", &[]);
        let _ = assign(&recorder, "ondataavailable", &JsValue::NULL);
        return Err(rejected("Recording cancelled"));
    }
    let resolver = Rc::new(RefCell::new(None::<Function>));
    let stopped = Promise::new(&mut {
        let resolver = resolver.clone();
        move |resolve, _reject| {
            *resolver.borrow_mut() = Some(resolve);
        }
    });
    let on_stop = Closure::<dyn FnMut()>::new(move || {
        if let Some(resolve) = resolver.borrow().as_ref() {
            let _ = resolve.call0(&JsValue::NULL);
        }
    });
    assign(&recorder, "onstop", on_stop.as_ref())?;
    let stop_result = invoke(&recorder, "stop", &[]);
    if stop_result.is_ok() {
        let _ = JsFuture::from(stopped).await;
    }
    let _ = assign(&recorder, "onstop", &JsValue::NULL);
    let _ = assign(&recorder, "ondataavailable", &JsValue::NULL);
    stop_result?;
    stop_tracks(&stream);
    LOCAL.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.generation == generation {
            state.microphone = None;
        }
    });
    if !active(generation) {
        return Err(rejected("Recording cancelled"));
    }
    let parts = Array::new();
    for chunk in chunks.borrow().iter() {
        parts.push(chunk);
    }
    let blob = new_object("Blob", &[parts.into()])?;
    let data = wait(invoke(&blob, "arrayBuffer", &[])?).await?
        .dyn_into::<ArrayBuffer>()?;
    if data.byte_length() == 0 || data.byte_length() > 8_000_000 {
        return Err(rejected("Invalid recorded microphone audio size"));
    }
    decode_audio(data).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_must_match_an_explicit_development_origin() {
        assert!(valid_origin("http://localhost:1420"));
        assert!(valid_origin("http://127.0.0.1:1420"));
        assert!(!valid_origin("http://localhost:1421"));
        assert!(!valid_origin("https://localhost:1420"));
        assert!(!valid_origin("http://localhost:1420.evil.test"));
    }
}
