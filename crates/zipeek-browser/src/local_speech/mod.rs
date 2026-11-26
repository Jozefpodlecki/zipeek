mod browser_api;
mod vad;
pub mod live;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use js_sys::{Array, ArrayBuffer, Float32Array, Function, Promise, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;

use browser_api::{assign, decode_audio, fetch, invoke, new_object, property, wait};
use vad::{SpeechStopDetector, VadConfig};

#[derive(Default)]
struct LocalState {
    enabled: bool,
    trusted_origin: bool,
    generation: u64,
    recording: bool,
    abort: Option<JsValue>,
    audio: Option<JsValue>,
    audio_url: Option<String>,
    microphone: Option<JsValue>,
    audio_context: Option<JsValue>,
    analyser: Option<JsValue>,
    audio_source: Option<JsValue>,
    recording_stop: Option<Function>,
    capture_finishing: bool,
    vad: Option<SpeechStopDetector>,
}

thread_local! {
    static LOCAL: RefCell<LocalState> = RefCell::new(LocalState::default());
}

fn rejected(message: &str) -> JsValue {
    JsValue::from_str(message)
}

fn valid_origin(origin: &str) -> bool {
    matches!(
        origin,
        "http://localhost:1420"
            | "http://localhost.:1420"
            | "http://127.0.0.1:1420"
    )
}

pub fn configure_origin(origin: &str) {
    let trusted = valid_origin(origin);
    if !trusted {
        cancel();
    }
    LOCAL.with(|state| {
        let mut state = state.borrow_mut();
        state.trusted_origin = trusted;
        if !trusted {
            state.enabled = false;
        }
    });
}

pub fn allowed() -> bool {
    LOCAL.with(|state| state.borrow().trusted_origin)
}

pub fn enabled() -> bool {
    allowed() && LOCAL.with(|state| state.borrow().enabled)
}

fn require_enabled() -> Result<(), JsValue> {
    if enabled() {
        Ok(())
    } else {
        Err(rejected(
            "Enable local speech explicitly in Speech settings first",
        ))
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
    let (abort, audio, url, microphone, context, stop) = LOCAL.with(|cell| {
        let mut state = cell.borrow_mut();
        state.generation = state.generation.wrapping_add(1);
        state.recording = false;
        state.capture_finishing = false;
        state.vad = None;
        (
            state.abort.take(),
            state.audio.take(),
            state.audio_url.take(),
            state.microphone.take(),
            state.audio_context.take(),
            state.recording_stop.take(),
        )
    });
    LOCAL.with(|cell| {
        let mut state = cell.borrow_mut();
        state.analyser = None;
        state.audio_source = None;
    });
    if let Some(stop) = stop {
        let _ = stop.call0(&JsValue::NULL);
    }
    if let Some(context) = context {
        let _ = invoke(&context, "close", &[]);
    }
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

pub fn finish_microphone_recording() -> bool {
    let resolver = LOCAL.with(|cell| cell.borrow_mut().recording_stop.take());
    if let Some(resolver) = resolver {
        let _ = resolver.call0(&JsValue::NULL);
        true
    } else {
        false
    }
}

pub fn playback_progress() -> Option<f64> {
    let audio = LOCAL.with(|cell| cell.borrow().audio.clone())?;
    let duration = property(&audio, "duration").ok()?.as_f64()?;
    let current = property(&audio, "currentTime").ok()?.as_f64()?;
    if !duration.is_finite() || duration <= 0.0 || !current.is_finite() {
        return None;
    }
    Some((current / duration).clamp(0.0, 1.0))
}

pub fn seek_playback(fraction: f64) -> bool {
    if !fraction.is_finite() {
        return false;
    }
    let Some(audio) = LOCAL.with(|cell| cell.borrow().audio.clone()) else {
        return false;
    };
    let Some(duration) = property(&audio, "duration").ok().and_then(|value| value.as_f64()) else {
        return false;
    };
    if !duration.is_finite() || duration <= 0.0 {
        return false;
    }
    assign(&audio, "currentTime", &JsValue::from_f64(fraction.clamp(0.0, 1.0) * duration)).is_ok()
}

pub fn microphone_auto_stop_tick() -> bool {
    let analyser = LOCAL.with(|cell| {
        let state = cell.borrow();
        if !state.recording || state.recording_stop.is_none() || state.capture_finishing || state.vad.is_none() {
            return None;
        }
        state.analyser.clone()
    });
    let Some(analyser) = analyser else { return false; };
    let samples = Float32Array::new_with_length(512);
    if invoke(&analyser, "getFloatTimeDomainData", &[samples.clone().into()]).is_err() {
        return false;
    }
    let now_ms = property(&js_sys::global(), "performance")
        .ok()
        .and_then(|clock| invoke(&clock, "now", &[]).ok())
        .and_then(|time| time.as_f64())
        .unwrap_or_else(js_sys::Date::now);
    let should_stop = LOCAL.with(|cell| {
        let mut state = cell.borrow_mut();
        state.vad.as_mut().is_some_and(|vad| vad.observe(&samples.to_vec(), now_ms))
    });
    should_stop && finish_microphone_recording()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AutoStopMode {
    HskWord,
    CustomSentence,
}

impl AutoStopMode {
    fn settings(self) -> (i32, VadConfig) {
        match self {
            Self::HskWord => (9_000, VadConfig { silence_ms: 900.0 }),
            Self::CustomSentence => (28_000, VadConfig { silence_ms: 1_500.0 }),
        }
    }
}

pub fn microphone_processing() -> bool {
    LOCAL.with(|cell| cell.borrow().capture_finishing)
}

pub fn microphone_level() -> Option<f32> {
    let analyser = LOCAL.with(|cell| cell.borrow().analyser.clone())?;
    let data = Float32Array::new_with_length(256);
    invoke(&analyser, "getFloatTimeDomainData", &[data.clone().into()]).ok()?;
    let samples = data.to_vec();
    let rms = (samples.iter().map(|v| v * v).sum::<f32>() / samples.len() as f32).sqrt();
    Some((rms * 5.0).clamp(0.0, 1.0))
}

pub fn microphone_waveform() -> Option<Vec<f32>> {
    let analyser = LOCAL.with(|cell| cell.borrow().analyser.clone())?;
    let samples = Float32Array::new_with_length(512);
    invoke(&analyser, "getFloatTimeDomainData", &[samples.clone().into()]).ok()?;
    Some(microphone_rms_peaks(&samples.to_vec(), 32))
}

pub fn recorded_wav_waveform(wav: &[u8]) -> Vec<f32> {
    if wav.len() < 44 || &wav[..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
        return Vec::new();
    }
    let Some(offset) = wav.windows(4).position(|bytes| bytes == b"data") else {
        return Vec::new();
    };
    if offset + 8 > wav.len() { return Vec::new(); }
    let declared = u32::from_le_bytes(wav[offset + 4..offset + 8].try_into().unwrap_or([0; 4])) as usize;
    let end = (offset + 8).saturating_add(declared).min(wav.len());
    if end <= offset + 8 || end - offset - 8 > 8_000_000 { return Vec::new(); }
    let pcm: Vec<f32> = wav[offset + 8..end].chunks_exact(2)
        .map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]]) as f32 / 32768.0)
        .collect();
    waveform_peaks(&pcm, 32)
}

