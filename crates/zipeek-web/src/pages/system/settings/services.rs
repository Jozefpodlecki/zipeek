use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::hooks::{use_ai_service, use_app_environment, use_image_service, use_speech_service};

use super::native_services::NativeServicePanel;

#[function_component(DesktopServicesSettings)]
pub(super) fn desktop_services_settings() -> Html {
    let environment = use_app_environment();
    let ai = use_ai_service();
    let image = use_image_service();
    let speech = use_speech_service();

    if !environment.is_tauri {
        return Html::default();
    }

    html! {
        <section data-component="desktop-services-settings" class="space-y-4" aria-labelledby="desktop-services-heading">
            <header class="flex flex-wrap items-center gap-3 rounded-2xl border border-line bg-surface p-5">
                <div class="flex items-center gap-2">
                    <Icon data={IconData::LUCIDE_SERVER} width="20px" height="20px" />
                    <h2 id="desktop-services-heading" class="text-xl font-semibold">{"Local AI services"}</h2>
                </div>
                <p class="w-full text-sm text-muted">{"Start, stop and check the health of each local service."}</p>
            </header>
            <div class="space-y-3" data-managed-service-list="">
                <NativeServicePanel service={ai} />
                <NativeServicePanel service={image} />
                <NativeServicePanel service={speech} />
            </div>
        </section>
    }
}
