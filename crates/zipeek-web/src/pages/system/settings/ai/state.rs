use std::rc::Rc;

use yew::prelude::*;

use crate::services::ai;

const LOCAL_URL: &str = "http://127.0.0.1:1430/v1/";
const LM_STUDIO_URL: &str = "http://192.168.1.148:1234/v1/";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AiProvider {
    Local,
    LmStudio,
}

#[derive(Clone)]
pub(super) struct AiSettingsState {
    pub url: String,
    pub model: String,
    pub key: String,
    pub message: String,
    pub has_key: bool,
    pub active_test: Option<u64>,
    pub trace: Option<ai::AiConnectionTrace>,
    pub models: Vec<String>,
    pub model_discovery: Option<u64>,
    pub discovery_error: Option<String>,
    pub test_ok: Option<bool>,
}

pub(super) enum AiSettingsAction {
    UrlChanged(String),
    ProviderChanged(AiProvider),
    DiscoverModels { ticket: u64 },
    ModelsDiscovered { ticket: u64, models: Vec<String> },
    ModelDiscoveryFailed { ticket: u64, message: String },
    ModelChanged(String),
    KeyChanged(String),
    Saved {
        has_key: bool,
    },
    SaveFailed(String),
    KeyCleared,
    BeginTest {
        ticket: u64,
    },
    CancelTest,
    TestSucceeded {
        ticket: u64,
        trace: ai::AiConnectionTrace,
    },
    TestFailed {
        ticket: u64,
        message: String,
    },
}

impl AiSettingsState {
    pub(super) fn new(config: ai::AiConfiguration) -> Self {
        Self {
            url: config.url,
            model: config.model,
            key: String::new(),
            message: String::new(),
            has_key: config.key_present,
            active_test: None,
            trace: None,
            models: Vec::new(),
            model_discovery: None,
            discovery_error: None,
            test_ok: None,
        }
    }

    pub(super) fn provider(&self) -> AiProvider {
        if self.url == LOCAL_URL { AiProvider::Local } else { AiProvider::LmStudio }
    }

    pub(super) fn test_busy(&self) -> bool {
        self.active_test.is_some()
    }

    fn clear_diagnostic(&mut self) {
        self.trace = None;
        self.test_ok = None;
        self.active_test = None;
        self.message.clear();
    }
}

