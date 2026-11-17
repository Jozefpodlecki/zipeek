use crate::hooks::use_app_environment;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SpeechProvider {
    Browser,
    Native,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SpeechProviderHandle {
    pub selected: SpeechProvider,
    pub native_available: bool,
    pub desktop: bool,
}

impl SpeechProviderHandle {
    pub fn supports_provider_selection(self) -> bool {
        self.desktop && self.native_available
    }
}

#[hook]
pub fn use_tts_service() -> SpeechProviderHandle {
    let environment = use_app_environment();
    SpeechProviderHandle {
        selected: SpeechProvider::Browser,
        native_available: false,
        desktop: environment.is_tauri,
    }
}

#[hook]
pub fn use_stt_service() -> SpeechProviderHandle {
    let environment = use_app_environment();
    SpeechProviderHandle {
        selected: SpeechProvider::Browser,
        native_available: false,
        desktop: environment.is_tauri,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_selection_requires_an_implemented_native_provider() {
        let browser = SpeechProviderHandle {
            selected: SpeechProvider::Browser,
            native_available: false,
            desktop: false,
        };
        let desktop = SpeechProviderHandle {
            desktop: true,
            ..browser
        };
        assert!(!browser.supports_provider_selection());
        assert!(!desktop.supports_provider_selection());
        assert!(
            SpeechProviderHandle {
                native_available: true,
                ..desktop
            }
            .supports_provider_selection()
        );
    }
}
