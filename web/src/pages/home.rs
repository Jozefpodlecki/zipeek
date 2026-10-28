use yew::prelude::*;
use yew_icons::IconData;

use crate::{
    components::{DailyWordCard, NavigationTile},
    route::Route,
};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {}

#[function_component(Home)]
pub fn home(_props: &Props) -> Html {
    html! {
        <div data-home="" class="flex flex-col gap-6">
            <DailyWordCard />
            <nav aria-label="Practice, browse, or random">
                <div data-navigation-tiles="" class="grid grid-cols-1 gap-4 lg:grid-cols-3">
                    <NavigationTile route={Route::PracticeSetup} title="Practice" description="Connect hanzi, pinyin, and meaning." icon={IconData::LUCIDE_GRADUATION_CAP} />
                    <NavigationTile route={Route::Browse} title="Browse" description="Explore words and their readings." icon={IconData::LUCIDE_BOOK_OPEN} />
                    <NavigationTile route={Route::Random} title="Random" description="Take a new word as your starting point." icon={IconData::LUCIDE_SHUFFLE} />
                </div>
            </nav>
        </div>
    }
}
