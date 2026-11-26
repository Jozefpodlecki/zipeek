use crate::hooks::use_app_environment;
use yew::prelude::*;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use zipeek_browser::local_speech;

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
        self.native_available
    }
}

#[hook]
fn use_local_speech_capabilities() -> UseStateHandle<Option<local_speech::LocalSpeechCapabilities>> {
    let status = use_state(|| None);
    {
        let status = status.clone();
        use_effect_with(local_speech::enabled(), move |enabled| {
            let alive = Rc::new(Cell::new(true));
            status.set(None);
            if *enabled {
                let pending = alive.clone();
                let result_state = status.clone();
                spawn_local(async move {
                    let result = local_speech::service_capabilities().await.ok();
                    if pending.get() { result_state.set(result); }
                });
            }
            move || alive.set(false)
        });
    }
    status
}

#[hook]
pub fn use_tts_service() -> SpeechProviderHandle {
    let environment = use_app_environment();
    let status = use_local_speech_capabilities();
    let local_selected = local_speech::enabled();
    SpeechProviderHandle {
        selected: if local_selected { SpeechProvider::Native } else { SpeechProvider::Browser },
        native_available: status.as_ref().is_some_and(|state| state.synthesis),
        desktop: environment.is_tauri,
    }
}

#[hook]
pub fn use_stt_service() -> SpeechProviderHandle {
    let environment = use_app_environment();
    let status = use_local_speech_capabilities();
    let local_selected = local_speech::enabled();
    SpeechProviderHandle {
        selected: if local_selected { SpeechProvider::Native } else { SpeechProvider::Browser },
        native_available: status.as_ref().is_some_and(|state| state.recognition),
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
