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
    #[at("/browse")]
    Browse,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Random => html! { <Random /> },
        Route::PracticeSetup => html! { <PracticeSetup /> },
        Route::PracticeEngPinyin => html! { <Practice variant={Variant::EngPinyin} /> },
        Route::PracticePinyinHanzi => html! { <Practice variant={Variant::PinyinHanzi} /> },
        Route::PracticeHanziEng => html! { <Practice variant={Variant::HanziEng} /> },
        Route::Browse => html! { <Browse /> },
        Route::NotFound => html! { <NotFound /> },
    }
}
