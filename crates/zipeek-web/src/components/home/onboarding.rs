use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::{
    appearance::{Appearance, Preset},
    hooks::use_app_environment,
    preferences::PreferencesStore,
    services::practice_history::{
        RecordingPreference, notify_recording_preference_changed, recording_preference,
    },
};

#[derive(Properties, PartialEq)]
pub(super) struct Props {
    pub on_enter: Callback<MouseEvent>,
    pub on_back: Callback<MouseEvent>,
}

#[function_component(OnboardingForm)]
pub(super) fn onboarding_form(props: &Props) -> Html {
    let environment = use_app_environment();
    let appearance = use_context::<Appearance>()
        .expect("Appearance provider is missing; mount AppearanceProvider");
    let initial_preset = use_state(|| appearance.preset);
    let original_preset = *initial_preset;
    let selected = use_state(|| original_preset);
    let committed = use_mut_ref(|| false);
    {
        let preview_preset = appearance.preview_preset.clone();
        let committed = committed.clone();
        use_effect_with((), move |_| {
            move || {
                if !*committed.borrow() {
                    preview_preset.emit(original_preset);
                }
            }
        });
    }
    let existing_consent = recording_preference(&environment.local_storage);
    let initially_enabled = matches!(&existing_consent, Ok(RecordingPreference::Enabled));
    let recording_enabled = use_state(move || initially_enabled);
    let error = use_state(|| None::<String>);
    let on_recording_change = {
        let recording_enabled = recording_enabled.clone();
        Callback::from(move |event: Event| {
            recording_enabled.set(
                event
                    .target_unchecked_into::<web_sys::HtmlInputElement>()
                    .checked(),
            );
        })
    };

    let on_save = {
        let storage = environment.local_storage.clone();
        let selected = selected.clone();
        let preview_preset = appearance.preview_preset.clone();
        let committed = committed.clone();
        let recording_enabled = recording_enabled.clone();
        let error = error.clone();
        let on_enter = props.on_enter.clone();
        Callback::from(move |event: MouseEvent| {
            let chosen = *selected;
            let enabled = *recording_enabled;
            let result = PreferencesStore::new(&storage).update(|settings| {
                settings.preset = chosen.key().to_owned();
                settings.learning_history_enabled = enabled;
            });
            match result {
                Ok(()) => {
                    notify_recording_preference_changed();
                    *committed.borrow_mut() = true;
                    preview_preset.emit(chosen);
                    on_enter.emit(event);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    let on_back = {
        let preview_preset = appearance.preview_preset.clone();
        let on_back = props.on_back.clone();
        Callback::from(move |event: MouseEvent| {
            preview_preset.emit(original_preset);
            on_back.emit(event);
        })
    };
    html! {
        <main data-onboarding-form="" class="mx-auto flex min-h-dvh w-full max-w-4xl flex-col justify-center gap-6 px-5 py-10 text-ink sm:px-8">
            <div>
                <p class="text-xs font-bold uppercase tracking-[.18em] text-accent">{"First-time setup"}</p>
                <h1 class="mt-3 text-3xl font-extrabold tracking-tight sm:text-4xl">{"Set up your learning space"}</h1>
                <p class="mt-3 max-w-2xl text-base leading-relaxed text-muted">{"Choose a theme and decide whether completed Practice outcomes should be saved locally. You can change this later in Settings."}</p>
            </div>
            <fieldset class="rounded-2xl border border-line bg-surface/95 p-4 shadow-lg sm:p-6">
                <legend class="px-2 font-semibold">{"Theme"}</legend>
                <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-4">
                    {for Preset::ALL.into_iter().map(|preset| {
                        let chosen = *selected == preset;
                        let selected = selected.clone();
                        let preview_preset = appearance.preview_preset.clone();
                        let onclick = Callback::from(move |_: MouseEvent| {
                            selected.set(preset);
                            preview_preset.emit(preset);
                        });
                        let tile_classes = classes!(
                            "group",
                            "relative",
                            "rounded-xl",
                            "border-2",
                            "p-2.5",
                            "text-left",
                            "shadow-sm",
                            "transition-[border-color,background-color,box-shadow,transform]",
                            "duration-200",
                            "motion-reduce:transition-none",
                            "hover:-translate-y-0.5",
                            "motion-reduce:hover:transform-none",
                            "hover:shadow-md",
                            "focus-visible:outline-2",
                            "focus-visible:outline-accent",
                            if chosen {
                                "border-accent bg-accent-soft"
                            } else {
                                "border-line bg-raised"
                            },
                        );
                        html! {
                            <button key={preset.key()} type="button" data-onboarding-theme={preset.key()}
                                aria-pressed={chosen.to_string()} {onclick}
                                class={tile_classes}>
                                <img src={format!("public/backgrounds/{}", preset.background())} alt="" loading="lazy" class="mb-2 aspect-[2.6/1] w-full rounded-lg object-cover" />
                                <span class="flex items-center justify-between gap-2 text-sm font-semibold">
                                    {preset.name()}
                                    if chosen { <Icon data={IconData::LUCIDE_CHECK} width="17px" height="17px" /> }
                                </span>
                            </button>
                        }
                    })}
                </div>
            </fieldset>
            <fieldset class="rounded-2xl border border-line bg-surface/95 p-4 shadow-lg sm:p-6">
                <legend class="px-2 font-semibold">{"Learning history"}</legend>
                <label class="flex items-start gap-3 rounded-xl p-1">
                    <input type="checkbox" data-onboarding-recording="" checked={*recording_enabled}
                        onchange={on_recording_change} class="mt-1 size-5 shrink-0 accent-[var(--accent)]" />
                    <span class="flex flex-col gap-1">
                        <span class="font-semibold">{"Save completed Practice outcomes on this device"}</span>
                        <span class="text-sm leading-relaxed text-muted">{"Off by default. Enabling this saves completed Practice sessions to your personal database. It does not assess proficiency, unlock Courses, or enable AI tracking. You can disable future recording in Settings → Learning."}</span>
                    </span>
                </label>
            </fieldset>
            {if let Err(message) = &existing_consent {
                html! { <p role="alert" class="text-sm text-red-600">{format!("Existing learning-history preference could not be read: {message}. Recording will remain off until you save a valid choice.")}</p> }
            } else { Html::default() }}
            {if let Some(message) = &*error {
                html! { <p role="alert" class="text-sm text-red-600">{message.clone()}</p> }
            } else { Html::default() }}
            <div class="flex flex-wrap gap-3">
                <button type="button" data-onboarding-back="" onclick={on_back} class="min-h-11 rounded-xl border border-line bg-surface px-6 font-semibold transition-[background-color,border-color] duration-200 hover:border-accent hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent motion-reduce:transition-none">{"Back"}</button>
                <button type="button" data-onboarding-finish="" onclick={on_save} class="min-h-11 rounded-xl bg-brand-action px-7 font-bold text-white shadow-md transition-[background-color,box-shadow,transform] duration-200 hover:-translate-y-0.5 hover:shadow-lg focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent motion-reduce:transform-none motion-reduce:transition-none">{"Start learning"}</button>
            </div>
        </main>
    }
}
