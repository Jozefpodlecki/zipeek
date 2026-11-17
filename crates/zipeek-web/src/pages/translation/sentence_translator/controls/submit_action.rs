use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct TranslationSubmitActionProps {
    pub busy: bool,
    pub model_ready: bool,
    pub has_source: bool,
    pub on_submit: Callback<MouseEvent>,
}

#[function_component(TranslationSubmitAction)]
pub(super) fn translation_submit_action(props: &TranslationSubmitActionProps) -> Html {
    html! {
        <div
            class="mt-3 flex flex-wrap items-center justify-between gap-3"
        >
            <span
                class="text-xs text-muted"
            >
                {"Ctrl+Enter or ⌘+Enter to translate"}
            </span>
            <button
                data-translate-submit=""
                type="button"
                disabled={props.busy || !props.model_ready || !props.has_source}
                onclick={props.on_submit.clone()}
                class="inline-flex min-h-11 items-center justify-center gap-2 rounded-lg bg-brand-action px-6 font-semibold text-white hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-50"
            >
                <Icon
                    data={IconData::LUCIDE_LANGUAGES}
                    width="19px"
                    height="19px"
                />
                {if props.busy { "Analysing…" } else { "Translate & Analyse" }}
            </button>
        </div>
    }
}