fn release_audio_meter(generation: u64) {
    let context = LOCAL.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.generation != generation {
            return None;
        }
        state.recording_stop = None;
        state.vad = None;
        state.capture_finishing = true;
        state.analyser = None;
        state.audio_source = None;
        state.audio_context.take()
    });
    if let Some(context) = context {
        let _ = invoke(&context, "close", &[]);
    }
}

pub fn select(local: bool) -> Result<(), JsValue> {
    if local && !allowed() {
        return Err(rejected(
            "Local speech requires the trusted localhost:1420 development app",
        ));
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
        state.capture_finishing = false;
        state.vad = None;
        state.generation
    });
    Ok((generation, abort))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalSpeechCapabilities {
    pub synthesis: bool,
    pub recognition: bool,
}

pub async fn service_capabilities() -> Result<LocalSpeechCapabilities, JsValue> {
    require_enabled()?;
    let response = fetch("/v1/capabilities", "GET", None, None, None).await?;
    let body = wait(invoke(&response, "text", &[])?).await?
        .as_string().unwrap_or_default();
    Ok(LocalSpeechCapabilities {
        synthesis: body.lines().any(|line| line == "synthesis=true"),
        recognition: body.lines().any(|line| line == "recognition=true"),
    })
}

pub async fn capabilities() -> Result<String, JsValue> {
    let state = service_capabilities().await?;
    Ok(format!(
        "Local service: voice {}, microphone {}",
        if state.synthesis { "available" } else { "unavailable" },
        if state.recognition { "available" } else { "unavailable" },
    ))
}

