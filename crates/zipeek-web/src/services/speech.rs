use std::cell::Cell;

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use zipeek_browser::local_speech;

use web_sys::{SpeechSynthesisUtterance, SpeechSynthesisVoice, Window};

thread_local! {
    static NATIVE_RETRY_AFTER_MS: Cell<f64> = const { Cell::new(0.0) };
    static SPEECH_REQUEST_ID: Cell<u64> = const { Cell::new(0) };
    static FALLBACK_NOTIFIED: Cell<bool> = const { Cell::new(false) };
    static UNAVAILABLE_NOTIFIED: Cell<bool> = const { Cell::new(false) };
}

const FALLBACK_EVENT: &str = "zipeek:speech-browser-fallback";
const UNAVAILABLE_EVENT: &str = "zipeek:speech-unavailable";

fn announce_once(
    window: &Window,
    event_name: &str,
    notified: &'static std::thread::LocalKey<Cell<bool>>,
) {
    notified.with(|sent| {
        if !sent.replace(true) {
            if let Ok(event) = web_sys::Event::new(event_name) {
                let _ = window.dispatch_event(&event);
            }
        }
    });
}

fn browser_speak(window: &Window, text: &str, language: &str) -> Result<(), JsValue> {
    let synthesis = window.speech_synthesis()?;
    let voices = synthesis.get_voices();
    let preferred = voices
        .iter()
        .filter_map(|voice| voice.dyn_into::<SpeechSynthesisVoice>().ok())
        .find(|voice| voice.lang().eq_ignore_ascii_case(language));
    let matching = preferred.or_else(|| {
        synthesis
            .get_voices()
            .iter()
            .filter_map(|voice| voice.dyn_into::<SpeechSynthesisVoice>().ok())
            .find(|voice| voice.lang().to_ascii_lowercase().starts_with("zh"))
    });
    if language.starts_with("zh") && matching.is_none() {
        return Err(JsValue::from_str("No Mandarin browser voice is available"));
    }
    let utterance = SpeechSynthesisUtterance::new_with_text(text)?;
    utterance.set_lang(language);
    if let Some(voice) = matching {
        utterance.set_voice(Some(&voice));
    }
    synthesis.cancel();
    synthesis.speak(&utterance);
    Ok(())
}

pub(crate) const E2E_SETTINGS_KEY: &str = "zipeek.e2e.settings.v1";
pub(crate) const E2E_SPEECH_EVENTS_KEY: &str = "zipeek.e2e.speech.events.v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SpeechMode {
    #[default]
    Browser,
    Silent,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct E2eSettings {
    speech: Option<String>,
}

