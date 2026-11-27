mod drawer;
mod excluded_words;

use self::{drawer::DrawerPreferences, excluded_words::ExcludedWords};
use crate::pages::WordSetsPage;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProfileTab {
    WordSets,
    ExcludedWords,
    Drawer,
}

#[function_component(ProfilePage)]
pub fn profile_page() -> Html {
    let tab = use_state(|| ProfileTab::WordSets);
    let choose_sets = {
        let tab = tab.clone();
        Callback::from(move |_| tab.set(ProfileTab::WordSets))
    };
    let choose_exclusions = {
        let tab = tab.clone();
        Callback::from(move |_| tab.set(ProfileTab::ExcludedWords))
    };
    let choose_drawer = {
        let tab = tab.clone();
        Callback::from(move |_| tab.set(ProfileTab::Drawer))
    };
    html! {
        <section data-component="profile-page" class="mx-auto flex w-full max-w-7xl flex-col gap-5 py-4">
            <header>
                <h1 class="text-2xl font-bold">{"Profile"}</h1>
            </header>
            <div role="tablist" aria-label="Profile settings" class="grid grid-cols-3 gap-1 rounded-xl border border-line bg-surface p-1">
                <button type="button" role="tab" aria-selected={(*tab == ProfileTab::WordSets).to_string()} class={classes!("min-h-11", "rounded-lg", "px-2", "text-sm", "font-semibold", if *tab == ProfileTab::WordSets { "bg-brand-action text-white" } else { "text-muted hover:bg-raised" })} onclick={choose_sets}>{"Word Sets"}</button>
                <button type="button" role="tab" aria-selected={(*tab == ProfileTab::ExcludedWords).to_string()} class={classes!("min-h-11", "rounded-lg", "px-2", "text-sm", "font-semibold", if *tab == ProfileTab::ExcludedWords { "bg-brand-action text-white" } else { "text-muted hover:bg-raised" })} onclick={choose_exclusions}>{"Excluded Words"}</button>
                <button type="button" role="tab" aria-selected={(*tab == ProfileTab::Drawer).to_string()} class={classes!("min-h-11", "rounded-lg", "px-2", "text-sm", "font-semibold", if *tab == ProfileTab::Drawer { "bg-brand-action text-white" } else { "text-muted hover:bg-raised" })} onclick={choose_drawer}>{"Drawer Menu"}</button>
            </div>
            <div role="tabpanel">
                {match *tab {
                    ProfileTab::WordSets => html! { <WordSetsPage /> },
                    ProfileTab::ExcludedWords => html! { <ExcludedWords /> },
                    ProfileTab::Drawer => html! { <DrawerPreferences /> },
                }}
            </div>
        </section>
    }
}
