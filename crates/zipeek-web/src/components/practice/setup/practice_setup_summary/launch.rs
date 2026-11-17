use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::PracticeSetupSummaryProps;

pub(super) fn render(props: &PracticeSetupSummaryProps) -> Html {
    let on_back = {
        let action = props.on_back.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };
    let on_start = props.on_start.clone();
    html! {
        <footer
            class="flex flex-wrap items-center justify-between gap-3 border-b border-line pb-4"
            data-practice-summary-actions=""
        >
            <button
                type="button"
                data-practice-summary-back=""
                class="min-h-11 rounded-xl border border-line bg-raised px-5 font-semibold"
                onclick={on_back}
            >
                {"Back"}
            </button>
            <button
                type="button"
                data-practice-start=""
                data-practice-local-start={if props.draft.supports_local_vocabulary_session() { "" } else { "inactive" }}
                data-spot-mistake-local-start={if props.draft.has_exercise("spot-mistake") && props.draft.supports_local_vocabulary_session() { "" } else { "inactive" }}
                data-practice-planned-start={if props.plan_rounds.is_some() && props.draft.supports_local_vocabulary_session() { "" } else { "inactive" }}
                data-translation-setup-start={if props.draft.translation_session_only() { "" } else { "inactive" }}
                disabled={!props.can_start}
                onclick={on_start}
                class="flex min-h-11 items-center justify-center gap-3 rounded-xl bg-brand-action px-6 py-2 font-semibold text-white hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-50"
            >
                {"Start"}
                <Icon
                    data={IconData::LUCIDE_PLAY}
                    width="20px"
                    height="20px"
                />
            </button>
            {(!props.can_start).then(|| html! {
                <div role="status" data-practice-start-gate="" class="flex w-full flex-wrap items-center gap-3 rounded-xl border border-amber-500/35 bg-amber-100/90 px-3 py-3 text-sm text-amber-950 dark:bg-amber-950/25 dark:text-amber-100">
                    <Icon data={IconData::LUCIDE_INFO} width="18px" height="18px" />
                    <span class="min-w-0 flex-1">{props.gate_message.clone()}</span>
                    if props.gate_message.contains("AI model") {
                        <a href="#/settings?section=ai" class="inline-flex min-h-9 items-center rounded-lg border border-amber-400/40 bg-surface px-3 font-semibold text-ink no-underline hover:bg-raised focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent">
                            {"Open AI settings"}
                        </a>
                    }
                </div>
            }).unwrap_or_default()}
            {props.start_failed.then(|| html! {
                <p
                    role="alert"
                    class="text-sm text-red-500"
                >
                    {"Could not start practice. Check your selections and retry."}
                </p>
            }).unwrap_or_default()}
        </footer>
    }
}
