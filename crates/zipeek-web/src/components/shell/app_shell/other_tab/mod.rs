mod coordinator;
mod identity;
mod protocol;
mod registry;

use yew::prelude::*;

use crate::hooks::use_app_environment;

use self::coordinator::TabCoordinator;

#[function_component(OtherTabIndicator)]
pub(super) fn other_tab_indicator() -> Html {
    let environment = use_app_environment();
    let other_open = use_state(|| false);
    let browser_mode = !environment.is_tauri;

    {
        let other_open = other_open.clone();
        use_effect_with(browser_mode, move |enabled| {
            let coordinator = if *enabled {
                TabCoordinator::start(other_open.clone()).ok()
            } else {
                None
            };
            if coordinator.is_none() {
                other_open.set(false);
            }
            move || drop(coordinator)
        });
    }

    if !*other_open {
        return Html::default();
    }
    html! {
        <div
            data-other-tab-warning=""
            role="status"
            class="fixed bottom-4 right-4 z-40 max-w-[min(24rem,calc(100vw-2rem))] rounded-xl border border-line bg-surface p-3 text-sm text-ink shadow-lg"
        >
            <strong>{"Zipeek is open in another tab"}</strong>
            <p class="mt-1 text-muted">{"Avoid saving learning activity in both tabs at once. Close the other tab before continuing."}</p>
        </div>
    }
}
