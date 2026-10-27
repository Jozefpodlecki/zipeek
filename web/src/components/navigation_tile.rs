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
        <Link<Route> to={props.route.clone()} classes="navigation-tile">
            <span class="tile-icon" aria-hidden="true">
                <Icon data={props.icon} width="24px" height="24px" />
            </span>
            <h2>{props.title.clone()}</h2>
            <p>{props.description.clone()}</p>
        </Link<Route>>
    }
}