async fn playback_blob(blob: JsValue, generation: u64, abort: &JsValue) -> Result<(), JsValue> {
    let api = property(&js_sys::global(), "URL")?;
    let url = invoke(&api, "createObjectURL", &[blob])?
        .as_string()
        .ok_or_else(|| rejected("Could not create local audio URL"))?;
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
    let signal = property(abort, "signal")?;
    invoke(
        &signal,
        "addEventListener",
        &[JsValue::from_str("abort"), on_abort.as_ref().clone()],
    )?;
    let started = wait(invoke(&audio, "play", &[])?).await;
    if started.is_ok() {
        let _ = JsFuture::from(finished).await;
    }
    let _ = assign(&audio, "onended", &JsValue::NULL);
    let _ = assign(&audio, "onerror", &JsValue::NULL);
    let _ = invoke(
        &signal,
        "removeEventListener",
        &[JsValue::from_str("abort"), on_abort.as_ref().clone()],
    );
    started?;
    if !active(generation) {
        return Err(rejected("Playback cancelled"));
    }
    if failed.get() {
        return Err(rejected("Browser could not play the WAV audio"));
    }
    Ok(())
}

pub async fn play(text: &str) -> Result<String, JsValue> {
    play_with_waveform(text, |_| {}).await
}

pub async fn play_with_waveform(
    text: &str,
    on_waveform: impl FnOnce(Vec<f32>),
) -> Result<String, JsValue> {
    require_enabled()?;
    if text.trim().is_empty() || text.len() > 4096 {
        return Err(rejected("Enter a short Mandarin phrase"));
    }
    cancel();
    let (generation, abort) = begin(false)?;
    let result = async {
        let response = fetch(
            "/v1/tts",
            "POST",
            Some("text/plain; charset=utf-8"),
            Some(&JsValue::from_str(text)),
            Some(&abort),
        )
        .await?;
        let blob = wait(invoke(&response, "blob", &[])?).await?;
        let size = property(&blob, "size")?.as_f64().unwrap_or(0.0);
        if !(44.0..=8_000_000.0).contains(&size) {
            return Err(rejected("Invalid WAV response size"));
        }
        if !active(generation) {
            return Err(rejected("Playback cancelled"));
        }
        if let Ok(buffer) = invoke(&blob, "arrayBuffer", &[]) {
            if let Ok(buffer) = wait(buffer).await {
                if let Ok(context) = new_object("AudioContext", &[]) {
                    let peaks = async {
                        let decoded = wait(invoke(&context, "decodeAudioData", &[buffer])?).await?;
                        let channel = invoke(&decoded, "getChannelData", &[JsValue::from_f64(0.0)])?;
                        Ok::<_, JsValue>(waveform_peaks(&Float32Array::new(&channel).to_vec(), 32))
                    }.await;
                    let _ = invoke(&context, "close", &[]);
                    if active(generation) {
                        if let Ok(peaks) = peaks { on_waveform(peaks); }
                    }
                }
            }
        }
        playback_blob(blob, generation, &abort).await?;
        Ok("Local Mandarin WAV playback completed".to_owned())
    }
    .await;
    if active(generation) {
        cancel();
    }
    result
}

