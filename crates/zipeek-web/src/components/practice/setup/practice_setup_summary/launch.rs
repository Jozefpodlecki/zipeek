use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::definition::ExternalService;

use super::PracticeSetupSummaryProps;

pub(super) fn render(props: &PracticeSetupSummaryProps) -> Html {
    if props.can_start && !props.start_failed {
        return Html::default();
    }
    let settings = match props.missing_service {
        Some(ExternalService::AiText) => Some(("#/settings?section=ai", "Open AI settings")),
        Some(ExternalService::ImageGeneration) => {
            Some(("#/settings?section=images", "Open image settings"))
        }
        None => None,
    };
    let on_retry_image = props.on_retry_image.clone();
    html! {
        <div class="space-y-3" data-practice-summary-actions="">
            if !props.can_start {
                <div id="practice-start-gate" role="status" data-practice-start-gate="" class="grid w-full grid-cols-[auto_minmax(0,1fr)] items-start gap-3 rounded-xl border border-amber-500/35 bg-amber-100/90 px-3 py-3 text-sm text-amber-950 dark:bg-amber-950/25 dark:text-amber-100">
                    <span class="mt-0.5" aria-hidden="true"><Icon data={IconData::LUCIDE_INFO} width="18px" height="18px" /></span>
                    <span class="min-w-0">{props.gate_message.clone()}</span>
                    if let Some((href, label)) = settings {
                        <div class="col-span-2 flex flex-wrap gap-2 sm:col-span-1 sm:col-start-2">
                        <a {href} class="inline-flex min-h-9 items-center rounded-lg border border-amber-400/40 bg-surface px-3 font-semibold text-ink no-underline hover:bg-raised focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent">
                            {label}
                        </a>
                        if props.missing_service == Some(ExternalService::ImageGeneration) {
                            <button type="button" data-practice-image-retry="" disabled={props.image_checking} onclick={on_retry_image}
                                class="min-h-9 rounded-lg border border-amber-400/40 bg-surface px-3 font-semibold text-ink hover:bg-raised focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:opacity-50">
                                {"Retry image service"}
                            </button>
                        }
                        </div>
                    }
                </div>
            }
            if props.start_failed {
                <p role="alert" class="text-sm text-red-500">{"Could not start practice. Check your selections and retry."}</p>
            }
        </div>
    }
}
