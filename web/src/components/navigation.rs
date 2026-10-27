use yew::prelude::*;
use yew_router::prelude::*;

use crate::route::Route;

fn active(current: Option<&Route>, destination: &Route) -> bool {
    match destination {
        Route::PracticeSetup => matches!(
            current,
            Some(
                Route::PracticeSetup
                    | Route::PracticeEngPinyin
                    | Route::PracticePinyinHanzi
                    | Route::PracticeHanziEng
                    | Route::Random
            )
        ),
        _ => current == Some(destination),
    }
}

#[function_component(Navigation)]
pub fn navigation() -> Html {
    let current = use_route::<Route>();
    html! {
        <nav class="primary-navigation" aria-label="Main navigation">
            {for [(Route::Home, "Home", "01"), (Route::PracticeSetup, "Practice", "02"), (Route::Browse, "Browse", "03")].into_iter().map(|(route, label, number)| {
                let selected = active(current.as_ref(), &route);
                html! {
                    <a href={format!("#{}", route.to_path())} class={classes!("navigation-link", selected.then_some("is-active"))} aria-current={if selected {"page"} else {"false"}}>
                        <span class="navigation-number" aria-hidden="true">{number}</span>
                        <span>{label}</span>
                        <span class="navigation-indicator" aria-hidden="true" />
                    </a>
                }
            })}
        </nav>
    }
}