pub fn download_recorded_wav(wav: &[u8]) -> Result<(), JsValue> {
    if !allowed() || wav.len() < 44 || wav.len() > 8_000_000 || &wav[..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
        return Err(rejected("No valid locally recorded WAV to download"));
    }
    let parts = Array::new();
    parts.push(&Uint8Array::from(wav).into());
    let options = js_sys::Object::new();
    assign(&options, "type", &JsValue::from_str("audio/wav"))?;
    let blob = new_object("Blob", &[parts.into(), options.into()])?;
    let url_api = property(&js_sys::global(), "URL")?;
    let url = invoke(&url_api, "createObjectURL", &[blob])?
        .as_string().ok_or_else(|| rejected("Could not create WAV download URL"))?;
    let result = (|| {
        let document = property(&js_sys::global(), "document")?;
        let anchor = invoke(&document, "createElement", &[JsValue::from_str("a")])?;
        assign(&anchor, "href", &JsValue::from_str(&url))?;
        assign(&anchor, "download", &JsValue::from_str("zipeek-recording.wav"))?;
        let body = property(&document, "body")?;
        invoke(&body, "appendChild", &[anchor.clone()])?;
        let clicked = invoke(&anchor, "click", &[]);
        let _ = invoke(&body, "removeChild", &[anchor]);
        clicked?;
        Ok::<(), JsValue>(())
    })();
    if result.is_ok() {
        let cleanup_url = url.clone();
        let cleanup = Closure::once_into_js(move || revoke(&cleanup_url));
        if invoke(&js_sys::global(), "setTimeout", &[cleanup, JsValue::from_f64(1000.0)]).is_err() {
            revoke(&url);
        }
    } else {
        revoke(&url);
    }
    result
}

pub async fn play_recorded_wav(wav: &[u8]) -> Result<(), JsValue> {
    require_enabled()?;
    if wav.len() < 44 || wav.len() > 8_000_000 {
        return Err(rejected("Invalid recorded WAV size"));
    }
    cancel();
    let (generation, abort) = begin(false)?;
    let result = async {
        let parts = Array::new();
        parts.push(&Uint8Array::from(wav).into());
        let options = js_sys::Object::new();
        assign(&options, "type", &JsValue::from_str("audio/wav"))?;
        let blob = new_object("Blob", &[parts.into(), options.into()])?;
        playback_blob(blob, generation, &abort).await
    }
    .await;
    if active(generation) {
        cancel();
    }
    result
}

pub struct RecordedTranscription {
    pub wav: Vec<u8>,
    pub transcript: Result<String, JsValue>,
}

pub async fn transcribe_microphone_with_audio() -> Result<RecordedTranscription, JsValue> {
    let (generation, abort) = begin(true)?;
    let result = async {
        let wav = capture(generation, 5_000, None).await?;
        let transcript = send_audio(&wav, generation, &abort).await;
        Ok(RecordedTranscription { wav, transcript })
    }
    .await;
    if active(generation) {
        cancel();
    }
    result
}

pub async fn transcribe_microphone_with_auto_stop(mode: AutoStopMode) -> Result<RecordedTranscription, JsValue> {
    let (generation, abort) = begin(true)?;
    let (maximum_ms, detector) = mode.settings();
    let result = async {
        let wav = capture(generation, maximum_ms, Some(detector)).await?;
        let transcript = send_audio(&wav, generation, &abort).await;
        Ok(RecordedTranscription { wav, transcript })
    }.await;
    if active(generation) { cancel(); }
    result
}

