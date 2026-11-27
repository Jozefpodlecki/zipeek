use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(LearningProgress)]
pub(super) fn learning_progress() -> Html {
    html! {
        <section class="rounded-xl border border-line bg-surface p-4 sm:p-5" aria-labelledby="learning-progress-title" data-learning-evidence-summary="">
            <div class="flex items-center gap-2">
                <Icon data={IconData::LUCIDE_SHIELD_CHECK} width="18px" />
                <h3 id="learning-progress-title" class="font-semibold">{"Pedagogical assessment"}</h3>
            </div>
            <p class="mt-3 text-sm text-muted">{"Not available yet"}</p>
        </section>
    }
}
