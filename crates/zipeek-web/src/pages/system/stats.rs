use yew::prelude::*;

#[function_component(StatsPage)]
pub fn stats_page() -> Html {
    html! {
        <main
            class="mx-auto flex w-full max-w-4xl flex-col gap-5 px-4 py-8 sm:py-12"
            data-stats-page=""
        >
            <header class="space-y-2">
                <h1 class="text-3xl font-bold">{"Learning statistics"}</h1>
                <p class="text-muted">{"Your private learning progress, when verified learning evidence is available."}</p>
            </header>
            <section
                class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8"
                aria-labelledby="stats-data-status-heading"
                data-stats-unavailable=""
            >
                <h2 id="stats-data-status-heading" class="text-xl font-semibold">
                    {"Learning statistics are not available yet"}
                </h2>
                <p class="mt-3 max-w-2xl leading-7 text-muted" role="status">
                    {"The pedagogical database and its browser worker are not connected to this dashboard. There is no verified pedagogical evidence pipeline yet, so no skill, mastery or retention statistics can be shown reliably."}
                </p>
                <p class="mt-3 max-w-2xl text-sm text-muted">
                    {"Practice and courses remain available. This page does not read personal learning records, launch the worker, or estimate progress from game scores."}
                </p>
                <div class="mt-5 flex flex-wrap gap-3">
                    <a
                        href="#/settings?section=learning"
                        class="inline-flex min-h-11 items-center rounded-lg border border-line bg-raised px-4 font-semibold text-ink no-underline hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action"
                    >
                        {"Learning Data settings"}
                    </a>
                </div>
            </section>
        </main>
    }
}