pub(crate) fn speech_mode(settings: Option<&str>, e2e_build: bool) -> SpeechMode {
    if !e2e_build {
        return SpeechMode::Browser;
    }
    let parsed = settings.and_then(|value| serde_json::from_str::<E2eSettings>(value).ok());
    match parsed.and_then(|settings| settings.speech) {
        Some(value) if value == "silent" => SpeechMode::Silent,
        _ => SpeechMode::Browser,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum SpeechEvent {
    Speak { text: String, language: String },
    Cancel,
}

#[derive(Clone, Debug)]
pub(crate) struct SpeechService {
    window: Window,
    mode: SpeechMode,
}

impl SpeechService {
    pub(crate) fn new(window: Window, mode: SpeechMode) -> Self {
        Self { window, mode }
    }

    fn record(&self, event: SpeechEvent) {
        if let Some(storage) = self.window.session_storage().ok().flatten() {
            let mut events = storage
                .get_item(E2E_SPEECH_EVENTS_KEY)
                .ok()
                .flatten()
                .and_then(|value| serde_json::from_str::<Vec<SpeechEvent>>(&value).ok())
                .unwrap_or_default();
            events.push(event);
            if let Ok(serialized) = serde_json::to_string(&events) {
                let _ = storage.set_item(E2E_SPEECH_EVENTS_KEY, &serialized);
            }
        }
    }

    pub(crate) fn speak(&self, text: &str, language: &str) -> Result<(), wasm_bindgen::JsValue> {
        if text.trim().is_empty() || text.chars().count() > 128 {
            return Err(wasm_bindgen::JsValue::from_str(
                "No valid text to pronounce",
            ));
        }
        if self.mode == SpeechMode::Silent {
            self.record(SpeechEvent::Cancel);
            self.record(SpeechEvent::Speak {
                text: text.to_owned(),
                language: language.to_owned(),
            });
            return Ok(());
        }
        let request_id = SPEECH_REQUEST_ID.with(|current| {
            let next = current.get().wrapping_add(1);
            current.set(next);
            next
        });
        if language == "zh-CN" && local_speech::enabled() {
            let retry_native =
                NATIVE_RETRY_AFTER_MS.with(|retry| js_sys::Date::now() >= retry.get());
            if retry_native {
                self.window.speech_synthesis()?.cancel();
                let text = text.to_owned();
                let window = self.window.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = local_speech::play(&text).await;
                    if !SPEECH_REQUEST_ID.with(|current| current.get() == request_id) {
                        return;
                    }
                    match result {
                        Ok(_) => {
                            NATIVE_RETRY_AFTER_MS.with(|retry| retry.set(0.0));
                        }
                        Err(error) => {
                            log::warn!("Local Mandarin playback failed: {error:?}");
                            NATIVE_RETRY_AFTER_MS
                                .with(|retry| retry.set(js_sys::Date::now() + 60_000.0));
                            match browser_speak(&window, &text, "zh-CN") {
                                Ok(()) => {
                                    announce_once(&window, FALLBACK_EVENT, &FALLBACK_NOTIFIED)
                                }
                                Err(browser_error) => {
                                    log::warn!(
                                        "Browser Mandarin fallback unavailable: {browser_error:?}"
                                    );
                                    announce_once(
                                        &window,
                                        UNAVAILABLE_EVENT,
                                        &UNAVAILABLE_NOTIFIED,
                                    );
                                }
                            }
                        }
                    }
                });
                return Ok(());
            }
        }
        local_speech::cancel();
        let result = browser_speak(&self.window, text, language);
        if result.is_ok() && language == "zh-CN" && local_speech::enabled() {
            announce_once(&self.window, FALLBACK_EVENT, &FALLBACK_NOTIFIED);
        } else if result.is_err() && language == "zh-CN" {
            announce_once(&self.window, UNAVAILABLE_EVENT, &UNAVAILABLE_NOTIFIED);
        }
        result
    }

    pub(crate) fn cancel(&self) -> Result<(), wasm_bindgen::JsValue> {
        if self.mode == SpeechMode::Silent {
            self.record(SpeechEvent::Cancel);
            return Ok(());
        }
        SPEECH_REQUEST_ID.with(|current| current.set(current.get().wrapping_add(1)));
        local_speech::cancel();
        self.window.speech_synthesis()?.cancel();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_mode_requires_e2e_build_and_explicit_valid_settings() {
        let settings = r#"{"speech":"silent"}"#;
        assert_eq!(speech_mode(Some(settings), false), SpeechMode::Browser);
        assert_eq!(speech_mode(Some(settings), true), SpeechMode::Silent);
        assert_eq!(speech_mode(None, true), SpeechMode::Browser);
        assert_eq!(speech_mode(Some("null"), true), SpeechMode::Browser);
        assert_eq!(
            speech_mode(Some(r#"{"speech":"invalid"}"#), true),
            SpeechMode::Browser
        );
        assert_eq!(
            speech_mode(Some(r#"{"speech":"silent","other":true}"#), true),
            SpeechMode::Browser
        );
    }
}

impl PartialEq for SpeechService {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode && self.window == other.window
    }
}
