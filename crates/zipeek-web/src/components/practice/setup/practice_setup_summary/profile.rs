use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::PracticeSetupSummaryProps;
use crate::services::practice_profiles::PracticeProfile;

pub(super) fn render(props: &PracticeSetupSummaryProps) -> Html {
    let loaded = props.loaded_profile.as_ref();
    let unchanged = loaded.is_some_and(|profile| {
        PracticeProfile::capture(
            &profile.name,
            props.profile_advanced,
            &props.draft,
            props.profile_options,
        )
        .is_ok_and(|current| current == *profile)
    });
    let profile_name_changed =
        loaded.is_some_and(|profile| profile.name != props.profile_name.trim());
    let show_form = !unchanged || profile_name_changed;
    let heading = if loaded.is_some() {
        "Save changes to profile"
    } else {
        "Save this configuration"
    };
    let save_label = if loaded.is_some() && !profile_name_changed {
        "Update profile"
    } else {
        "Save profile"
    };

    html! {
        <div class="border-t border-line pt-4 space-y-3" data-practice-save-profile="">
            {if let Some(profile) = loaded {
                html! {
                    <div class="flex items-center gap-2 text-sm text-muted" data-practice-profile-active="">
                        <Icon data={IconData::LUCIDE_BOOKMARK} width="18px" height="18px" />
                        <span>{format!("Using saved profile: {}", profile.name)}</span>
                    </div>
                }
            } else { Html::default() }}
            {if show_form {
                html! {
                    <>
                        <h3 class="font-semibold">{heading}</h3>
                        <p class="text-sm text-muted">
                            {if loaded.is_some() {
                                "You have unsaved changes. Update this profile or enter a new name to save a copy."
                            } else {
                                "Save the selected exercises, options, vocabulary, and session mode for future Practice sessions."
                            }}
                        </p>
                        <div class="flex flex-wrap items-center gap-3">
                            <label class="sr-only" for="practice-profile-name">{"Profile name"}</label>
                            <input
                                id="practice-profile-name"
                                type="text"
                                maxlength="64"
                                value={props.profile_name.clone()}
                                oninput={props.on_profile_name.clone()}
                                placeholder="Profile name"
                                class="min-h-11 w-full max-w-sm rounded-lg border border-line bg-raised px-3 text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                            />
                            <button
                                type="button"
                                data-practice-profile-save=""
                                onclick={props.on_save_profile.clone()}
                                class="min-h-11 rounded-lg border border-line px-5 font-semibold"
                            >
                                <span class="inline-flex items-center gap-2">
                                    <Icon data={IconData::LUCIDE_SAVE} width="17px" height="17px" />
                                    {save_label}
                                </span>
                            </button>
                        </div>
                    </>
                }
            } else { Html::default() }}
            {if let Some(error) = &props.profile_error {
                html! { <p role="alert" data-practice-profile-error="" class="text-sm text-red-400">{error.clone()}</p> }
            } else { Html::default() }}
            {if let Some(notice) = props.profile_saved_notice.as_ref().filter(|_| unchanged) {
                html! {
                    <p role="status" data-practice-profile-saved="" class="flex items-center gap-2 text-sm text-green-400">
                        <Icon data={IconData::LUCIDE_CHECK} width="18px" height="18px" />
                        {notice.clone()}
                    </p>
                }
            } else { Html::default() }}
        </div>
    }
}
