use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use zipeek_browser::local_speech;

use web_sys::{SpeechSynthesisUtterance, Window};

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
        if language == "zh-CN" && local_speech::enabled() {
            self.window.speech_synthesis()?.cancel();
            let text = text.to_owned();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(error) = local_speech::play(&text).await {
                    log::warn!("Local Mandarin playback failed: {error:?}");
                }
            });
            return Ok(());
        }
        local_speech::cancel();
        let synthesis = self.window.speech_synthesis()?;
        let utterance = SpeechSynthesisUtterance::new_with_text(text)?;
        utterance.set_lang(language);
        synthesis.cancel();
        synthesis.speak(&utterance);
        Ok(())
    }

    pub(crate) fn cancel(&self) -> Result<(), wasm_bindgen::JsValue> {
        if self.mode == SpeechMode::Silent {
            self.record(SpeechEvent::Cancel);
            return Ok(());
        }
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
