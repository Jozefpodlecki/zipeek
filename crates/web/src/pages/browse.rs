use yew::prelude::*;

use crate::{
    components::browse::{BrowseResults, BrowseSearchForm},
};

#[function_component(Browse)]
pub fn browse() -> Html {
    html! {
        <section data-component="browse-page" data-browse="" class="mx-auto flex max-w-7xl flex-col gap-7">
            <header class="flex flex-col gap-2">
                <h1 class="text-3xl font-bold tracking-tight">{"Dictionary"}</h1>
                <p class="text-sm text-muted">{"Search Chinese, pinyin and meanings. Combine filters to narrow results."}</p>
                <a href="#/word-sets" class="w-fit text-sm font-semibold text-accent hover:underline">{"Manage my Word Sets"}</a>
            </header>
            <div class="flex min-w-0 flex-col gap-5">
                <BrowseSearchForm />
                <BrowseResults />
            </div>

        </section>
    }
}
