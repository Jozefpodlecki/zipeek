use yew::prelude::*;
use zipseek_core::OwnedLexemeNeighbors;


#[derive(Properties)]
pub struct Props {
    pub value: Option<OwnedLexemeNeighbors>
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

#[function_component(Loaded)]
pub fn loaded(props: &Props) -> Html {

    html! {
        <p>{"dddd"}</p>
    }
}