impl Reducible for AiSettingsState {
    type Action = AiSettingsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            AiSettingsAction::DiscoverModels { ticket } => {
                next.clear_diagnostic();
                next.model_discovery = Some(ticket);
                next.models.clear();
                next.discovery_error = None;
            }
            AiSettingsAction::ModelsDiscovered { ticket, models } => {
                if next.model_discovery != Some(ticket) { return self; }
                next.model_discovery = None;
                next.discovery_error = if models.is_empty() {
                    Some("No models were returned. Enter a model ID manually.".to_owned())
                } else {
                    None
                };
                if next.model.trim().is_empty() && !models.is_empty() {
                    next.model = models[0].clone();
                }
                next.models = models;
            }
            AiSettingsAction::ModelDiscoveryFailed { ticket, message } => {
                if next.model_discovery != Some(ticket) { return self; }
                next.model_discovery = None;
                next.discovery_error = Some(message);
            }
            AiSettingsAction::ProviderChanged(provider) => {
                next.model_discovery = None;
                next.models.clear();
                next.discovery_error = None;
                next.clear_diagnostic();
                match provider {
                    AiProvider::Local => {
                        next.url = LOCAL_URL.to_owned();
                        next.model = "local-gguf".to_owned();
                        next.key.clear();
                        next.has_key = false;
                    }
                    AiProvider::LmStudio => {
                        next.url = LM_STUDIO_URL.to_owned();
                        next.model.clear();
                    }
                }
            }
            AiSettingsAction::UrlChanged(url) => {
                next.model_discovery = None;
                next.models.clear();
                next.discovery_error = None;
                next.clear_diagnostic();
                next.url = url;
            }
            AiSettingsAction::ModelChanged(model) => {
                next.clear_diagnostic();
                next.model_discovery = None;
                next.discovery_error = None;
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
                next.message = "Provider settings saved.".to_owned();
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
            AiSettingsAction::BeginTest { ticket } => {
                if next.test_busy() {
                    return self;
                }
                next.clear_diagnostic();
                next.active_test = Some(ticket);
                next.message = "Sending a test request to the selected model…".to_owned();
            }
            AiSettingsAction::CancelTest => {
                if !next.test_busy() {
                    return self;
                }
                next.clear_diagnostic();
                next.message = "Diagnostic dismissed. The request may still finish.".to_owned();
            }
            AiSettingsAction::TestSucceeded { ticket, trace } => {
                if next.active_test != Some(ticket) {
                    return self;
                }
                next.active_test = None;
                next.trace = Some(trace);
                next.test_ok = Some(true);
                next.message = "Model responded successfully.".to_owned();
            }
            AiSettingsAction::TestFailed { ticket, message } => {
                if next.active_test != Some(ticket) {
                    return self;
                }
                next.active_test = None;
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
    fn late_failure_after_edit_is_ignored() {
        let working = state().reduce(AiSettingsAction::BeginTest { ticket: 1 });
        let edited = working.reduce(AiSettingsAction::ModelChanged("changed".into()));
        let stale = edited.clone().reduce(AiSettingsAction::TestFailed {
            ticket: 1,
            message: "stale failure".into(),
        });
        assert!(Rc::ptr_eq(&edited, &stale));
    }

    #[test]
    fn editing_model_resets_previous_test_feedback() {
        let working = state().reduce(AiSettingsAction::BeginTest { ticket: 1 });
        let edited = working.reduce(AiSettingsAction::ModelChanged("other-model".to_owned()));
        assert_eq!(edited.model, "other-model");
        assert!(!edited.test_busy());
        assert!(edited.trace.is_none());
        assert!(edited.test_ok.is_none());
    }

    #[test]
    fn changing_key_cancels_pending_diagnostic() {
        let testing = state().reduce(AiSettingsAction::BeginTest { ticket: 1 });
        let changed = testing.reduce(AiSettingsAction::KeyChanged("new-key".to_owned()));
        assert_eq!(changed.key, "new-key");
        assert!(!changed.test_busy());
        assert!(changed.test_ok.is_none());
    }

    #[test]
    fn saving_settings_clears_inflight_diagnostic() {
        let testing = state().reduce(AiSettingsAction::BeginTest { ticket: 1 });
        let saved = testing.reduce(AiSettingsAction::Saved { has_key: true });
        assert!(!saved.test_busy());
        assert!(saved.test_ok.is_none());
        assert!(saved.has_key);
    }

    #[test]
    fn failed_save_clears_inflight_diagnostic() {
        let testing = state().reduce(AiSettingsAction::BeginTest { ticket: 1 });
        let failed = testing.reduce(AiSettingsAction::SaveFailed("storage unavailable".into()));
        assert!(!failed.test_busy());
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

    #[test]
    fn previous_test_result_cannot_finish_new_test() {
        let first = state().reduce(AiSettingsAction::BeginTest { ticket: 1 });
        let cancelled = first.reduce(AiSettingsAction::CancelTest);
        let second = cancelled.reduce(AiSettingsAction::BeginTest { ticket: 2 });
        let stale = second.clone().reduce(AiSettingsAction::TestFailed {
            ticket: 1,
            message: "old".into(),
        });
        assert!(Rc::ptr_eq(&stale, &second));
        assert_eq!(stale.active_test, Some(2));
        let completed = stale.reduce(AiSettingsAction::TestFailed {
            ticket: 2,
            message: "new".into(),
        });
        assert!(!completed.test_busy());
        assert_eq!(completed.message, "new");
    }

    #[test]
    fn concurrent_begin_is_ignored() {
        let first = state().reduce(AiSettingsAction::BeginTest { ticket: 1 });
        let duplicate = first
            .clone()
            .reduce(AiSettingsAction::BeginTest { ticket: 2 });
        assert!(Rc::ptr_eq(&first, &duplicate));
    }

    #[test]
    fn cancelling_inactive_test_has_no_effect() {
        let initial = state();
        let next = initial.clone().reduce(AiSettingsAction::CancelTest);
        assert!(Rc::ptr_eq(&initial, &next));
    }
}

#[cfg(test)]
mod discovery_feedback_tests {
    use super::*;

    fn state() -> Rc<AiSettingsState> {
        Rc::new(AiSettingsState::new(ai::AiConfiguration {
            url: LM_STUDIO_URL.to_owned(),
            model: "test-model".to_owned(),
            key_present: false,
        }))
    }

    #[test]
    fn discovery_error_is_separate_from_inference_status() {
        let discovering = state().reduce(AiSettingsAction::DiscoverModels { ticket: 5 });
        let failed = discovering.reduce(AiSettingsAction::ModelDiscoveryFailed {
            ticket: 5,
            message: "Server did not respond".to_owned(),
        });
        assert_eq!(failed.discovery_error.as_deref(), Some("Server did not respond"));
        assert!(failed.message.is_empty());
        let edited = failed.reduce(AiSettingsAction::UrlChanged("http://localhost:1234/v1".to_owned()));
        assert!(edited.discovery_error.is_none());
    }

    #[test]
    fn stale_discovery_cannot_replace_the_selected_provider() {
        let discovering = state().reduce(AiSettingsAction::DiscoverModels { ticket: 9 });
        let switched = discovering.reduce(AiSettingsAction::ProviderChanged(AiProvider::Local));
        let late = switched.clone().reduce(AiSettingsAction::ModelsDiscovered {
            ticket: 9,
            models: vec!["stale-model".to_owned()],
        });
        assert!(Rc::ptr_eq(&switched, &late));
    }

    #[test]
    fn discovered_models_keep_manual_model_selection() {
        let discovering = state().reduce(AiSettingsAction::DiscoverModels { ticket: 10 });
        let discovered = discovering.reduce(AiSettingsAction::ModelsDiscovered {
            ticket: 10,
            models: vec!["one".to_owned(), "two".to_owned()],
        });
        assert_eq!(discovered.model, "test-model");
        assert_eq!(discovered.models.len(), 2);
        assert!(discovered.discovery_error.is_none());
    }
}

#[cfg(test)]
mod provider_tests {
    use super::*;
    #[test]
    fn choosing_local_sets_native_server_and_model() {
        let initial = Rc::new(AiSettingsState::new(ai::AiConfiguration {
            url: LM_STUDIO_URL.into(), model: "remote".into(), key_present: false,
        }));
        let chosen = initial.reduce(AiSettingsAction::ProviderChanged(AiProvider::Local));
        assert_eq!(chosen.url, LOCAL_URL);
        assert_eq!(chosen.model, "local-gguf");
        assert_eq!(chosen.provider(), AiProvider::Local);
    }
}
