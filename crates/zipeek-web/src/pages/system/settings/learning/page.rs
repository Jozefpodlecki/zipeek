use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::{
    diagnostics::LearningDiagnostics, progress::LearningProgress, recording::RecordingControl,
};

#[function_component(LearningSettings)]
pub(crate) fn learning_settings() -> Html {
    html! {
        <div data-settings-learning="" class="space-y-4">
            <header class="flex items-center gap-3">
                <Icon data={IconData::LUCIDE_GRADUATION_CAP} width="22px" />
                <div>
                    <h2 class="text-xl font-semibold">{"Learning"}</h2>
                    <p class="mt-1 text-sm text-muted">{"Learning preferences and personal data controls."}</p>
                </div>
            </header>
            <div class="grid gap-3 lg:grid-cols-2">
                <RecordingControl />
                <LearningProgress />
            </div>
            <super::super::super::personal_storage::PersonalStoragePanel />
            <LearningDiagnostics />
        </div>
    }
}
