use yew::prelude::*;

use crate::components::browse::{BrowseResults, BrowseSearchForm};

#[function_component(Browse)]
pub fn browse() -> Html {
    html! {
        <section
            data-component="browse-page"
            data-browse=""
            class="mx-auto flex w-full max-w-6xl flex-col gap-5 px-1 py-2"
        >
            <header
                class="flex flex-col gap-2"
            >
                <p
                    class="text-sm text-muted"
                >
                    {"Search by meaning, Hanzi or pinyin. Add HSK, grammar and topic filters as chips."}
                </p>
            </header>
            <div
                class="flex min-w-0 flex-col gap-4"
            >
                <BrowseSearchForm />
                <BrowseResults />
            </div>

        </section>
    }
}
