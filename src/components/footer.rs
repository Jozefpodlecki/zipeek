use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{route::Route, models::Social};
use yew_icons::{Icon, IconData};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub social: Social
}

#[function_component(Footer)]
pub fn footer(props: &Props) -> Html {
    html! {
        <footer class="h-10 bg-black flex items-center px-10">
           <a class="font-[roboto] text-white flex items-center gap-1 hover:underline" 
               href={props.social.portfolio.clone()}>
                {"© Jozef Podlecki 2026"}
                <Icon data={IconData::LUCIDE_EXTERNAL_LINK} width={"14px"}/>
            </a>
           <div class="flex gap-2 ml-auto text-white">
                <a class="transition-all dark:text-white hover:bg-black/80 hover:opacity-50" href={props.social.github.clone()}>
                    <Icon data={IconData::SIMPLE_ICONS_GITHUB} width={"20px"}/>
                </a>
                <a class="transition-all dark:text-white hover:bg-black/80 hover:opacity-50" href={props.social.linkedin.clone()}>
                    <Icon data={IconData::SIMPLE_ICONS_LINKEDIN} width={"20px"}/>
                </a>
           </div>
        </footer>
    }
}