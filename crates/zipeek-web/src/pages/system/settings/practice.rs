use crate::hooks::use_app_environment;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component(PracticeSettings)]
pub(crate) fn practice_settings() -> Html {
    let environment = use_app_environment();
    let audio_storage = environment.local_storage.clone();
    let matching_autoplay =
        use_state(move || crate::services::practice_audio::autoplay_enabled(&audio_storage));
    let on_matching_autoplay = {
        let matching_autoplay = matching_autoplay.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |event: Event| {
            let selected = event.target_unchecked_into::<HtmlInputElement>().checked();
            match crate::services::practice_audio::set_autoplay(&storage, selected) {
                Ok(()) => matching_autoplay.set(selected),
                Err(error) => log::warn!("Could not save Matching audio preference: {error:?}"),
            }
        })
    };
    html! {
                <section
                    data-settings-practice=""
                    class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8"
                >
                    <h2
                        class="text-xl font-semibold"
                    >
                        {"Practice preferences"}
                    </h2>
                    <div
                        class="border-t border-line pt-4"
                        data-settings-matching-audio=""
                    >
                        <h3
                            class="text-sm font-semibold"
                        >
                            {"Matching exercise audio"}
                        </h3>
                        <label
                            class="mt-3 flex items-center gap-3 text-sm"
                        >
                            <input
                                type="checkbox"
                                class="size-4 accent-brand-action"
                                checked={*matching_autoplay}
                                onchange={on_matching_autoplay.clone()}
                            />
                            {"Play pronunciation automatically when selecting a matching card"}
                        </label>
                        <p
                            class="mt-2 text-xs text-muted"
                        >
                            {"Off by default. Audio remains available through the speaker button."}
                        </p>
                    </div>
                </section>
    }
}
