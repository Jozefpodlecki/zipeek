use yew::*;
use yew_router::Routable;

use crate::pages::*;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/random")]
    Random,
    #[at("/practice")]
    PracticeSetup,
    #[at("/practice/eng-pinyin")]
    PracticeEngPinyin,
    #[at("/practice/pinyin-hanzi")]
    PracticePinyinHanzi,
    #[at("/practice/hanzi-eng")]
    PracticeHanziEng,
    #[at("/practice/mcq/hanzi-to-meaning")]
    PracticeHanziToMeaning,
    #[at("/practice/mcq/meaning-to-hanzi")]
    PracticeMeaningToHanzi,
    #[at("/practice/mcq/meaning-to-pinyin")]
    PracticeMeaningToPinyin,
    #[at("/browse")]
    Browse,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    let not_found = routes == Route::NotFound;
    let page = match routes {
        Route::Home => html! { <Home /> },
        Route::Random => html! { <Random /> },
        Route::PracticeSetup => html! { <PracticeSetup /> },
        Route::PracticeEngPinyin => html! { <Practice variant={Variant::EngPinyin} /> },
        Route::PracticePinyinHanzi => html! { <Practice variant={Variant::PinyinHanzi} /> },
        Route::PracticeHanziEng => html! { <Practice variant={Variant::HanziEng} /> },
        Route::PracticeHanziToMeaning => html! { <Practice variant={Variant::HanziToMeaning} /> },
        Route::PracticeMeaningToHanzi => html! { <Practice variant={Variant::MeaningToHanzi} /> },
        Route::PracticeMeaningToPinyin => html! { <Practice variant={Variant::MeaningToPinyin} /> },
        Route::Browse => html! { <Browse /> },
        Route::NotFound => html! { <NotFound /> },
    };
    if not_found {
        page
    } else {
        html! { <crate::components::DatabaseBoundary>{page}</crate::components::DatabaseBoundary> }
    }
}

impl Route {
    pub fn practice(variant: Variant) -> Self {
        match variant {
            Variant::EngPinyin => Self::PracticeEngPinyin,
            Variant::PinyinHanzi => Self::PracticePinyinHanzi,
            Variant::HanziEng => Self::PracticeHanziEng,
            Variant::HanziToMeaning => Self::PracticeHanziToMeaning,
            Variant::MeaningToHanzi => Self::PracticeMeaningToHanzi,
            Variant::MeaningToPinyin => Self::PracticeMeaningToPinyin,
        }
    }
    pub fn title(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Random => "Random",
            Self::PracticeSetup => "Practice",
            Self::PracticeEngPinyin => "English to pinyin",
            Self::PracticePinyinHanzi => "Pinyin to hanzi",
            Self::PracticeHanziEng => "Hanzi to English",
            Self::PracticeHanziToMeaning => "Hanzi to meaning",
            Self::PracticeMeaningToHanzi => "Meaning to hanzi",
            Self::PracticeMeaningToPinyin => "Meaning to pinyin",
            Self::Browse => "Browse",
            Self::NotFound => "Page not found",
        }
    }
}
