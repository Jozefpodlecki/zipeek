use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(AboutPage)]
pub fn about_page() -> Html {
    html! {
        <section data-component="about-page" class="mx-auto flex w-full max-w-4xl flex-col gap-8 py-4 sm:py-10">
            <header class="space-y-3 text-center">
                <span class="inline-flex justify-center text-accent"><Icon data={IconData::LUCIDE_BOOK_OPEN} width="30px" height="30px" /></span>
                <h1 class="text-3xl font-bold tracking-tight sm:text-4xl">{"About Zipeek"}</h1>
                <p class="mx-auto max-w-2xl text-base leading-relaxed text-muted">{"A Chinese vocabulary browser and practice space built around the connection between characters, pronunciation, meaning, and context."}</p>
            </header>
            <div class="grid gap-5 md:grid-cols-2">
                <article class="rounded-2xl border border-line bg-surface p-6 sm:p-8">
                    <h2 class="mb-3 text-lg font-semibold">{"Why Zipeek exists"}</h2>
                    <p class="text-sm leading-7 text-muted">{"A word is more than a translation. Hanzi, pinyin, meanings, HSK membership, parts of speech, and topic classifications should be easy to explore together. Zipeek brings those connections into one approachable interface."}</p>
                </article>
                <article class="rounded-2xl border border-line bg-surface p-6 sm:p-8">
                    <h2 class="mb-3 text-lg font-semibold">{"Designed for active learning"}</h2>
                    <p class="text-sm leading-7 text-muted">{"Browse the dictionary, collect words into personal sets, discover unfamiliar vocabulary, and practise recognition and translation. The goal is to make it easy to move from discovery to practice."}</p>
                </article>
                <article class="rounded-2xl border border-line bg-surface p-6 sm:p-8">
                    <h2 class="mb-3 text-lg font-semibold">{"Data matters"}</h2>
                    <p class="text-sm leading-7 text-muted">{"Zipeek uses a pregenerated local dictionary and classifies word senses by grammatical role and topic. Descriptive notes, such as abbreviations, should be distinguished from primary definitions wherever the data supports that distinction. Classification and cross-references are still being refined."}</p>
                </article>
                <article class="rounded-2xl border border-line bg-surface p-6 sm:p-8">
                    <h2 class="mb-3 text-lg font-semibold">{"Built with care, still evolving"}</h2>
                    <p class="text-sm leading-7 text-muted">{"The app runs client-side using Rust, Yew, WebAssembly, and DuckDB. Features and data are under active development. Some learning tools, including verified example sentences, are not yet available."}</p>
                </article>
            </div>
            <div class="rounded-2xl border border-line bg-surface px-6 py-5 text-sm leading-7 text-muted sm:px-8">
                <h2 class="mb-2 font-semibold text-ink">{"Dictionary attribution"}</h2>
                <p>{"Chinese dictionary fallback data includes CC-CEDICT, distributed under CC BY-SA 4.0. Source attribution and license notices are available in the app footer."}</p>
            </div>
        </section>
    }
}
