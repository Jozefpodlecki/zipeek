mod controls;
mod insights;
mod results;
mod state;

use crate::hooks::use_dictionary;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlTextAreaElement, KeyboardEvent};
use yew::prelude::*;

use crate::{components::AiStreamProgress, hooks::use_app_environment, services::ai};
use controls::TranslatorControls;
use insights::TranslatorInsights;
use results::TranslationResults;
use state::{TranslationDirection, TranslatorAction, TranslatorState};

fn actionable_translation_error(error: &str) -> String {
    let normalized = error.to_ascii_lowercase();
    if [
        "failed to fetch",
        "connection refused",
        "networkerror",
        "network request failed",
    ]
    .iter()
    .any(|indicator| normalized.contains(indicator))
    {
        "Cannot reach the configured AI service. Check its address and connection in Settings, ensure the service is running, and try again.".to_owned()
    } else {
        error.to_owned()
    }
}

#[cfg(test)]
mod translation_error_tests {
    use super::actionable_translation_error;

    #[test]
    fn explains_unreachable_ai_service() {
        let message = actionable_translation_error("NetworkError: Failed to fetch");
        assert!(message.contains("configured AI service"));
        assert!(message.contains("Settings"));
    }

    #[test]
    fn preserves_non_network_errors() {
        assert_eq!(
            actionable_translation_error("Invalid translation response"),
            "Invalid translation response"
        );
    }
}

#[function_component(SentenceTranslatorPage)]
pub fn sentence_translator_page() -> Html {
    let environment = use_app_environment();
    let dictionary = use_dictionary();
    let state = use_reducer(TranslatorState::default);
    let generation = use_mut_ref(|| 0_u64);
    let configured = ai::configuration();

    let on_source = {
        let state = state.clone();
        let generation = generation.clone();
        Callback::from(move |event: InputEvent| {
            *generation.borrow_mut() += 1;
            state.dispatch(TranslatorAction::EditSource(
                event.target_unchecked_into::<HtmlTextAreaElement>().value(),
            ));
        })
    };

    let on_direction = {
        let state = state.clone();
        let generation = generation.clone();
        Callback::from(move |direction: TranslationDirection| {
            *generation.borrow_mut() += 1;
            state.dispatch(TranslatorAction::SetDirection(direction));
        })
    };

    let on_translate = {
        let dictionary = dictionary.clone();
        let state = state.clone();
        let generation = generation.clone();
        Callback::from(move |_: ()| {
            if state.busy || state.source.trim().is_empty() {
                return;
            }
            let input = state.source.clone();
            let direction = state.direction.code().to_owned();
            let ticket = {
                let mut current = generation.borrow_mut();
                *current += 1;
                *current
            };
            state.dispatch(TranslatorAction::BeginRequest);

            let state = state.clone();
            let generation = generation.clone();
            let dictionary = dictionary.clone();
            spawn_local(async move {
                let response = ai::translate_and_analyze_with_progress(
                    &input,
                    &direction,
                    |phase, received_chars| {
                        if *generation.borrow() == ticket {
                            state.dispatch(TranslatorAction::Progress {
                                phase: phase.to_owned(),
                                received_chars,
                            });
                        }
                    },
                )
                .await;
                if *generation.borrow() != ticket {
                    return;
                }
                match response {
                    Ok(outcome) => {
                        let ai::TranslationOutcome {
                            text,
                            analysis,
                            warning,
                        } = outcome;
                        let mut verified = None;
                        if let Some(ref details) = analysis {
                            let result = async {
                                let database =
                                    dictionary.database().map_err(|error| error.to_string())?;
                                database
                                    .verify_sentence_vocabulary(&details.vocabulary)
                                    .await
                                    .map_err(|error| error.to_string())
                            }
                            .await;
                            if *generation.borrow() != ticket {
                                return;
                            }
                            verified = Some(result);
                        }
                        state.dispatch(TranslatorAction::Resolve {
                            text,
                            analysis,
                            verified,
                            warning,
                        });
                    }
                    Err(error) => state.dispatch(TranslatorAction::Fail(error)),
                }
            });
        })
    };

    let on_keydown = {
        let on_translate = on_translate.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() == "Enter"
                && (event.ctrl_key() || event.meta_key())
                && !event.is_composing()
            {
                event.prevent_default();
                on_translate.emit(());
            }
        })
    };

    let on_clear = {
        let state = state.clone();
        let generation = generation.clone();
        Callback::from(move |_: MouseEvent| {
            *generation.borrow_mut() += 1;
            state.dispatch(TranslatorAction::Clear);
        })
    };

    let on_copy = {
        let navigator = environment.navigator.clone();
        let state = state.clone();
        let generation = generation.clone();
        Callback::from(move |text: String| {
            let ticket = *generation.borrow();
            let clipboard = navigator.clipboard();
            let state = state.clone();
            let generation = generation.clone();
            state.dispatch(TranslatorAction::CopyStatus("Copying…".to_owned()));
            spawn_local(async move {
                let message = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&text))
                    .await
                    .map(|_| "Copied translation to clipboard.")
                    .unwrap_or("Clipboard is unavailable or permission denied. Select and copy the translation manually.");
                if *generation.borrow() == ticket {
                    state.dispatch(TranslatorAction::CopyStatus(message.to_owned()));
                }
            });
        })
    };

    {
        let generation = generation.clone();
        use_effect_with((), move |_| move || *generation.borrow_mut() += 1);
    }

    let translation_analysis_content = {
        if let Some(analysis) = &state.analysis {
            html! {
                <TranslatorInsights
                    direction={state.direction.clone()}
                    source={state.source.clone()}
                    analysis={analysis.clone()}
                    verified={state.verified.clone()}
                />
            }
        } else if let Some(warning) = &state.analysis_warning {
            html! {
                <p
                    role="status"
                    class="rounded-xl border border-line bg-surface p-4 text-sm text-muted"
                >
                    {warning.clone()}
                </p>
            }
        } else {
            Html::default()
        }
    };
    let translation_error_message = {
        if state.error.is_empty() {
            Html::default()
        } else {
            html! {
                <p
                    role="alert"
                    class="rounded-xl border border-line bg-surface p-4 text-sm"
                >
                    {actionable_translation_error(&state.error)}
                </p>
            }
        }
    };

    let translation_progress_indicator = {
        if state.busy {
            html! {
                <AiStreamProgress
                    phase={state.progress_phase.clone()}
                    received_chars={state.progress_chars}
                />
            }
        } else {
            Html::default()
        }
    };
    html! {
        <section
            data-component="translate-sentence-page"
            class="mx-auto flex w-full max-w-6xl min-w-0 flex-col gap-4 px-3 py-5 sm:px-5"
        >
            <TranslatorControls
                direction={state.direction.clone()}
                source={state.source.clone()}
                busy={state.busy}
                model_ready={!configured.model.trim().is_empty()}
                on_direction={on_direction}
                on_source={on_source}
                on_keydown={on_keydown}
                on_clear={on_clear}
                on_submit={on_translate.reform(|_: MouseEvent| ())}
            />
            {translation_error_message}
            {translation_progress_indicator}
            <TranslationResults
                direction={state.direction.clone()}
                result={state.result.clone()}
                analysis={state.analysis.clone()}
                busy={state.busy}
                copy_status={state.copy_status.clone()}
                on_copy={on_copy}
            />
            {translation_analysis_content}
        </section>
    }
}
