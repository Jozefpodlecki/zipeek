use super::{
    super::tiles::TileSelection,
    tile::{MetricTile, TileAction},
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct MetricCatalogueProps {
    pub tiles: TileSelection,
    pub on_action: Callback<TileAction>,
}

#[function_component(MetricCatalogue)]
pub(super) fn metric_catalogue(props: &MetricCatalogueProps) -> Html {
    let search = use_state(String::new);
    let category = use_state(|| "All".to_owned());
    let on_search = {
        let search = search.clone();
        Callback::from(move |event: InputEvent| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            search.set(input.value());
        })
    };
    let normalized = search.trim().to_lowercase();
    let matched = props
        .tiles
        .order
        .iter()
        .copied()
        .filter(|tile| {
            (*category == "All" || tile.group() == *category)
                && (tile.label().to_lowercase().contains(&normalized)
                    || tile.group().to_lowercase().contains(&normalized))
        })
        .collect::<Vec<_>>();
    let has_matches = !matched.is_empty();
    html! {
        <section class="rounded-xl border border-line bg-surface p-3" aria-label="Metric library">
            <div class="flex flex-wrap items-end justify-between gap-3">
                <h3 class="font-semibold">{"Metric library"}</h3>
                <label class="flex w-full max-w-xs flex-col gap-1 text-xs font-medium text-muted">
                    {"Find a metric"}
                    <input type="search" value={(*search).clone()} oninput={on_search} placeholder="Search name or category"
                        class="min-h-10 w-full rounded-lg border border-line bg-raised px-3 text-sm text-ink outline-none placeholder:text-muted focus-visible:outline-2 focus-visible:outline-accent" />
                </label>
            </div>
            <div class="mt-2 flex flex-wrap gap-1" role="group" aria-label="Metric categories">
                {for ["All", "Vocabulary", "Activity coverage", "Practice history"].into_iter().map(|name| {
                    let category = category.clone();
                    let active = *category == name;
                    let select = Callback::from(move |_: MouseEvent| category.set(name.to_owned()));
                    html! {
                        <button key={name} type="button" aria-pressed={active.to_string()} onclick={select}
                            class={classes!("min-h-9", "rounded-md", "px-3", "text-xs", "font-medium", "focus-visible:outline-2", "focus-visible:outline-accent", if active { "bg-brand-action text-white" } else { "border border-line bg-raised text-ink hover:bg-surface" })}>{name}</button>
                    }
                })}
            </div>
            <div class="mt-2 grid gap-1 lg:grid-cols-2" data-stats-tile-library="">
                {for matched.into_iter().map(|tile| {
                    let index = props.tiles.order.iter().position(|value| *value == tile).unwrap_or(0);
                    html! {
                        <MetricTile key={tile.label()} tile={tile}
                            enabled={props.tiles.visible.contains(&tile)}
                            can_up={index > 0} can_down={index + 1 < props.tiles.order.len()}
                            on_action={props.on_action.clone()} />
                    }
                })}
            </div>
            if !has_matches {
                <p role="status" class="mt-4 rounded-lg bg-raised p-4 text-sm text-muted">{"No metrics match that search."}</p>
            }
        </section>
    }
}
