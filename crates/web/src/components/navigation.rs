use yew::prelude::*;
use yew_router::prelude::*;
use yew_icons::{Icon, IconData};

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
                    | Route::SentenceTranslation
                    | Route::Speaking
            )
        ),
        _ => current == Some(destination),
    }
}

#[derive(Properties, PartialEq)]
pub struct NavigationProps {
    #[prop_or_default]
    pub collapsed: bool,
}

#[function_component(Navigation)]
pub fn navigation(props: &NavigationProps) -> Html {
    let current = use_route::<Route>();
    html! {
        <nav data-component="main-navigation" class="flex flex-wrap gap-[.4rem] md:flex-col md:gap-2" aria-label="Main navigation">
            {for [(Route::Home, "Home", IconData::LUCIDE_HOME), (Route::PracticeSetup, "Practice", IconData::LUCIDE_GRADUATION_CAP), (Route::Browse, "Browse", IconData::LUCIDE_BOOK_OPEN), (Route::WordSets, "Word Sets", IconData::LUCIDE_BOOKMARK), (Route::Random, "Random", IconData::LUCIDE_SHUFFLE), (Route::Settings, "Settings", IconData::LUCIDE_SETTINGS), (Route::About, "About", IconData::LUCIDE_INFO)].into_iter().map(|(route, label, icon)| {
                let selected = active(current.as_ref(), &route);
                html! {
                    <a href={format!("#{}", route.to_path())} data-navigation-link="" class={classes!("flex", "min-h-10", "flex-1", "items-center", "justify-center", "rounded-[.65rem]", "px-[.6rem]", "py-[.6rem]", "text-[.8rem]", "font-semibold", "no-underline", "transition-colors", "duration-150", "motion-reduce:transition-none", "md:min-h-12", "md:py-[.8rem]", "md:text-sm", if selected {"bg-blue-600/85 text-white shadow-sm"} else {"text-muted hover:bg-raised hover:text-ink"}, if props.collapsed {"md:justify-center md:px-2"} else {"md:justify-start md:gap-[.9rem] md:px-[.9rem]"})} aria-current={if selected {"page"} else {"false"}} title={label}>
                        <span aria-hidden="true" class="inline-flex shrink-0 items-center justify-center"><Icon data={icon} width="19px" height="19px" /></span>
                        {if !props.collapsed {html! {<span>{label}</span>}} else {Html::default()}}
                    </a>
                }
            })}
        </nav>
    }
}
