use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::state::RecognitionState;

#[derive(Properties, PartialEq)]
pub(super) struct RecognitionControlsProps {
    pub state: RecognitionState,
    pub model_configured: bool,
    pub on_recognize: Callback<MouseEvent>,
}

#[function_component(RecognitionControls)]
pub(super) fn recognition_controls(props: &RecognitionControlsProps) -> Html {
    let state = &props.state;
    html! {
        <div class="flex w-full flex-col gap-2 sm:w-auto sm:min-w-56" aria-label="Recognition controls">
            <button type="button" onclick={props.on_recognize.clone()} disabled={state.busy || !props.model_configured || !state.input.has_input()} class="flex min-h-12 w-full items-center justify-center gap-2 rounded-xl bg-brand-action px-4 py-3 text-sm font-semibold text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action disabled:cursor-not-allowed disabled:opacity-50" data-recognition-submit="" aria-label={if state.busy {"Recognizing handwriting"} else {"Recognize handwriting with AI"}} aria-busy={state.busy.to_string()}>
                if state.busy {
                    <>
                        <Icon data={IconData::LUCIDE_REFRESH_CW} width="19px" height="19px" class="animate-spin motion-reduce:animate-none" />
                        <span>{"Recognizing"}</span>
                    </>
                } else {
                    <>
                        <Icon data={IconData::LUCIDE_SCAN_LINE} width="19px" height="19px" />
                        <span>{"Recognize"}</span>
                    </>
                }
            </button>
            if !props.model_configured {
                <div role="alert" class="rounded-lg border border-line bg-raised p-3 text-sm text-ink">
                    {"Choose an image-capable AI model in Settings."}
                    <a href="#/settings" class="mt-2 block font-semibold text-brand-action underline underline-offset-2">{"Open Settings"}</a>
                </div>
            }
            if let Some(error) = &state.error {
                <div role="alert" data-recognition-error="" class="rounded-lg border border-line bg-raised p-3 text-sm text-ink">
                    <p class="break-words">{error.clone()}</p>
                </div>
            }
        </div>
    }
}
