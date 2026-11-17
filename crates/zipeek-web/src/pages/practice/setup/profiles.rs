use crate::{
    reducers::practice_setup_flow::{PracticeSetupAction, PracticeSetupFlow},
    services::practice_profiles::{self, PracticeProfile},
};
use yew::prelude::*;

pub(super) struct ProfileControls {
    pub profiles: UseStateHandle<Result<Vec<PracticeProfile>, String>>,
    pub name: UseStateHandle<String>,
    pub error: UseStateHandle<Option<String>>,
    pub loaded: UseStateHandle<Option<PracticeProfile>>,
    pub saved_notice: UseStateHandle<Option<String>>,
    pub pending_delete: UseStateHandle<Option<String>>,
    pub on_name_change: Callback<InputEvent>,
    pub on_save: Callback<MouseEvent>,
    pub on_load: Callback<String>,
    pub on_request_delete: Callback<String>,
    pub on_confirm_delete: Callback<MouseEvent>,
    pub on_cancel_delete: Callback<MouseEvent>,
}

#[hook]
pub(super) fn use_profile_controls(
    storage: web_sys::Storage,
    flow: UseReducerHandle<PracticeSetupFlow>,
) -> ProfileControls {
    let profiles = {
        let storage = storage.clone();
        use_state(move || practice_profiles::read(&storage))
    };
    let name = use_state(String::new);
    let error = use_state(|| None::<String>);
    let loaded = use_state(|| None::<PracticeProfile>);
    let saved_notice = use_state(|| None::<String>);
    let pending_delete = use_state(|| None::<String>);
    let on_name_change = {
        let name = name.clone();
        Callback::from(move |event: InputEvent| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            name.set(input.value());
        })
    };
    let on_save = {
        let storage = storage.clone();
        let profiles = profiles.clone();
        let name = name.clone();
        let error = error.clone();
        let loaded = loaded.clone();
        let saved_notice = saved_notice.clone();
        let draft = flow.draft.clone();
        let advanced = flow.advanced;
        let options = flow.exercise_options;
        Callback::from(move |_: MouseEvent| {
            let result = PracticeProfile::capture(name.as_str(), advanced, &draft, options)
                .and_then(|profile| {
                    practice_profiles::save(&storage, profile.clone())?;
                    Ok(profile)
                });
            match result {
                Ok(profile) => {
                    let saved_name = profile.name.clone();
                    profiles.set(practice_profiles::read(&storage));
                    loaded.set(Some(profile));
                    name.set(saved_name.clone());
                    error.set(None);
                    saved_notice.set(Some(format!("Profile ‘{saved_name}’ saved.")));
                }
                Err(message) => {
                    saved_notice.set(None);
                    error.set(Some(message));
                }
            }
        })
    };
    let on_load = {
        let flow = flow.clone();
        let profiles = profiles.clone();
        let error = error.clone();
        let name = name.clone();
        let loaded = loaded.clone();
        let saved_notice = saved_notice.clone();
        Callback::from(move |selected: String| {
            let result = match &*profiles {
                Ok(saved) => saved
                    .iter()
                    .find(|profile| profile.name == selected)
                    .ok_or_else(|| "Saved profile was not found".to_owned())
                    .and_then(PracticeProfile::restore),
                Err(message) => Err(message.clone()),
            };
            match result {
                Ok((draft, options, advanced)) => {
                    flow.dispatch(PracticeSetupAction::LoadProfile {
                        draft,
                        options,
                        advanced,
                    });
                    let profile = (*profiles)
                        .as_ref()
                        .ok()
                        .and_then(|saved| saved.iter().find(|profile| profile.name == selected))
                        .cloned();
                    loaded.set(profile);
                    name.set(selected);
                    error.set(None);
                    saved_notice.set(None);
                }
                Err(message) => {
                    saved_notice.set(None);
                    error.set(Some(message));
                }
            }
        })
    };
    let on_request_delete = {
        let pending_delete = pending_delete.clone();
        Callback::from(move |name: String| pending_delete.set(Some(name)))
    };
    let on_cancel_delete = {
        let pending_delete = pending_delete.clone();
        Callback::from(move |_: MouseEvent| pending_delete.set(None))
    };
    let on_confirm_delete = {
        let storage = storage.clone();
        let profiles = profiles.clone();
        let error = error.clone();
        let pending_delete = pending_delete.clone();
        let loaded = loaded.clone();
        let profile_name = name.clone();
        let saved_notice = saved_notice.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(name) = &*pending_delete else {
                return;
            };
            match practice_profiles::remove(&storage, name) {
                Ok(()) => {
                    profiles.set(practice_profiles::read(&storage));
                    pending_delete.set(None);
                    if (*loaded)
                        .as_ref()
                        .is_some_and(|profile| profile.name == name.as_str())
                    {
                        loaded.set(None);
                        profile_name.set(String::new());
                    }
                    saved_notice.set(None);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    ProfileControls {
        profiles,
        name,
        error,
        loaded,
        saved_notice,
        pending_delete,
        on_name_change,
        on_save,
        on_load,
        on_request_delete,
        on_confirm_delete,
        on_cancel_delete,
    }
}
