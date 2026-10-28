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
                    | Route::PracticeHanziToMeaning
                    | Route::PracticeMeaningToHanzi
                    | Route::PracticeMeaningToPinyin
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
        <nav class="flex flex-row gap-[.4rem] md:flex-col md:gap-2" aria-label="Main navigation">
            {for [(Route::Home, "Home"), (Route::PracticeSetup, "Practice"), (Route::Browse, "Browse")].into_iter().map(|(route, label)| {
                let selected = active(current.as_ref(), &route);
                html! {
                    <a href={format!("#{}", route.to_path())} data-navigation-link="" class="flex min-h-10 flex-1 items-center justify-center rounded-[.65rem] px-[.6rem] py-[.6rem] text-[.8rem] font-[550] text-muted no-underline transition-colors duration-150 hover:bg-raised hover:text-ink motion-reduce:transition-none md:min-h-12 md:justify-start md:gap-[.9rem] md:px-[.9rem] md:py-[.8rem] md:text-sm" aria-current={if selected {"page"} else {"false"}}>
                        <span>{label}</span>
                    </a>
                }
            })}
        </nav>
    }
}
