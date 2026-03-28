use log::*;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {}

#[function_component(SettingsWidget)]
pub fn settings(props: &Props) -> Html {

    html! {
        <>
            <side class="absolute text-white">
                <button type="button" class="p-2">
                    <Icon data={IconData::LUCIDE_SETTINGS}/>
                </button>
            </side>
        </>
    }
}