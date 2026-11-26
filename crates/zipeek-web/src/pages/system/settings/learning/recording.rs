use crate::hooks::use_app_environment;
use crate::services::practice_history::{
    RecordingPreference, recording_preference, set_recording_preference,
};
use gloo::events::EventListener;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(RecordingControl)]
pub(super) fn recording_control() -> Html {
    let environment = use_app_environment();
    let recording = {
        let storage = environment.local_storage.clone();
        use_state(move || recording_preference(&storage))
    };
    {
        let window = environment.window.clone();
        let storage = environment.local_storage.clone();
        let recording = recording.clone();
        use_effect_with((), move |_| {
            let local_storage = storage.clone();
            let local_recording = recording.clone();
            let changed = EventListener::new(&window, "zipeek-recording-change", move |_| {
                local_recording.set(recording_preference(&local_storage));
            });
            let storage_recording = recording.clone();
            let storage_changed = EventListener::new(&window, "storage", move |_| {
                storage_recording.set(recording_preference(&storage));
            });
            move || {
                drop(changed);
                drop(storage_changed);
            }
        });
    }
    let on_change = {
        let recording = recording.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |event: Event| {
            let enabled = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .checked();
            match set_recording_preference(&storage, enabled) {
                Ok(()) => recording.set(Ok(if enabled {
                    RecordingPreference::Enabled
                } else {
                    RecordingPreference::Disabled
                })),
                Err(error) => recording.set(Err(error)),
            }
        })
    };
    let enabled = matches!(&*recording, Ok(RecordingPreference::Enabled));
    let available = recording.is_ok();
    let status = if !available {
        "Unavailable"
    } else if enabled {
        "On"
    } else {
        "Off"
    };
    html! {
        <section class="rounded-xl border border-line bg-surface p-4 sm:p-5" aria-labelledby="learning-recording-title" data-learning-recording-summary="">
            <div class="flex items-center gap-2">
                <Icon data={IconData::LUCIDE_DATABASE} width="18px" />
                <h3 id="learning-recording-title" class="font-semibold">{"Practice recording"}</h3>
            </div>
            <label class="mt-4 flex min-h-11 cursor-pointer items-center justify-between gap-3 rounded-lg border border-line bg-raised px-3 py-2">
                <span class="text-sm">{"Save completed Practice outcomes"}</span>
                <span class="flex items-center gap-3">
                    <span class="text-xs text-muted" data-learning-recording-status="">{status}</span>
                    <input type="checkbox" checked={enabled} disabled={!available} onchange={on_change} data-personal-learning-history-toggle="" class="size-4 accent-accent" />
                </span>
            </label>
            <p class="mt-3 text-xs text-muted">{"Off by default. Saved on this device only. Recording is not pedagogical assessment."}</p>
            if let Err(error) = &*recording {
                <p role="alert" class="mt-2 break-words text-sm text-red-500">{error.clone()}</p>
            }
        </section>
    }
}
