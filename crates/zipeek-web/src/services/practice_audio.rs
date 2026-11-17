use super::speech::SpeechService;

use crate::preferences::PreferencesStore;

pub(crate) fn autoplay_enabled(storage: &web_sys::Storage) -> bool {
    PreferencesStore::new(storage).read().matching_autoplay
}

pub(crate) fn set_autoplay(storage: &web_sys::Storage, enabled: bool) -> Result<(), String> {
    PreferencesStore::new(storage).update(|settings| settings.matching_autoplay = enabled)
}

fn valid_speech_text(text: &str) -> bool {
    !text.trim().is_empty() && text.chars().count() <= 128
}

pub(crate) fn speak(service: &SpeechService, hanzi: &str) -> Result<(), wasm_bindgen::JsValue> {
    if !valid_speech_text(hanzi) {
        return Err(wasm_bindgen::JsValue::from_str(
            "No Mandarin text to pronounce",
        ));
    }
    service.speak(hanzi, "zh-CN")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speech_limits_exclude_empty_and_unbounded_audio_prompts() {
        assert!(valid_speech_text("你好"));
        assert!(!valid_speech_text("   "));
        assert!(!valid_speech_text(&"汉".repeat(129)));
    }
}
