use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;

use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct NavigationTileProps {
    pub route: Route,
    pub title: AttrValue,
    pub description: AttrValue,
    pub icon: IconData,
}

#[function_component(NavigationTile)]
pub fn navigation_tile(props: &NavigationTileProps) -> Html {
    html! {
        <Link<Route> to={props.route.clone()} classes="flex flex-col rounded-[.9rem] border border-line bg-surface p-6 no-underline backdrop-blur-[12px] transition-[background-color,border-color,color,transform] duration-150 motion-safe:hover:-translate-y-[3px] hover:border-accent motion-reduce:translate-none motion-reduce:transform-none motion-reduce:transition-none lg:min-h-42">
            <span class="grid size-11 place-items-center rounded-[.8rem] bg-accent-soft text-accent" aria-hidden="true">
                <Icon data={props.icon} width="24px" height="24px" />
            </span>
            <h2 class="mt-[1.35rem] mb-2 text-[1.1rem] font-semibold tracking-[-.025em] text-ink">{props.title.clone()}</h2>
            <p class="text-[.8rem] leading-[1.7] text-muted">{props.description.clone()}</p>
        </Link<Route>>
    }
}
