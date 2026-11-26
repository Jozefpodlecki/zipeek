use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::state::{AiProvider, AiSettingsAction, AiSettingsState};
use crate::services::ai;

#[derive(Default)]
struct RequestGate {
    generation: u64,
    active: Option<u64>,
}

impl RequestGate {
    fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.active = None;
    }

    fn start(&mut self) -> Option<u64> {
        if self.active.is_some() {
            return None;
        }
        self.generation = self.generation.wrapping_add(1);
        self.active = Some(self.generation);
        self.active
    }

    fn finish(&mut self, ticket: u64) -> bool {
        if self.active != Some(ticket) {
            return false;
        }
        self.active = None;
        true
    }
}

pub(super) struct AiSettingsController {
    pub state: UseReducerHandle<AiSettingsState>,
    pub on_url: Callback<InputEvent>,
    pub on_local: Callback<MouseEvent>,
    pub on_lmstudio: Callback<MouseEvent>,
    pub on_model: Callback<InputEvent>,
    pub on_discover: Callback<MouseEvent>,
    pub on_model_selected: Callback<Event>,
    pub on_key: Callback<InputEvent>,
    pub on_save: Callback<MouseEvent>,
    pub on_clear_key: Callback<MouseEvent>,
    pub on_test: Callback<MouseEvent>,
    pub on_cancel_test: Callback<MouseEvent>,
}

#[hook]
pub(super) fn use_ai_settings_controller() -> AiSettingsController {
    let state = use_reducer(|| AiSettingsState::new(ai::configuration()));
    let request_gate = use_mut_ref(RequestGate::default);

    let on_local = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |_: MouseEvent| {
            gate.borrow_mut().cancel();
            ai::clear_key();
            state.dispatch(AiSettingsAction::ProviderChanged(AiProvider::Local));
        })
    };
    let on_lmstudio = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |_: MouseEvent| {
            gate.borrow_mut().cancel();
            state.dispatch(AiSettingsAction::ProviderChanged(AiProvider::LmStudio));
        })
    };
    let on_url = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |event: InputEvent| {
            gate.borrow_mut().cancel();
            state.dispatch(AiSettingsAction::UrlChanged(
                event.target_unchecked_into::<HtmlInputElement>().value(),
            ));
        })
    };
    let on_model = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |event: InputEvent| {
            gate.borrow_mut().cancel();
            state.dispatch(AiSettingsAction::ModelChanged(
                event.target_unchecked_into::<HtmlInputElement>().value(),
            ));
        })
    };
    let on_model_selected = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |event: Event| {
            gate.borrow_mut().cancel();
            state.dispatch(AiSettingsAction::ModelChanged(
                event.target_unchecked_into::<web_sys::HtmlSelectElement>().value(),
            ));
        })
    };
    let on_discover = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |_: MouseEvent| {
            let url = state.url.clone();
            let Some(ticket) = gate.borrow_mut().start() else { return; };
            state.dispatch(AiSettingsAction::DiscoverModels { ticket });
            let state = state.clone();
            let gate = gate.clone();
            spawn_local(async move {
                let result = ai::discover_models(&url).await;
                if !gate.borrow_mut().finish(ticket) { return; }
                match result {
                    Ok(models) => state.dispatch(AiSettingsAction::ModelsDiscovered { ticket, models }),
                    Err(message) => state.dispatch(AiSettingsAction::ModelDiscoveryFailed { ticket, message }),
                }
            });
        })
    };
    let on_key = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |event: InputEvent| {
            gate.borrow_mut().cancel();
            state.dispatch(AiSettingsAction::KeyChanged(
                event.target_unchecked_into::<HtmlInputElement>().value(),
            ));
        })
    };
    let on_save = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |_: MouseEvent| {
            gate.borrow_mut().cancel();
            if state.provider() == AiProvider::Local { ai::clear_key(); }
            match ai::save_configuration(&state.url, &state.model, &state.key) {
                Ok(()) => {
                    crate::contexts::service_management::notify_configuration_changed();
                    state.dispatch(AiSettingsAction::Saved {
                        has_key: state.provider() != AiProvider::Local && (state.has_key || !state.key.is_empty()),
                    });
                }
                Err(message) => state.dispatch(AiSettingsAction::SaveFailed(message)),
            }
        })
    };
    let on_clear_key = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |_: MouseEvent| {
            gate.borrow_mut().cancel();
            ai::clear_key();
            state.dispatch(AiSettingsAction::KeyCleared);
        })
    };
    let on_test = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |_: MouseEvent| {
            if state.test_busy() || state.model.trim().is_empty() {
                return;
            }
            let url = state.url.clone();
            let model = state.model.clone();
            let Some(ticket) = gate.borrow_mut().start() else {
                return;
            };
            state.dispatch(AiSettingsAction::BeginTest { ticket });
            let state = state.clone();
            let gate = gate.clone();
            spawn_local(async move {
                let response = ai::test_connection_details(&url, &model).await;
                if !gate.borrow_mut().finish(ticket) {
                    return;
                }
                match response {
                    Ok(trace) => state.dispatch(AiSettingsAction::TestSucceeded { ticket, trace }),
                    Err(message) => {
                        state.dispatch(AiSettingsAction::TestFailed { ticket, message })
                    }
                }
            });
        })
    };
    let on_cancel_test = {
        let state = state.clone();
        let gate = request_gate.clone();
        Callback::from(move |_: MouseEvent| {
            gate.borrow_mut().cancel();
            state.dispatch(AiSettingsAction::CancelTest);
        })
    };
    {
        let gate = request_gate.clone();
        use_effect_with((), move |_| {
            move || {
                gate.borrow_mut().cancel();
            }
        });
    }
    AiSettingsController {
        state,
        on_url,
        on_local,
        on_lmstudio,
        on_model,
        on_discover,
        on_model_selected,
        on_key,
        on_save,
        on_clear_key,
        on_test,
        on_cancel_test,
    }
}

#[cfg(test)]
mod tests {
    use super::RequestGate;

    #[test]
    fn duplicate_request_cannot_replace_active_ticket() {
        let mut gate = RequestGate::default();
        let first = gate.start().expect("first request");
        assert_eq!(gate.start(), None);
        assert!(gate.finish(first));
    }

    #[test]
    fn cancelled_request_cannot_publish() {
        let mut gate = RequestGate::default();
        let first = gate.start().expect("first request");
        gate.cancel();
        let second = gate.start().expect("new request");
        assert!(!gate.finish(first));
        assert!(gate.finish(second));
    }
}
