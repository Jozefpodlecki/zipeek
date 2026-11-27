use super::{controller::use_ai_settings_controller, state::AiProvider};
use crate::hooks::use_app_environment;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(AiTranslationSettings)]
pub(crate) fn ai_translation_settings() -> Html {
    let controller = use_ai_settings_controller();
    let environment = use_app_environment();
    let state = &controller.state;
    let local = state.provider() == AiProvider::Local && environment.is_tauri;
    let provider_card = "flex min-h-24 flex-col items-start gap-2 rounded-xl border p-4 text-left transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";
    let field = "min-h-11 w-full rounded-lg border border-line bg-raised px-3 text-sm font-normal text-ink focus-visible:outline-2 focus-visible:outline-accent";
    let action = "inline-flex min-h-10 items-center justify-center gap-2 rounded-lg border border-line px-4 text-sm font-medium transition-colors hover:bg-raised disabled:cursor-not-allowed disabled:opacity-50";
    let busy = state.test_busy() || state.model_discovery.is_some();
    let discovered_model = state.models.iter().any(|name| name == &state.model);

    html! {
        <section data-settings-ai="" class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8">
            <div class="flex flex-wrap items-start justify-between gap-2">
                <div>
                    <h2 class="text-xl font-semibold">{"AI translation"}</h2>
                    <p class="mt-1 text-sm text-muted">{"Choose where language generation runs."}</p>
                </div>
            </div>
            <div class="mt-6 grid gap-3 sm:grid-cols-2" role="group" aria-label="AI provider">
                if environment.is_tauri {
                    <button type="button" onclick={controller.on_local.clone()} data-ai-provider="local"
                        aria-pressed={local.to_string()}
                        class={classes!(provider_card, if local {"border-brand-action bg-brand-action/10 text-ink"} else {"border-line bg-raised/50 text-ink hover:border-brand-action/50 hover:bg-raised"})}>
                        <Icon data={IconData::LUCIDE_CPU} width="20px" height="20px" />
                        <span class="font-semibold">{"Zipeek Local AI"}</span>
                        <span class="text-xs font-normal text-muted">{"GGUF inference on this device"}</span>
                    </button>
                }
                <button type="button" onclick={controller.on_lmstudio.clone()} data-ai-provider="lmstudio"
                    aria-pressed={(!local).to_string()}
                    class={classes!(provider_card, if !local {"border-brand-action bg-brand-action/10 text-ink"} else {"border-line bg-raised/50 text-ink hover:border-brand-action/50 hover:bg-raised"})}>
                    <Icon data={IconData::LUCIDE_SERVER} width="20px" height="20px" />
                    <span class="font-semibold">{"LM Studio / OpenAI API"}</span>
                    <span class="text-xs font-normal text-muted">{"Connect to a model server"}</span>
                </button>
            </div>
            <div class="mt-5 min-h-72 space-y-4">
                if local {
                    <div class="rounded-xl border border-line bg-raised/50 p-4">
                        <div class="flex items-center gap-3">
                            <Icon data={IconData::LUCIDE_FILE_BOX} width="20px" height="20px" />
                            <div>
                                <p class="text-sm font-semibold">{"Local GGUF model"}</p>
                                <p class="text-xs text-muted">{"models/ai or ZIPEEK_AI_MODEL_PATH"}</p>
                            </div>
                        </div>
                        <p class="mt-3 text-sm text-muted">{"Place a GGUF model in models/ai, or configure its absolute path before starting the AI service. Manage the process in Settings > Services."}</p>
                        <div class="mt-4 flex flex-wrap items-center justify-between gap-2 rounded-lg border border-line bg-surface px-3 py-2">
                            <div>
                                <p class="text-xs text-muted">{"Endpoint"}</p>
                                <p class="font-mono text-sm">{"http://127.0.0.1:1430/v1"}</p>
                            </div>
                            <span class="text-xs text-muted">{"local-gguf"}</span>
                        </div>
                    </div>
                } else {
                    <label class="flex flex-col gap-2 text-sm font-semibold">
                        {"Server URL"}
                        <input data-ai-base-url="" type="url" value={state.url.clone()}
                            oninput={controller.on_url.clone()} placeholder="http://192.168.1.148:1234/v1/"
                            class={field} />
                    </label>
                    <div class="space-y-2">
                        <div class="flex items-center justify-between gap-2">
                            <span class="text-sm font-semibold">{"Model"}</span>
                            <button type="button" onclick={controller.on_discover.clone()}
                                disabled={busy || state.url.trim().is_empty()}
                                aria-label="Discover available models"
                                class="inline-flex items-center gap-2 rounded-lg px-2 py-1 text-xs text-muted hover:bg-raised hover:text-ink disabled:opacity-50">
                                <Icon data={IconData::LUCIDE_REFRESH_CW} width="15px" height="15px"
                                    class={if state.model_discovery.is_some() {"animate-spin motion-reduce:animate-none"} else {""}} />
                                {if state.model_discovery.is_some() {"Discovering"} else {"Discover models"}}
                            </button>
                        </div>
                        if !state.models.is_empty() {
                            <select value={if discovered_model {state.model.clone()} else {String::new()}}
                                onchange={controller.on_model_selected.clone()} aria-label="Available models" class={field}>
                                <option value="">{"Select a model"}</option>
                                {for state.models.iter().map(|name| html! {<option value={name.clone()}>{name}</option>})}
                            </select>
                            <details class="text-sm">
                                <summary class="cursor-pointer text-xs text-muted hover:text-ink">{"Enter a model ID manually"}</summary>
                                <input data-ai-model-setting="" value={state.model.clone()}
                                    oninput={controller.on_model.clone()} aria-label="Manual model ID"
                                    placeholder="Model ID" class={classes!(field, "mt-2")} />
                            </details>
                        } else {
                            <input data-ai-model-setting="" value={state.model.clone()}
                                oninput={controller.on_model.clone()} aria-label="Model ID"
                                placeholder="Choose a model or enter its ID" class={field} />
                        }
                        if let Some(message) = &state.discovery_error {
                            <p role="alert" class="flex items-start gap-2 text-xs text-amber-600 dark:text-amber-400">
                                <Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="15px" height="15px" />
                                <span>{message}</span>
                            </p>
                        }
                    </div>
                    <details class="rounded-lg border border-line px-3 py-2">
                        <summary class="cursor-pointer text-sm font-medium">{"Authentication (optional)"}</summary>
                        <div class="mt-3 space-y-2">
                            <label class="flex flex-col gap-2 text-xs font-semibold">
                                {"API key"}
                                <input data-ai-api-key="" type="password" autocomplete="off"
                                    value={state.key.clone()} oninput={controller.on_key.clone()}
                                    placeholder={if state.has_key {"Key already set"} else {"Leave blank if not required"}}
                                    class={field} />
                            </label>
                            if state.has_key {
                                <button type="button" onclick={controller.on_clear_key.clone()}
                                    class="text-xs text-muted underline-offset-2 hover:text-ink hover:underline">{"Clear saved key"}</button>
                            }
                            <p class="text-xs text-muted">{"Keys stay in this browser tab. LAN HTTP requests are unencrypted, and browser access requires an allowed origin."}</p>
                        </div>
                    </details>
                }
            </div>
            <div class="mt-5 border-t border-line pt-4">
                <div class="flex flex-wrap items-center justify-end gap-2">
                    <button type="button" onclick={controller.on_test.clone()} data-ai-test-connection=""
                        disabled={busy || state.model.trim().is_empty()} class={action}>
                        if state.test_busy() {
                            <Icon data={IconData::LUCIDE_REFRESH_CW} width="16px" height="16px" class="animate-spin motion-reduce:animate-none" />
                            {"Testing model"}
                        } else {
                            <Icon data={IconData::LUCIDE_PLUG} width="16px" height="16px" />
                            {"Test"}
                        }
                    </button>
                    if state.test_busy() {
                        <button type="button" onclick={controller.on_cancel_test.clone()}
                            class="min-h-10 rounded-lg px-3 text-sm text-muted hover:bg-raised hover:text-ink">
                            {"Dismiss"}
                        </button>
                    }
                    <button type="button" onclick={controller.on_save.clone()} data-ai-save=""
                        disabled={busy || state.model.trim().is_empty()}
                        class="inline-flex min-h-10 items-center justify-center rounded-lg border border-brand-action bg-brand-action px-5 text-sm font-semibold text-white hover:bg-brand-hover disabled:opacity-50">
                        {"Save"}
                    </button>
                </div>
                {super::diagnostics::status(state)}
                {super::diagnostics::trace(state)}
            </div>
        </section>
    }
}
