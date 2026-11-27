use crate::{
    components::shell::navigation::{NAVIGATION_ITEMS, drawer_item_is_required},
    hooks::use_app_environment,
    preferences::PreferencesStore,
};
use yew::prelude::*;
use yew_router::Routable;

#[function_component(DrawerPreferences)]
pub(super) fn drawer_preferences() -> Html {
    let environment = use_app_environment();
    let storage = environment.local_storage.clone();
    let hidden = use_state(move || PreferencesStore::new(&storage).read().hidden_drawer_items);
    let error = use_state(|| None::<String>);
    html! {
        <section data-component="profile-drawer-settings" class="rounded-2xl border border-line bg-surface p-4 sm:p-5">
            {if let Some(message) = &*error { html! { <p role="alert" class="mb-3 text-sm text-red-400">{message}</p> } } else { Html::default() }}
            <div class="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
                {for NAVIGATION_ITEMS.iter().map(|item| {
                    let route = item.route.clone();
                    let required = drawer_item_is_required(&route);
                    let path = route.to_path();
                    let checked = required || !hidden.contains(&path);
                    let storage = environment.local_storage.clone();
                    let window = environment.window.clone();
                    let hidden = hidden.clone();
                    let error = error.clone();
                    let onchange = Callback::from(move |event: Event| {
                        if required { return; }
                        let visible = event.target_unchecked_into::<web_sys::HtmlInputElement>().checked();
                        let update = PreferencesStore::new(&storage).update(|preferences| {
                            preferences.hidden_drawer_items.retain(|value| value != &path);
                            if !visible { preferences.hidden_drawer_items.push(path.clone()); }
                        });
                        match update {
                            Ok(()) => {
                                hidden.set(PreferencesStore::new(&storage).read().hidden_drawer_items);
                                error.set(None);
                                if let Ok(change) = web_sys::Event::new("zipeek-navigation-change") {
                                    let _ = window.dispatch_event(&change);
                                }
                            }
                            Err(message) => error.set(Some(message)),
                        }
                    });
                    html! {
                        <label key={item.label} class="flex min-h-12 cursor-pointer items-center justify-between gap-4 rounded-xl border border-line px-3 py-2 text-sm hover:bg-raised">
                            <span class="flex items-center gap-3"><Icon data={item.icon} width="18px" height="18px" />{item.label}</span>
                            <input type="checkbox" class="size-5 accent-[var(--color-brand-action)]" checked={checked} disabled={required} {onchange} aria-label={format!("Show {} in drawer", item.label)} />
                        </label>
                    }
                })}
            </div>
        </section>
    }
}