async fn send_audio(wav: &[u8], generation: u64, abort: &JsValue) -> Result<String, JsValue> {
    if !active(generation) {
        return Err(rejected("Transcription cancelled"));
    }
    let bytes = Uint8Array::from(wav);
    let response = fetch(
        "/v1/stt",
        "POST",
        Some("audio/wav"),
        Some(&bytes.into()),
        Some(abort),
    )
    .await?;
    let transcript = wait(invoke(&response, "text", &[])?)
        .await?
        .as_string()
        .unwrap_or_default()
        .trim()
        .to_owned();
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
        return Err(rejected(
            "Only an in-memory browser recording may be transcribed",
        ));
    }
    let (generation, abort) = begin(true)?;
    let result = async {
        let response = wait(invoke(
            &js_sys::global(),
            "fetch",
            &[JsValue::from_str(url)],
        )?)
        .await?;
        let data = wait(invoke(&response, "arrayBuffer", &[])?)
            .await?
            .dyn_into::<ArrayBuffer>()?;
        if data.byte_length() == 0 || data.byte_length() > 8_000_000 {
            return Err(rejected("Recording exceeds local transcription size limit"));
        }
        let wav = decode_audio(data).await?;
        send_audio(&wav, generation, &abort).await
    }
    .await;
    if active(generation) {
        cancel();
    }
    result
}

pub async fn transcribe_microphone() -> Result<String, JsValue> {
    transcribe_microphone_with_audio().await?.transcript
}

