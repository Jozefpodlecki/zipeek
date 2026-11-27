use crate::{
    components::HskPreferencesSelector,
    hooks::use_app_environment,
    preferences::{HskPreferences, PreferencesStore},
};
use yew::prelude::*;

#[function_component(HskSettings)]
pub(super) fn hsk_settings() -> Html {
    let environment = use_app_environment();
    let storage = environment.local_storage.clone();
    let selection = use_state(move || {
        PreferencesStore::new(&storage)
            .read()
            .hsk_defaults
            .normalized()
    });
    let error = use_state(|| None::<String>);
    let on_change = {
        let selection = selection.clone();
        let error = error.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |next: HskPreferences| {
            let next = next.normalized();
            match PreferencesStore::new(&storage).update(|prefs| prefs.hsk_defaults = next.clone())
            {
                Ok(()) => {
                    selection.set(next);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    let on_standard = {
        let selected = (*selection).clone();
        let on_change = on_change.clone();
        Callback::from(move |system| {
            let mut next = selected.clone();
            next.select_standard(system);
            on_change.emit(next);
        })
    };
    let on_toggle = {
        let selected = (*selection).clone();
        let on_change = on_change.clone();
        Callback::from(move |band| {
            let mut next = selected.clone();
            next.toggle(band);
            on_change.emit(next);
        })
    };
    html! {
        <section data-component="profile-hsk" class="rounded-2xl border border-line bg-surface p-4 sm:p-5">
            <HskPreferencesSelector standard={selection.standard} bands={selection.selected_bands()} {on_standard} {on_toggle} />
            {if let Some(message) = &*error { html! { <p role="alert" class="mt-3 text-sm text-red-400">{message}</p> } } else { Html::default() }}
        </section>
    }
}
