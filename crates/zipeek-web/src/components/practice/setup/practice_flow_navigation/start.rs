use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::practice_draft::PracticeDraft;

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    pub draft: PracticeDraft,
    pub can_start: bool,
    pub on_start: Callback<MouseEvent>,
}

#[function_component(PracticeStartButton)]
pub(crate) fn practice_start_button(props: &Props) -> Html {
    let supported = props.draft.supports_local_vocabulary_session();
    let on_start = props.on_start.clone();
    html! {
        <button type="button" data-practice-start=""
            data-practice-local-start={if supported { "" } else { "inactive" }}
            data-spot-mistake-local-start={if props.draft.has_exercise("spot-mistake") && supported { "" } else { "inactive" }}
            data-practice-planned-start={if props.draft.length.resumable_rounds().is_some() && supported { "" } else { "inactive" }}
            data-translation-setup-start={if props.draft.translation_session_only() { "" } else { "inactive" }}
            aria-label="Start practice" title="Start practice"
            aria-describedby={(!props.can_start).then_some("practice-start-gate")}
            disabled={!props.can_start} onclick={on_start}
            class="flex size-11 shrink-0 items-center justify-center rounded-xl bg-brand-action text-white hover:bg-brand-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40">
            <span aria-hidden="true"><Icon data={IconData::LUCIDE_PLAY} width="20px" height="20px" /></span>
        </button>
    }
}
