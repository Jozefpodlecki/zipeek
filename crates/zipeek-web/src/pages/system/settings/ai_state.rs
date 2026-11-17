use std::rc::Rc;

use yew::prelude::*;

use crate::services::ai;

#[derive(Clone)]
pub(super) struct AiSettingsState {
    pub url: String,
    pub model: String,
    pub key: String,
    pub message: String,
    pub has_key: bool,
    pub test_busy: bool,
    pub trace: Option<ai::AiConnectionTrace>,
    pub test_ok: Option<bool>,
}

pub(super) enum AiSettingsAction {
    UrlChanged(String),
    ModelChanged(String),
    KeyChanged(String),
    Saved { has_key: bool },
    SaveFailed(String),
    KeyCleared,
    BeginTest,
    TestSucceeded(ai::AiConnectionTrace),
    TestFailed(String),
}

impl AiSettingsState {
    pub(super) fn new(config: ai::AiConfiguration) -> Self {
        Self {
            url: config.url,
            model: config.model,
            key: String::new(),
            message: String::new(),
            has_key: config.key_present,
            test_busy: false,
            trace: None,
            test_ok: None,
        }
    }

    fn clear_diagnostic(&mut self) {
        self.trace = None;
        self.test_ok = None;
        self.test_busy = false;
        self.message.clear();
    }
}

impl Reducible for AiSettingsState {
    type Action = AiSettingsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            AiSettingsAction::UrlChanged(url) => {
                next.clear_diagnostic();
                next.url = url;
            }
            AiSettingsAction::ModelChanged(model) => {
                next.clear_diagnostic();
                next.model = model;
            }
            AiSettingsAction::KeyChanged(key) => {
                next.clear_diagnostic();
                next.key = key;
            }
            AiSettingsAction::Saved { has_key } => {
                next.clear_diagnostic();
                next.has_key = has_key;
                next.key.clear();
                next.message =
                    "AI settings saved. The key stays in memory only for this browser tab."
                        .to_owned();
            }
            AiSettingsAction::SaveFailed(message) => {
                next.clear_diagnostic();
                next.message = message;
            }
            AiSettingsAction::KeyCleared => {
                next.clear_diagnostic();
                next.key.clear();
                next.has_key = false;
                next.message = "API key cleared from this browser tab.".to_owned();
            }
            AiSettingsAction::BeginTest => {
                next.clear_diagnostic();
                next.test_busy = true;
                next.message = "Contacting your configured model…".to_owned();
            }
            AiSettingsAction::TestSucceeded(trace) => {
                next.test_busy = false;
                next.trace = Some(trace);
                next.test_ok = Some(true);
                next.message = "Connection successful · the selected model responded to the diagnostic request.".to_owned();
            }
            AiSettingsAction::TestFailed(message) => {
                next.test_busy = false;
                next.test_ok = Some(false);
                next.message = message;
            }
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> Rc<AiSettingsState> {
        Rc::new(AiSettingsState::new(ai::AiConfiguration {
            url: "http://127.0.0.1:1234/v1/".to_owned(),
            model: "test-model".to_owned(),
            key_present: false,
        }))
    }

    #[test]
    fn editing_model_resets_previous_test_feedback() {
        let working = state().reduce(AiSettingsAction::BeginTest);
        let edited = working.reduce(AiSettingsAction::ModelChanged("other-model".to_owned()));
        assert_eq!(edited.model, "other-model");
        assert!(!edited.test_busy);
        assert!(edited.trace.is_none());
        assert!(edited.test_ok.is_none());
    }

    #[test]
    fn changing_key_cancels_pending_diagnostic() {
        let testing = state().reduce(AiSettingsAction::BeginTest);
        let changed = testing.reduce(AiSettingsAction::KeyChanged("new-key".to_owned()));
        assert_eq!(changed.key, "new-key");
        assert!(!changed.test_busy);
        assert!(changed.test_ok.is_none());
        assert!(changed.trace.is_none());
    }

    #[test]
    fn saving_settings_clears_inflight_diagnostic() {
        let testing = state().reduce(AiSettingsAction::BeginTest);
        let saved = testing.reduce(AiSettingsAction::Saved { has_key: true });
        assert!(!saved.test_busy);
        assert!(saved.test_ok.is_none());
        assert!(saved.has_key);
    }

    #[test]
    fn failed_save_clears_inflight_diagnostic() {
        let testing = state().reduce(AiSettingsAction::BeginTest);
        let failed = testing.reduce(AiSettingsAction::SaveFailed("storage unavailable".into()));
        assert!(!failed.test_busy);
        assert!(failed.test_ok.is_none());
        assert_eq!(failed.message, "storage unavailable");
    }

    #[test]
    fn clearing_key_resets_status_without_erasing_url_and_model() {
        let working = state().reduce(AiSettingsAction::KeyChanged("secret".to_owned()));
        let cleared = working.reduce(AiSettingsAction::KeyCleared);
        assert!(cleared.key.is_empty());
        assert!(!cleared.has_key);
        assert_eq!(cleared.model, "test-model");
    }
}