async fn capture(generation: u64, maximum_ms: i32, vad_config: Option<VadConfig>) -> Result<Vec<u8>, JsValue> {
    let navigator = property(&js_sys::global(), "navigator")?;
    let devices = property(&navigator, "mediaDevices")?;
    if devices.is_null() || devices.is_undefined() {
        return Err(rejected(
            "Microphone recording is not supported by this browser",
        ));
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
    if let Ok(context) = new_object("AudioContext", &[]) {
        if let (Ok(source), Ok(analyser)) = (
            invoke(&context, "createMediaStreamSource", &[stream.clone()]),
            invoke(&context, "createAnalyser", &[]),
        ) {
            let _ = assign(&analyser, "fftSize", &JsValue::from_f64(512.0));
            if invoke(&source, "connect", &[analyser.clone()]).is_ok() {
                LOCAL.with(|cell| {
                    let mut state = cell.borrow_mut();
                    if state.generation == generation {
                        state.audio_context = Some(context.clone());
                        state.audio_source = Some(source);
                        state.analyser = Some(analyser);
                        state.vad = vad_config.map(SpeechStopDetector::new);
                    }
                });
            }
        }
        if !LOCAL.with(|cell| cell.borrow().audio_context.is_some()) {
            let _ = invoke(&context, "close", &[]);
        }
    }
    let recorder = new_object("MediaRecorder", &[stream.clone()])?;
    let chunks = Rc::new(RefCell::new(Vec::<JsValue>::new()));
    let on_data = {
        let chunks = chunks.clone();
        Closure::<dyn FnMut(JsValue)>::new(move |event| {
            if let Ok(blob) = property(&event, "data") {
                if property(&blob, "size")
                    .ok()
                    .and_then(|value| value.as_f64())
                    .unwrap_or(0.0)
                    > 0.0
                {
                    chunks.borrow_mut().push(blob);
                }
            }
        })
    };
    assign(&recorder, "ondataavailable", on_data.as_ref())?;
    invoke(&recorder, "start", &[])?;
    let window = web_sys::window().ok_or_else(|| rejected("Browser window unavailable"))?;
    let timer = Promise::new(&mut |resolve, _reject| {
        LOCAL.with(|cell| {
            let mut state = cell.borrow_mut();
            if state.generation == generation {
                state.recording_stop = Some(resolve.clone());
            }
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, maximum_ms);
    });
    let _ = JsFuture::from(timer).await;
    let no_speech = LOCAL.with(|cell| {
        let state = cell.borrow();
        state.generation == generation && state.vad.as_ref().is_some_and(|vad| !vad.heard_speech())
    });
    release_audio_meter(generation);
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
    if no_speech {
        return Err(rejected("No speech detected"));
    }
    let parts = Array::new();
    for chunk in chunks.borrow().iter() {
        parts.push(chunk);
    }
    let blob = new_object("Blob", &[parts.into()])?;
    let data = wait(invoke(&blob, "arrayBuffer", &[])?)
        .await?
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
        assert!(valid_origin("http://localhost.:1420"));
        assert!(valid_origin("http://127.0.0.1:1420"));
        assert!(!valid_origin("http://localhost:1421"));
        assert!(!valid_origin("http://localhost.:1421"));
        assert!(!valid_origin("http://localhost..:1420"));
        assert!(!valid_origin("http://localhost.:1420.evil.test"));
        assert!(!valid_origin("http://localhost.:1420@evil.test"));
        assert!(!valid_origin("https://localhost:1420"));
        assert!(!valid_origin("http://localhost:1420.evil.test"));
    }
}

#[derive(Clone, PartialEq)]
pub struct MatchingAudio {
    pub url: String,
    pub peaks: Vec<f32>,
    pub duration_seconds: f64,
}

fn microphone_rms_peaks(samples: &[f32], columns: usize) -> Vec<f32> {
    if samples.is_empty() || columns == 0 {
        return Vec::new();
    }
    (0..columns)
        .map(|column| {
            let start = column * samples.len() / columns;
            let end = ((column + 1) * samples.len() / columns).max(start + 1).min(samples.len());
            let window = &samples[start..end];
            let power = window.iter().copied().filter(|sample| sample.is_finite())
                .map(|sample| sample * sample).sum::<f32>() / window.len() as f32;
            ((power.sqrt() - 0.012).max(0.0) * 6.0).clamp(0.0, 1.0)
        })
        .collect()
}

fn waveform_peaks(samples: &[f32], columns: usize) -> Vec<f32> {
    if samples.is_empty() || columns == 0 {
        return Vec::new();
    }
    let mut peaks = (0..columns).map(|column| {
        let start = column * samples.len() / columns;
        let end = ((column + 1) * samples.len() / columns).max(start + 1).min(samples.len());
        samples[start..end].iter().copied().filter(|value| value.is_finite())
            .map(f32::abs).fold(0.0_f32, f32::max)
    }).collect::<Vec<_>>();
    let maximum = peaks.iter().copied().fold(0.0_f32, f32::max);
    if maximum > 0.0 {
        for peak in &mut peaks {
            *peak = (*peak / maximum).clamp(0.0, 1.0);
        }
    }
    peaks
}

pub async fn prepare_matching_audio(text: &str) -> Result<MatchingAudio, JsValue> {
    prepare_audio_with_limit(text, 256, 30.0).await
}

pub async fn prepare_record_reference_audio(text: &str) -> Result<MatchingAudio, JsValue> {
    if text.chars().count() > 200 {
        return Err(rejected("Reference text exceeds 200 characters"));
    }
    prepare_audio_with_limit(text, 4096, 90.0).await
}

async fn prepare_audio_with_limit(text: &str, limit: usize, max_duration: f64) -> Result<MatchingAudio, JsValue> {
    require_enabled()?;
    if text.trim().is_empty() || text.len() > limit || text.chars().any(char::is_control) {
        return Err(rejected("Invalid matching audio text"));
    }
    let response = fetch(
        "/v1/tts",
        "POST",
        Some("text/plain; charset=utf-8"),
        Some(&JsValue::from_str(text)),
        None,
    )
    .await?;
    let raw = wait(invoke(&response, "arrayBuffer", &[])?).await?;
    let length = property(&raw, "byteLength")?.as_f64().unwrap_or(0.0);
    if !(44.0..=8_000_000.0).contains(&length) {
        return Err(rejected("Invalid matching audio size"));
    }
    let context = new_object("AudioContext", &[])?;
    let decoded = async {
        let copy = invoke(&raw, "slice", &[JsValue::from_f64(0.0)])?;
        let audio = wait(invoke(&context, "decodeAudioData", &[copy])?).await?;
        let duration = property(&audio, "duration")?.as_f64().unwrap_or(0.0);
        let channels = property(&audio, "numberOfChannels")?.as_f64().unwrap_or(0.0);
        let samples = property(&audio, "length")?.as_f64().unwrap_or(0.0) as usize;
        if !(0.05..=max_duration).contains(&duration) || !(1.0..=2.0).contains(&channels) || samples == 0 {
            return Err(rejected("Invalid decoded matching audio"));
        }
        let channel = invoke(&audio, "getChannelData", &[JsValue::from_f64(0.0)])?;
        let pcm = js_sys::Float32Array::new(&channel).to_vec();
        let peaks = waveform_peaks(&pcm, 32);
        let parts = Array::new();
        parts.push(&raw);
        let options = js_sys::Object::new();
        assign(&options, "type", &JsValue::from_str("audio/wav"))?;
        let blob = new_object("Blob", &[parts.into(), options.into()])?;
        let api = property(&js_sys::global(), "URL")?;
        let url = invoke(&api, "createObjectURL", &[blob])?
            .as_string()
            .ok_or_else(|| rejected("Cannot prepare audio playback URL"))?;
        Ok(MatchingAudio { url, peaks, duration_seconds: duration })
    }.await;
    if let Ok(closing) = invoke(&context, "close", &[]) {
        let _ = wait(closing).await;
    }
    decoded
}

pub async fn play_prepared_reference(audio: &MatchingAudio) -> Result<(), JsValue> {
    require_enabled()?;
    cancel();
    let (generation, abort) = begin(false)?;
    let result = async {
        let response = wait(invoke(&js_sys::global(), "fetch", &[JsValue::from_str(&audio.url)])?).await?;
        let blob = wait(invoke(&response, "blob", &[])?) .await?;
        playback_blob(blob, generation, &abort).await
    }.await;
    if active(generation) { cancel(); }
    result
}

pub fn dispose_matching_audio(audio: &MatchingAudio) {
    revoke(&audio.url);
}

pub async fn play_matching_audio(audio: &MatchingAudio) -> Result<(), JsValue> {
    require_enabled()?;
    cancel();
    let element = new_object("Audio", &[JsValue::from_str(&audio.url)])?;
    LOCAL.with(|cell| cell.borrow_mut().audio = Some(element.clone()));
    wait(invoke(&element, "play", &[])?).await?;
    Ok(())
}

#[cfg(test)]
mod matching_waveform_tests {
    use super::{microphone_rms_peaks, waveform_peaks};

    #[test]
    fn peaks_are_normalized_and_bounded() {
        assert!(waveform_peaks(&[], 32).is_empty());
        assert!(microphone_rms_peaks(&[0.0; 512], 32).iter().all(|value| *value == 0.0));
        assert!(microphone_rms_peaks(&[0.005; 512], 32).iter().all(|value| *value == 0.0));
        assert!(microphone_rms_peaks(&[0.3; 512], 32).iter().all(|value| *value > 0.0));
        assert!(waveform_peaks(&[0.0; 64], 4).iter().all(|value| *value == 0.0));
        assert_eq!(waveform_peaks(&[0.0, 0.5, 1.0, -0.25], 2), vec![0.5, 1.0]);
        assert!(waveform_peaks(&[f32::NAN, 0.3], 4).iter().all(|value| value.is_finite()));
    }
}
